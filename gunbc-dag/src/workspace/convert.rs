//! Helpers for remapping DAG op types within workspace subdags.

use gunbc_ir::{Dag, Node, NodeBody};

/// Convert a Dag<S> to Dag<T> by applying an op-mapping function.
pub fn convert_dag<S, T, F>(dag: Dag<S>, f: &F) -> Dag<T>
where
    F: Fn(S) -> T,
{
    Dag {
        nodes: dag
            .nodes
            .into_iter()
            .map(|node| convert_node(node, f))
            .collect(),
        edges: dag.edges,
    }
}

/// Convert a Node<S> to Node<T> by applying an op-mapping function.
pub fn convert_node<S, T, F>(node: Node<S>, f: &F) -> Node<T>
where
    F: Fn(S) -> T,
{
    Node {
        id: node.id,
        inputs: node.inputs,
        outputs: node.outputs,
        body: match node.body {
            NodeBody::Opaque(op) => NodeBody::Opaque(f(op)),
            NodeBody::SubDag(dag) => NodeBody::SubDag(convert_dag(dag, f)),
        },
        examples: node.examples,
    }
}
