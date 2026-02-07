//! CI SubDag builder.
//!
//! Wraps the CI tool as a SubDag node using WorkspaceOp.

use crate::ci::{build_ci_graph, CIGraphOp};
use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_ir::Node;

/// Convert a CIGraphOp to WorkspaceOp.
fn convert_ci_op(op: CIGraphOp) -> WorkspaceOp {
    match op {
        CIGraphOp::CI(ci_op) => WorkspaceOp::Ci(ci_op),
        CIGraphOp::Codegen(codegen_op) => WorkspaceOp::Codegen(codegen_op),
        CIGraphOp::PrepareFileExists(pfe) => {
            WorkspaceOp::Primitive(gunbc_primitives::PrimitiveOp::EmbeddedFileExists(pfe))
        }
        CIGraphOp::Transport(t) => WorkspaceOp::Transport(t),
        CIGraphOp::CliTool(cli) => WorkspaceOp::Clippy(cli),
    }
}

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
pub fn build_ci_subdag() -> Node<WorkspaceOp> {
    let original = build_ci_graph().expect("CI graph should build");
    let converted_dag = convert_dag(original, &convert_ci_op);

    Node::subdag("ci", converted_dag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_ci_subdag_is_subdag() {
        let node = build_ci_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "ci");
    }

    #[test]
    fn test_ci_subdag_has_nodes() {
        let node = build_ci_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                // CI should have multiple nodes
                assert!(dag.nodes.len() > 10);
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
