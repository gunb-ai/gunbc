//! Infra SubDag builder.
//!
//! Wraps the infra orchestration tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_infra_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the infra SubDag node.
pub fn build_infra_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_infra_graph_dsl()?;
    Ok(Node::subdag("infra", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infra_subdag_is_subdag() {
        let node = build_infra_subdag().expect("infra subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "infra");
    }
}
