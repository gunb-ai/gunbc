//! Testgen SubDag builder.
//!
//! Wraps the testgen tool as a SubDag node using `DynOp`.

use crate::dsl_builder::build_testgen_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the testgen SubDag node.
pub fn build_testgen_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_testgen_graph_dsl()?;
    Ok(Node::subdag("testgen", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_testgen_subdag_is_subdag() {
        let node = build_testgen_subdag().expect("testgen subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "testgen");
    }

    #[test]
    fn test_testgen_subdag_has_core_nodes() {
        let node = build_testgen_subdag().expect("testgen subdag should build");
        match &node.body {
            NodeBody::SubDag(dag) => {
                assert!(!dag.nodes.is_empty(), "testgen DSL subdag should contain nodes");
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
