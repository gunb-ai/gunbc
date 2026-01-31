//! Clippy SubDag builder.
//!
//! Wraps the clippy tool as a SubDag node using WorkspaceOp.

use crate::workspace::WorkspaceOp;
use gunbc_clippy::build_clippy_upsert;
use gunbc_ir::transport::cli::CliToolOp;
use gunbc_ir::{Dag, Node};

/// Convert a Node<CliToolOp> to Node<WorkspaceOp>.
fn convert_clippy_node(node: Node<CliToolOp>) -> Node<WorkspaceOp> {
    Node {
        id: node.id,
        inputs: node.inputs,
        outputs: node.outputs,
        body: match node.body {
            gunbc_ir::NodeBody::Opaque(op) => {
                gunbc_ir::NodeBody::Opaque(WorkspaceOp::Clippy(op))
            }
            gunbc_ir::NodeBody::SubDag(dag) => {
                gunbc_ir::NodeBody::SubDag(convert_clippy_dag(dag))
            }
        },
    }
}

/// Convert a Dag<CliToolOp> to Dag<WorkspaceOp>.
fn convert_clippy_dag(dag: Dag<CliToolOp>) -> Dag<WorkspaceOp> {
    Dag {
        nodes: dag.nodes.into_iter().map(convert_clippy_node).collect(),
        edges: dag.edges,
    }
}

/// Build the clippy SubDag node with custom arguments.
///
/// This wraps the clippy upsert workflow as a `Node<WorkspaceOp>` that can be
/// composed into the Workspace DAG.
///
/// # Arguments
///
/// * `args` - Arguments to pass to clippy (e.g., `["--all-targets"]`)
///
/// # I/O Interface
///
/// The upsert pattern provides:
/// - Inputs: None (self-contained)
/// - Outputs: success, stdout, stderr from the run step
pub fn build_clippy_subdag(args: &[&str]) -> Node<WorkspaceOp> {
    convert_clippy_node(build_clippy_upsert(args))
}

/// Build the clippy lint-all SubDag with standard flags.
///
/// Uses `--all-targets -- -D warnings` for comprehensive linting.
pub fn build_clippy_lint_all_subdag() -> Node<WorkspaceOp> {
    build_clippy_subdag(&["--all-targets", "--", "-D", "warnings"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_clippy_subdag_is_subdag() {
        let node = build_clippy_subdag(&["--all-targets"]);
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "clippy");
    }

    #[test]
    fn test_clippy_subdag_contains_upsert_nodes() {
        let node = build_clippy_subdag(&[]);

        if let NodeBody::SubDag(subdag) = &node.body {
            // Upsert pattern has 3 nodes: check, create, resolve
            assert_eq!(subdag.nodes.len(), 3);

            let ids: Vec<&str> = subdag.nodes.iter().map(|n| n.id.0.as_str()).collect();
            assert!(ids.contains(&"check"));
            assert!(ids.contains(&"create"));
            assert!(ids.contains(&"resolve"));
        } else {
            panic!("Expected SubDag");
        }
    }

    #[test]
    fn test_clippy_lint_all_subdag() {
        let node = build_clippy_lint_all_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "clippy");
    }
}
