//! Testgen SubDag builder.
//!
//! Wraps the testgen tool as a SubDag node using WorkspaceOp.

use crate::testgen_dag::{build_testgen_graph, TestgenGraphOp};
use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};
use gunbc_primitives::PrimitiveOp;
use gunbc_testgen_registry::iter_dag_specs;
use std::path::Path;

fn convert_testgen_op(op: TestgenGraphOp) -> WorkspaceOp {
    match op {
        TestgenGraphOp::Domain(testgen_op) => WorkspaceOp::Testgen(testgen_op),
        TestgenGraphOp::FsEnv(env) => WorkspaceOp::FsEnv(env),
        TestgenGraphOp::PrepareFileRead(read_op) => {
            WorkspaceOp::Primitive(PrimitiveOp::PrepareFileRead(read_op))
        }
        TestgenGraphOp::PrepareFileWrite(write_op) => {
            WorkspaceOp::Primitive(PrimitiveOp::PrepareFileWrite(write_op))
        }
        TestgenGraphOp::Blob(blob_op) => WorkspaceOp::Blob(blob_op),
        TestgenGraphOp::Transport(transport) => WorkspaceOp::Transport(transport),
    }
}

/// Build the testgen SubDag node.
pub fn build_testgen_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let targets: Vec<_> = iter_dag_specs().collect();
    let original = build_testgen_graph(&targets, Path::new("target/generated/tests"))?;
    let converted_dag = convert_dag(original, &convert_testgen_op);
    Ok(Node::subdag("testgen", converted_dag))
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
