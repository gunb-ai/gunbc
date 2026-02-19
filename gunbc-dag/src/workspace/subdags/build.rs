//! Build SubDag builder.
//!
//! Wraps the build tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_build_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the build SubDag node.
pub fn build_build_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_build_graph_dsl()?;
    Ok(Node::subdag("build", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_subdag_is_subdag() {
        let node = build_build_subdag().expect("build subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "build");
    }
}
