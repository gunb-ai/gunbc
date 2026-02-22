//! Clippy SubDag builder.
//!
//! Wraps the clippy tool as a SubDag node using `DynOp`.

use crate::dsl_builder::build_clippy_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::Node;

/// Build the clippy SubDag node with custom arguments.
pub fn build_clippy_subdag(_args: &[&str]) -> Node<WorkspaceOp> {
    let dsl_dag = build_clippy_graph_dsl().expect("clippy DSL graph should build");
    Node::subdag("clippy", dsl_dag)
}

/// Build the clippy lint-all SubDag with standard flags.
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
    fn test_clippy_subdag_contains_nodes() {
        let node = build_clippy_subdag(&[]);

        if let NodeBody::SubDag(subdag) = &node.body {
            assert!(
                !subdag.nodes.is_empty(),
                "clippy DSL subdag should contain nodes"
            );
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
