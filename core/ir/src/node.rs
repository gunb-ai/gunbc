//! Node types for the DAG.

use crate::dag::{Dag, Port};
use crate::transport::cli::CliToolDef;
use crate::types::{Cardinality, NodeId, PortName, TypeId};
use serde::{Deserialize, Serialize};

/// A node in the DAG, generic over its operation type.
///
/// Nodes are pure transformations of inputs to outputs.
/// World-writes are determined structurally by boundary detection,
/// not by node annotations.
///
/// # Tool Requirements
///
/// Nodes can declare tool requirements via `.requires()`. The framework
/// automatically injects tool acquisition sub-DAGs and wires ToolHandle
/// values to the node's inputs.
///
/// ```ignore
/// Node::opaque("lint", inputs, outputs, LintOp)
///     .requires(&cli::CLIPPY)  // Adds "tool:clippy" input port
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
    
    /// Declare that this node requires a CLI tool.
    ///
    /// This adds the tool to the requirements list and creates an input port
    /// for the tool handle. During lowering, the framework will:
    /// 1. Inject a tool acquisition sub-DAG (upsert pattern)
    /// 2. Wire the ToolHandle output to this node's tool input port
    ///
    /// # Example
    ///
    /// ```ignore
    /// use gunbc_ir::transport::cli;
    ///
    /// Node::opaque("lint", inputs, outputs, LintOp)
    ///     .requires(&cli::CLIPPY)
    ///     .requires(&cli::RUSTFMT)  // Can require multiple tools
    /// ```
    ///
    /// The node's operation will receive the tool handles via inputs:
    ///
    /// ```ignore
    /// fn execute_lint(inputs: HashMap<String, Value>) -> Result<...> {
    ///     let clippy = inputs.get("tool:clippy").unwrap();
    ///     // ...
    /// }
    /// ```
    pub fn requires(mut self, tool: &'static CliToolDef) -> Self {
        let tool_id = tool.id.to_string();
        
        // Only add if not already required
        if !self.requires_tools.contains(&tool_id) {
            self.requires_tools.push(tool_id.clone());
            
            // Add an input port for the tool handle
            let port_name = format!("tool:{}", tool.id);
            self.inputs.push(Port {
                name: PortName(port_name),
                type_id: TypeId("ToolHandle".to_string()),
                cardinality: Cardinality::One,
                guard: None,
            });
        }
        
        self
    }
    
    /// Check if this node requires any tools.
    pub fn has_tool_requirements(&self) -> bool {
        !self.requires_tools.is_empty()
    }
    
    /// Get the list of required tool IDs.
    pub fn required_tools(&self) -> &[String] {
        &self.requires_tools
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
