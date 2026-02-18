//! Pragma SubDag builder.
//!
//! Wraps the pragma tool as a SubDag node using WorkspaceOp.

use crate::pragma::{build_pragma_graph, PragmaGraphOp};
use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};
use gunbc_primitives::PrimitiveOp;

fn convert_pragma_op(op: PragmaGraphOp) -> WorkspaceOp {
    match op {
        PragmaGraphOp::Domain(pragma_op) => WorkspaceOp::Pragma(pragma_op),
        PragmaGraphOp::FsEnv(env) => WorkspaceOp::FsEnv(env),
        PragmaGraphOp::PrepareFileRead(read_op) => {
            WorkspaceOp::Primitive(PrimitiveOp::PrepareFileRead(read_op))
        }
        PragmaGraphOp::PrepareFileWrite(write_op) => {
            WorkspaceOp::Primitive(PrimitiveOp::PrepareFileWrite(write_op))
        }
        PragmaGraphOp::Blob(blob_op) => WorkspaceOp::Blob(blob_op),
        PragmaGraphOp::Transport(transport) => WorkspaceOp::Transport(transport),
    }
}

/// Build the pragma SubDag node.
pub fn build_pragma_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let original = build_pragma_graph()?;
    let converted_dag = convert_dag(original, &convert_pragma_op);
    Ok(Node::subdag("pragma", converted_dag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_pragma_subdag_is_subdag() {
        let node = build_pragma_subdag().expect("pragma subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "pragma");
    }

    #[test]
    fn test_pragma_subdag_has_core_nodes() {
        let node = build_pragma_subdag().expect("pragma subdag should build");
        match &node.body {
            NodeBody::SubDag(dag) => {
                assert!(dag.get_node(&"render_clippy".into()).is_some());
                assert!(dag.get_node(&"render_allowlist".into()).is_some());
                assert!(dag.get_node(&"render_policy".into()).is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
