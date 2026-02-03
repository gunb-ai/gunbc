//! Node types for the DAG.

use crate::dag::{Dag, Port};
use crate::types::NodeId;
use crate::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    /// Create a new sub-DAG node.
    pub fn subdag(id: impl Into<NodeId>, inputs: Vec<Port>, outputs: Vec<Port>, dag: Dag<T>) -> Self {
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
