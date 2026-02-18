//! DAG Viz SubDag builder.
//!
//! Wraps the dag_viz tool as a SubDag node using WorkspaceOp.

use crate::dag_viz::{build_dag_viz_graph, DagVizMode};
use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Node};

/// Build the dag_viz SubDag node.
pub fn build_dag_viz_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let original = build_dag_viz_graph(DagVizMode::Snapshot)?;
    let converted_dag = convert_dag(original, &|op| DynOp::new(op));
    Ok(Node::subdag("dag_viz", converted_dag))
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
                assert!(dag.get_node(&"build_topology".into()).is_some());
                assert!(dag.get_node(&"render_snapshot".into()).is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
