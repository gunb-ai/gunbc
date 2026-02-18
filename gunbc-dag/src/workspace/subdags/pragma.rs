//! Pragma SubDag builder.
//!
//! Wraps the pragma tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_pragma_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the pragma SubDag node.
pub fn build_pragma_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_pragma_graph_dsl()?;
    Ok(Node::subdag("pragma", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pragma_subdag_is_subdag() {
        let node = build_pragma_subdag().expect("pragma subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "pragma");
    }
}
