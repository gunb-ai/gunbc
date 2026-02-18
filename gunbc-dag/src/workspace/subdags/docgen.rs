//! Docgen SubDag builder.
//!
//! Wraps the docgen tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_docgen_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the docgen SubDag node.
pub fn build_docgen_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_docgen_graph_dsl()?;
    Ok(Node::subdag("docgen", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docgen_subdag_is_subdag() {
        let node = build_docgen_subdag().expect("docgen subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "docgen");
    }
}
