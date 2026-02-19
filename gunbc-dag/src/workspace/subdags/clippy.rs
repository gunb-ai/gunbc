//! Clippy SubDag builder.
//!
//! Wraps the clippy tool as a SubDag node using `DynOp`.

use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_clippy::build_clippy_graph;
use gunbc_exec::DynOp;
use gunbc_ir::Node;

/// Build the clippy SubDag node with custom arguments.
pub fn build_clippy_subdag(args: &[&str]) -> Node<WorkspaceOp> {
    let original = build_clippy_graph(args);
    let converted = convert_dag(original, &|op| DynOp::new(op));
    Node::subdag("clippy", converted)
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
    fn test_clippy_subdag_contains_upsert_nodes() {
        let node = build_clippy_subdag(&[]);

        if let NodeBody::SubDag(subdag) = &node.body {
            let ids: Vec<&str> = subdag.nodes.iter().map(|n| n.id.0.as_str()).collect();
            assert!(ids.contains(&"prepare_check"));
            assert!(ids.contains(&"execute_check"));
            assert!(ids.contains(&"parse_check"));
            assert!(ids.contains(&"prepare_install"));
            assert!(ids.contains(&"execute_install"));
            assert!(ids.contains(&"parse_install"));
            assert!(ids.contains(&"prepare_resolve"));
            assert!(ids.contains(&"execute_resolve"));
            assert!(ids.contains(&"parse_resolve"));
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
