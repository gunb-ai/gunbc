use crate::algebra::Predicate;
use crate::node::Node;
use crate::types::{NodeId, PatternDecision, PortName, TypeId};

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
    /// Optional guard predicate — if present and evaluates false, node produces Skipped.
    pub guard: Option<Predicate>,
}

/// DAG-level metadata.
#[derive(Debug, Clone, Default)]
pub struct DagMetadata {
    pub pattern_decisions: Vec<PatternDecisionEntry>,
    /// If set, lowering uses this node's outputs as the SubDag's output boundary.
    /// If unset, lowering falls back to unconnected output ports.
    pub export_node: Option<NodeId>,
    /// Declarations of ports that cross external boundaries (e.g., network, filesystem).
    /// Used by codegen to auto-derive mock flags for each transport layer.
    pub boundary_declarations: Vec<BoundaryDeclaration>,
}

/// Declares that a port crosses an external system boundary.
///
/// External boundaries are typed hierarchically (e.g., `External::GitHub::Gist` uses
/// `External::REST::*` which uses `External::HTTP::*`). This enables mocking at any
/// layer of the transport stack.
#[derive(Debug, Clone)]
pub struct BoundaryDeclaration {
    /// The node containing the boundary port.
    pub node: NodeId,
    /// The port that crosses the external boundary.
    pub port: PortName,
    /// The external type (e.g., `External::GitHub::Gist`, `External::HTTP::Request`).
    pub external_type: TypeId,
}

/// Records that a SubDag node instantiates (or opts out of) a pattern.
#[derive(Debug, Clone)]
pub struct PatternDecisionEntry {
    /// The SubDag node this decision applies to.
    pub node: NodeId,
    pub pattern: String,
    pub decision: PatternDecision,
}
