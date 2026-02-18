//! Docgen SubDag builder.
//!
//! Wraps the docgen tool as a SubDag node using WorkspaceOp.

use crate::docgen::{build_docgen_graph, DocgenGraphOp};
use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};
use gunbc_primitives::PrimitiveOp;

fn convert_docgen_op(op: DocgenGraphOp) -> WorkspaceOp {
    match op {
        DocgenGraphOp::Docgen(docgen_op) => WorkspaceOp::Docgen(docgen_op),
        DocgenGraphOp::FsEnv(env) => WorkspaceOp::FsEnv(env),
        DocgenGraphOp::PrepareFileRead(read_op) => WorkspaceOp::Primitive(PrimitiveOp::PrepareFileRead(read_op)),
        DocgenGraphOp::PrepareFileWrite(write_op) => WorkspaceOp::Primitive(PrimitiveOp::PrepareFileWrite(write_op)),
        DocgenGraphOp::Blob(blob_op) => WorkspaceOp::Blob(blob_op),
        DocgenGraphOp::Transport(transport) => WorkspaceOp::Transport(transport),
    }
}

/// Build the docgen SubDag node.
pub fn build_docgen_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let original = build_docgen_graph()?;
    let converted_dag = convert_dag(original, &convert_docgen_op);
    Ok(Node::subdag("docgen", converted_dag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_docgen_subdag_is_subdag() {
        let node = build_docgen_subdag().expect("docgen subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "docgen");
    }

    #[test]
    fn test_docgen_subdag_has_core_nodes() {
        let node = build_docgen_subdag().expect("docgen subdag should build");
        match &node.body {
            NodeBody::SubDag(dag) => {
                assert!(dag.get_node(&"render_ab_workflows_doc".into()).is_some());
                assert!(dag.get_node(&"ab_doc_template".into()).is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
