//! CI SubDag builder.
//!
//! Wraps the CI tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_ci_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the CI SubDag node.
///
/// This wraps the CI workflow as a `Node<WorkspaceOp>` that can be
/// composed into the Workspace DAG.
///
/// # I/O Interface
///
/// Inputs: None (self-contained CI workflow)
///
/// Outputs:
/// - Various CI stage results (build, test, lint status)
pub fn build_ci_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_ci_graph_dsl()?;
    Ok(Node::subdag("ci", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_subdag_is_subdag() {
        let node = build_ci_subdag().expect("ci subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "ci");
    }
}
