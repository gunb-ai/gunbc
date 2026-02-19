//! Testgen SubDag builder.
//!
//! Wraps the testgen tool as a SubDag node using `DynOp`.

use crate::testgen_dag::build_testgen_graph;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};
use gunbc_testgen_registry::iter_dag_specs;
use std::path::Path;

/// Build the testgen SubDag node.
pub fn build_testgen_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let targets: Vec<_> = iter_dag_specs().collect();
    let dag = build_testgen_graph(&targets, Path::new("target/generated/tests"))?;
    Ok(Node::subdag("testgen", dag))
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
                assert!(dag.get_node(&"fs_env".into()).is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
