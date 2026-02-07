//! Node types for the DAG.

use crate::boundary::detect_boundaries;
use crate::dag::{Dag, Guard, Port};
use crate::entrypoint::detect_entrypoints;
use crate::types::{Cardinality, NodeId, PortName};
use crate::Value;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A node in the DAG, generic over its operation type.
///
/// Nodes are pure transformations of inputs to outputs.
/// World-writes are determined structurally by boundary detection,
/// not by node annotations.
///
/// # Tool Acquisition
///
/// Tools are acquired via an environment node that flows ToolHandle values
/// through DAG edges. Nodes that need tools declare `tool:*` input ports
/// and receive handles from upstream environment nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<T> {
    /// Unique identifier for this node
    pub id: NodeId,
    /// Input ports
    pub inputs: Vec<Port>,
    /// Output ports
    pub outputs: Vec<Port>,
    /// The node's body: either an opaque operation or a nested sub-DAG
    pub body: NodeBody<T>,
    /// I/O examples for test generation.
    ///
    /// Each example specifies concrete input values and expected output values.
    /// Testgen uses these to generate per-node unit tests that call
    /// `execute_single_node` with the given inputs and verify the outputs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<NodeIoExample>,
}

impl<T> Node<T> {
    /// Create a new opaque node.
    pub fn opaque(id: impl Into<NodeId>, inputs: Vec<Port>, outputs: Vec<Port>, op: T) -> Self {
        Self {
            id: id.into(),
            inputs,
            outputs,
            body: NodeBody::Opaque(op),
            examples: Vec::new(),
        }
    }

    /// Create a sub-DAG node with interface inferred from the inner DAG.
    ///
    /// Input ports are derived from the inner DAG's entrypoints (unconnected
    /// input ports). Output ports are derived from the inner DAG's boundaries
    /// (unconnected output ports). This makes it impossible to declare an
    /// interface that doesn't match the inner structure.
    ///
    /// Guards are stripped from inferred ports — use [`with_input_guard`] to
    /// add routing guards after construction.
    pub fn subdag(id: impl Into<NodeId>, dag: Dag<T>) -> Self {
        let entrypoints = detect_entrypoints(&dag);
        let boundaries = detect_boundaries(&dag);

        // Build input ports from entrypoints, deduplicated by name.
        // Guards are stripped — they're an internal routing concern.
        let mut seen_inputs = HashSet::new();
        let mut inputs = Vec::new();
        for (node_id, port_name, _) in &entrypoints.entrypoint_ports {
            if seen_inputs.insert(port_name.clone()) {
                if let Some(node) = dag.get_node(node_id) {
                    if let Some(port) = node.inputs.iter().find(|p| &p.name == port_name) {
                        let mut inferred = Port::with_cardinality(
                            port.name.0.as_str(),
                            port.type_id.0.as_str(),
                            port.cardinality,
                        );
                        // Preserve resource_access so SubDag auto-inference
                        // doesn't lose Write/Exclusive mode information.
                        inferred.resource_access = port.resource_access;
                        inputs.push(inferred);
                    }
                }
            }
        }

        // Build output ports from boundaries, deduplicated by name.
        let mut seen_outputs = HashSet::new();
        let mut outputs = Vec::new();
        for (node_id, port_name) in &boundaries.boundary_ports {
            if seen_outputs.insert(port_name.clone()) {
                if let Some(node) = dag.get_node(node_id) {
                    if let Some(port) = node.outputs.iter().find(|p| &p.name == port_name) {
                        let mut inferred = Port::with_cardinality(
                            port.name.0.as_str(),
                            port.type_id.0.as_str(),
                            port.cardinality,
                        );
                        inferred.resource_access = port.resource_access;
                        outputs.push(inferred);
                    }
                }
            }
        }

        // Sort by name for deterministic interface ordering.
        inputs.sort_by(|a, b| a.name.0.cmp(&b.name.0));
        outputs.sort_by(|a, b| a.name.0.cmp(&b.name.0));

        Self {
            id: id.into(),
            inputs,
            outputs,
            body: NodeBody::SubDag(dag),
            examples: Vec::new(),
        }
    }

