use crate::dag::{Dag, Port};
use crate::metadata::NodeMetadata;
use crate::types::NodeId;

/// A node in the DAG, generic over its operation type.
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub id: NodeId,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub metadata: NodeMetadata,
    pub body: NodeBody<T>,
}

/// The body of a node: either an opaque operation or a nested sub-DAG.
#[derive(Debug, Clone)]
pub enum NodeBody<T> {
    Opaque(T),
    SubDag(Dag<T>),
}
