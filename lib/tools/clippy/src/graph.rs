//! Clippy DAG builders.
//!
//! Builds self-ensuring DAGs for Clippy using the generic CLI tool
//! upsert pattern. These functions create sub-DAG nodes that can be
//! composed into larger workflows.
//!
//! # Fractal DAG Pattern
//!
//! The key insight is that `build_clippy_upsert()` returns a `Node<CliToolOp>`
//! containing a sub-DAG. When the executor encounters this node, it executes
//! the entire sub-DAG (check → install → run) as a unit.

use gunbc_ir::node::Node;
use gunbc_ir::transport::cli::{self, build_cli_upsert, CliToolOp};

/// Build a Clippy upsert sub-DAG node with custom arguments.
///
/// Returns a `Node<CliToolOp>` that can be composed into larger DAGs.
/// The node contains a sub-DAG implementing: check → install → run.
///
/// # Example
///
/// ```ignore
/// let clippy_node = build_clippy_upsert(&["--all-targets"]);
/// // Add to a larger DAG...
/// ```
pub fn build_clippy_upsert(args: &[&str]) -> Node<CliToolOp> {
    build_cli_upsert(&cli::CLIPPY, args)
}

/// Build a Clippy lint-all sub-DAG node with standard flags.
///
/// Uses `--all-targets -- -D warnings` for comprehensive linting.
pub fn build_clippy_lint_all() -> Node<CliToolOp> {
    build_clippy_upsert(&["--all-targets", "--", "-D", "warnings"])
}

/// Build a simple Clippy DAG (single node wrapping the upsert).
///
/// This is useful when you need a standalone `Dag<CliToolOp>` rather
/// than composing into a larger DAG.
pub fn build_clippy_dag(args: &[&str]) -> gunbc_ir::Dag<CliToolOp> {
    let node = build_clippy_upsert(args);
    gunbc_ir::Dag {
        nodes: vec![node],
        edges: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_clippy_upsert_is_subdag() {
        let node = build_clippy_upsert(&["--all-targets"]);
        assert_eq!(node.id.0, "clippy");
        
        // Should be a SubDag, not an Opaque node
        assert!(
            matches!(node.body, NodeBody::SubDag(_)),
            "Expected SubDag, got {:?}",
            node.body
        );
    }

    #[test]
    fn test_clippy_lint_all_has_correct_args() {
        let node = build_clippy_lint_all();
        assert_eq!(node.id.0, "clippy");
    }

    #[test]
    fn test_clippy_dag_structure() {
        let dag = build_clippy_dag(&["--fix"]);
        assert_eq!(dag.nodes.len(), 1);
    }

    #[test]
    fn test_subdag_contains_upsert_nodes() {
        let node = build_clippy_upsert(&[]);
        
        if let NodeBody::SubDag(subdag) = &node.body {
            // Upsert pattern has 3 nodes: check, create, resolve
            assert_eq!(subdag.nodes.len(), 3);
            
            // Verify node IDs follow upsert pattern
            let ids: Vec<&str> = subdag.nodes.iter().map(|n| n.id.0.as_str()).collect();
            assert!(ids.contains(&"check"), "Expected 'check' node, got: {:?}", ids);
            assert!(ids.contains(&"create"), "Expected 'create' node, got: {:?}", ids);
            assert!(ids.contains(&"resolve"), "Expected 'resolve' node, got: {:?}", ids);
        } else {
            panic!("Expected SubDag");
        }
    }
}
