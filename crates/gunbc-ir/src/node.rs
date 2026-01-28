use crate::dag::{Dag, Port};
use crate::types::NodeId;

/// A node in the DAG, generic over its operation type.
///
/// Nodes are pure transformations of inputs to outputs.
/// Effects are determined by where outputs flow (terminal sinks),
/// not by node annotations.
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub id: NodeId,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub body: NodeBody<T>,
}

/// The body of a node: either an opaque operation or a nested sub-DAG.
#[derive(Debug, Clone)]
pub enum NodeBody<T> {
    Opaque(T),
    SubDag(Dag<T>),
}
