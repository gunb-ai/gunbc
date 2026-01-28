use crate::node::Node;
use crate::types::{NodeId, PatternDecision, PortName, ToolId, TypeId};

/// An edge connecting an output port of one node to an input port of another.
#[derive(Debug, Clone)]
pub struct Edge {
    pub from_node: NodeId,
    pub from_port: PortName,
    pub to_node: NodeId,
    pub to_port: PortName,
}

/// A directed acyclic graph of nodes.
#[derive(Debug, Clone)]
pub struct Dag<T> {
    pub nodes: Vec<Node<T>>,
    pub edges: Vec<Edge>,
    pub metadata: DagMetadata,
}

/// Port definition on a node.
#[derive(Debug, Clone)]
pub struct Port {
    pub name: PortName,
    pub type_id: TypeId,
    /// Optional guard expression — if present and evaluates false, node produces Skipped.
    pub guard: Option<String>,
}

/// DAG-level metadata. Pattern decisions live here, keyed by tool, not repeated per node.
#[derive(Debug, Clone, Default)]
pub struct DagMetadata {
    pub pattern_decisions: Vec<PatternDecisionEntry>,
    /// If set, lowering uses this node's outputs as the SubDag's output boundary.
    /// If unset, lowering falls back to unconnected output ports.
    pub export_node: Option<NodeId>,
}

/// Records that a tool was evaluated against a pattern and a decision was made.
#[derive(Debug, Clone)]
pub struct PatternDecisionEntry {
    pub tool: ToolId,
    pub pattern: String,
    pub decision: PatternDecision,
}
