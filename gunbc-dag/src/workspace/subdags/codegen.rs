//! Codegen SubDag builder.
//!
//! Wraps the codegen tool as a SubDag node using WorkspaceOp.

use crate::codegen::{build_codegen_graph, CodegenGraphOp};
use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

fn convert_codegen_op(op: CodegenGraphOp) -> WorkspaceOp {
    match op {
        CodegenGraphOp::Codegen(codegen_op) => WorkspaceOp::Codegen(codegen_op),
        CodegenGraphOp::FsEnv(env) => WorkspaceOp::FsEnv(env),
        CodegenGraphOp::Transport(transport) => WorkspaceOp::Transport(transport),
    }
}

/// Build the codegen SubDag node.
pub fn build_codegen_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let original = build_codegen_graph()?;
    let converted_dag = convert_dag(original, &convert_codegen_op);
    Ok(Node::subdag("codegen", converted_dag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_codegen_subdag_is_subdag() {
        let node = build_codegen_subdag().expect("codegen subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "codegen");
    }

    #[test]
    fn test_codegen_subdag_has_core_nodes() {
        let node = build_codegen_subdag().expect("codegen subdag should build");
        match &node.body {
            NodeBody::SubDag(dag) => {
                assert!(dag.get_node(&"codegen_exists".into()).is_some());
                assert!(dag.get_node(&"prepare_codegen_command".into()).is_some());
                assert!(dag.get_node(&"execute_codegen".into()).is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
