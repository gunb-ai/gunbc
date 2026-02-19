//! Codegen SubDag builder.
//!
//! Wraps the codegen tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_codegen_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the codegen SubDag node.
pub fn build_codegen_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_codegen_graph_dsl()?;
    Ok(Node::subdag("codegen", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_subdag_is_subdag() {
        let node = build_codegen_subdag().expect("codegen subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "codegen");
    }
}
