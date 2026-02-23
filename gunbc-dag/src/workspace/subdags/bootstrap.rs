//! Bootstrap SubDag builder.
//!
//! Wraps the bootstrap tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_bootstrap_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the bootstrap SubDag node.
pub fn build_bootstrap_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_bootstrap_graph_dsl()?;
    Ok(Node::subdag("bootstrap", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_subdag_is_subdag() {
        let node = build_bootstrap_subdag().expect("bootstrap subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "bootstrap");
    }
}
