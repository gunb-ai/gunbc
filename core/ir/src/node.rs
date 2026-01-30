//! Node types for the DAG.

use crate::dag::{Dag, Port};
use crate::types::NodeId;
use serde::{Deserialize, Serialize};

/// A node in the DAG, generic over its operation type.
///
/// Nodes are pure transformations of inputs to outputs.
/// World-writes are determined structurally by boundary detection,
/// not by node annotations.
///
/// # Tool Requirements
///
/// Nodes can declare tool requirements via the `requires_tools` field.
/// The executor automatically handles tool acquisition (check/install)
/// before executing nodes with tool requirements.
///
/// ```ignore
/// let mut node = Node::opaque("lint", inputs, outputs, LintOp);
/// node.requires_tools.push("clippy".to_string());
/// ```
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
    /// Required tools (tool IDs). Framework injects acquisition sub-DAGs.
    /// Each tool ID here will have a corresponding "tool:{id}" input port.
    #[serde(default)]
    pub requires_tools: Vec<String>,
}

impl<T> Node<T> {
    /// Create a new opaque node.
    pub fn opaque(id: impl Into<NodeId>, inputs: Vec<Port>, outputs: Vec<Port>, op: T) -> Self {
        Self {
            id: id.into(),
            inputs,
            outputs,
            body: NodeBody::Opaque(op),
            requires_tools: Vec::new(),
        }
    }

    /// Create a new sub-DAG node.
    pub fn subdag(id: impl Into<NodeId>, inputs: Vec<Port>, outputs: Vec<Port>, dag: Dag<T>) -> Self {
        Self {
            id: id.into(),
            inputs,
            outputs,
            body: NodeBody::SubDag(dag),
            requires_tools: Vec::new(),
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
    
    /// Check if this node requires any tools.
    ///
    /// Tool requirements are set via `requires_tools.push("tool_id".to_string())`.
    /// The executor handles tool acquisition automatically.
    pub fn has_tool_requirements(&self) -> bool {
        !self.requires_tools.is_empty()
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
