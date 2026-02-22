//! DAG Viz SubDag builder.
//!
//! Wraps the dag_viz tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_dag_viz_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the dag_viz SubDag node.
pub fn build_dag_viz_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_dag_viz_graph_dsl()?;
    Ok(Node::subdag("dag_viz", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_dag_viz_subdag_is_subdag() {
        let node = build_dag_viz_subdag().expect("dag_viz subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "dag_viz");
    }

    #[test]
    fn test_dag_viz_subdag_has_core_nodes() {
        let node = build_dag_viz_subdag().expect("dag_viz subdag should build");
        match &node.body {
            NodeBody::SubDag(dag) => {
                assert!(
                    !dag.nodes.is_empty(),
                    "dag_viz DSL subdag should contain nodes"
                );
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