    /// Add an I/O example for test generation.
    ///
    /// Examples are used by testgen to generate per-node unit tests.
    /// Each example specifies concrete input values and expected output values.
    ///
    /// ```ignore
    /// Node::opaque("prepare", inputs, outputs, MyOp::Prepare)
    ///     .with_example(
    ///         [("input".into(), Value::Str("hello".into()))].into(),
    ///         [("output".into(), Value::Str("HELLO".into()))].into(),
    ///     )
    /// ```
    pub fn with_example(
        mut self,
        inputs: HashMap<String, Value>,
        expected_outputs: HashMap<String, Value>,
    ) -> Self {
        self.examples.push(NodeIoExample {
            inputs,
            expected_outputs,
            description: None,
        });
        self
    }

    /// Add a described I/O example for test generation.
    ///
    /// Like `with_example`, but includes a description used in the generated
    /// test function name and doc comment.
    pub fn with_described_example(
        mut self,
        description: impl Into<String>,
        inputs: HashMap<String, Value>,
        expected_outputs: HashMap<String, Value>,
    ) -> Self {
        self.examples.push(NodeIoExample {
            inputs,
            expected_outputs,
            description: Some(description.into()),
        });
        self
    }

    /// Add a routing guard to an input port.
    ///
    /// Used by pattern builders (Branch, If) to control conditional execution.
    ///
    /// # Panics
    ///
    /// Panics if no input port with the given name exists.
    pub(crate) fn with_input_guard(mut self, port: &str, guard: Guard) -> Self {
        let port_name: PortName = port.into();
        let p = self
            .inputs
            .iter_mut()
            .find(|p| p.name == port_name)
            .unwrap_or_else(|| panic!("no input port '{}' on node '{}'", port, self.id));
        p.guard = Some(guard);
        self
    }

    /// Override the cardinality of an output port.
    ///
    /// Used by pattern builders when the SubDag's runtime behavior differs
    /// from the inner boundary's declared cardinality (e.g., If pattern
    /// produces optional output even though the inner boundary is scalar).
    pub(crate) fn with_output_cardinality(mut self, port: &str, cardinality: Cardinality) -> Self {
        let port_name: PortName = port.into();
        let p = self
            .outputs
            .iter_mut()
            .find(|p| p.name == port_name)
            .unwrap_or_else(|| panic!("no output port '{}' on node '{}'", port, self.id));
        p.cardinality = cardinality;
        self
    }

    /// Map the operation type for this node, recursively for sub-DAGs.
    pub fn map_ops<U, F>(self, f: &mut F) -> Node<U>
    where
        F: FnMut(T) -> U,
    {
        let body = match self.body {
            NodeBody::Opaque(op) => NodeBody::Opaque(f(op)),
            NodeBody::SubDag(dag) => NodeBody::SubDag(dag.map_ops(f)),
        };

        Node {
            id: self.id,
            inputs: self.inputs,
            outputs: self.outputs,
            body,
            examples: self.examples,
        }
    }

    /// Check if this node is opaque (not a sub-DAG).
    pub fn is_opaque(&self) -> bool {
        matches!(self.body, NodeBody::Opaque(_))
    }

    /// Check if this node is a sub-DAG.
    pub fn is_subdag(&self) -> bool {
        matches!(self.body, NodeBody::SubDag(_))
    }
}

/// The body of a node: either an opaque operation or a nested sub-DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeBody<T> {
    /// An opaque operation — we trust it, don't look inside
    Opaque(T),
    /// A nested sub-DAG — same structure, recursive
    SubDag(Dag<T>),
}

/// An I/O example for a node, used by testgen to generate per-node unit tests.
///
/// This is the "on-node" form of examples — it uses exact `Value` matching
/// for expected outputs. For richer matching (contains, non-empty, predicates),
/// use `NodeExample` + `OutputMatcher` from `gunbc-test` via MockSpec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIoExample {
    /// Input values keyed by port name.
    pub inputs: HashMap<String, Value>,
    /// Expected output values keyed by port name (exact match).
    pub expected_outputs: HashMap<String, Value>,
    /// Optional description for the generated test name.
    pub description: Option<String>,
}
