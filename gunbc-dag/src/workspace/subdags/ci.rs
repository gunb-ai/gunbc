//! CI SubDag builder.
//!
//! Wraps the CI tool as a SubDag node using WorkspaceOp.

use crate::workspace::WorkspaceOp;
use crate::ci::{build_ci_graph, CIGraphOp};
use gunbc_ir::{Dag, Node, Port};

/// Convert a Node<CIGraphOp> to Node<WorkspaceOp>.
fn convert_ci_node(node: Node<CIGraphOp>) -> Node<WorkspaceOp> {
    Node {
        id: node.id,
        inputs: node.inputs,
        outputs: node.outputs,
        body: match node.body {
            gunbc_ir::NodeBody::Opaque(op) => {
                gunbc_ir::NodeBody::Opaque(convert_ci_op(op))
            }
            gunbc_ir::NodeBody::SubDag(dag) => {
                gunbc_ir::NodeBody::SubDag(convert_ci_dag(dag))
            }
        },
    }
}

/// Convert a CIGraphOp to WorkspaceOp.
fn convert_ci_op(op: CIGraphOp) -> WorkspaceOp {
    match op {
        CIGraphOp::CI(ci_op) => WorkspaceOp::Ci(ci_op),
        CIGraphOp::PrepareFileExists(pfe) => {
            WorkspaceOp::Primitive(gunbc_primitives::PrimitiveOp::EmbeddedFileExists(pfe))
        }
        CIGraphOp::Transport(t) => WorkspaceOp::Transport(t),
        CIGraphOp::CliTool(cli) => WorkspaceOp::Clippy(cli),
        CIGraphOp::Env(env) => WorkspaceOp::Env(env),
    }
}

/// Convert a Dag<CIGraphOp> to Dag<WorkspaceOp>.
fn convert_ci_dag(dag: Dag<CIGraphOp>) -> Dag<WorkspaceOp> {
    Dag {
        nodes: dag.nodes.into_iter().map(convert_ci_node).collect(),
        edges: dag.edges,
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
    let converted_dag = convert_ci_dag(original);

    Node::subdag(
        "ci",
        vec![], // No inputs - CI is self-contained
        vec![
            // Main outputs from CI workflow
            Port::scalar("build_success", "Bool"),
            Port::scalar("test_success", "Bool"),
            Port::scalar("lint_success", "Bool"),
        ],
        converted_dag,
    )
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
