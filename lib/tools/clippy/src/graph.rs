//! Clippy DAG builder.
//!
//! Builds a self-ensuring DAG for clippy that:
//! 1. Checks if clippy is installed
//! 2. Installs clippy if needed (via rustup)
//! 3. Runs cargo clippy with the given arguments
//!
//! This DAG can be composed into other workflows (like CI Lint)
//! and the dependency on clippy is implicit through usage.

use crate::ops::ClippyOp;
use gunbc_ir::patterns::UpsertBuilder;
use gunbc_ir::{Dag, Node};

/// Build a Clippy DAG that ensures clippy is available and runs it.
///
/// The DAG follows the upsert pattern:
/// - **Check**: Verify clippy is installed (`cargo clippy --version`)
/// - **Create**: Install clippy if missing (`rustup component add clippy`)
/// - **Resolve**: Run clippy with the provided arguments
///
/// # Arguments
///
/// * `args` - Arguments to pass to `cargo clippy`
///
/// # Returns
///
/// A `Node<ClippyOp>` containing the upsert sub-dag. This node can be
/// added to any DAG that needs clippy linting.
///
/// # Example
///
/// ```ignore
/// // Create a clippy dag with standard lint args
/// let clippy_node = build_clippy_upsert(&["--all-targets", "--", "-D", "warnings"]);
///
/// // Add to a larger workflow
/// builder.add_node(clippy_node);
/// ```
pub fn build_clippy_upsert(args: &[&str]) -> Node<ClippyOp> {
    let run_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    
    UpsertBuilder::new("clippy")
        .with_check(ClippyOp::CheckInstalled)
        .with_create(ClippyOp::Install)
        .with_resolve(ClippyOp::Run { args: run_args })
        .with_input_port("trigger", "Unit")  // No real input needed
        .with_output_port("lint_result", "LintResult")
        .build()
}

/// Build a full Clippy DAG for standalone execution.
///
/// This creates a complete DAG that can be executed directly,
/// not just a node for composition.
pub fn build_clippy_dag(args: &[&str]) -> Dag<ClippyOp> {
    let mut dag = Dag::new();
    dag.add_node(build_clippy_upsert(args));
    dag
}

/// Build a Clippy DAG with default lint-all arguments.
///
/// Uses `--all-targets -- -D warnings` for comprehensive linting.
pub fn build_clippy_lint_all() -> Node<ClippyOp> {
    build_clippy_upsert(&["--all-targets", "--", "-D", "warnings"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::node::NodeBody;

    #[test]
    fn test_clippy_upsert_structure() {
        let node = build_clippy_upsert(&["--all-targets"]);
        
        assert_eq!(node.id.0, "clippy");
        assert!(node.is_subdag());
        
        // Should have upsert structure
        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 3); // check, create, resolve
                
                let names: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(names.contains(&"check"));
                assert!(names.contains(&"create"));
                assert!(names.contains(&"resolve"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_clippy_dag_complete() {
        let dag = build_clippy_dag(&["--all-targets"]);
        
        // Should have one top-level node (the upsert)
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.nodes[0].id.0, "clippy");
    }

    #[test]
    fn test_clippy_lint_all() {
        let node = build_clippy_lint_all();
        
        assert_eq!(node.id.0, "clippy");
        
        // Verify it has the expected resolve operation with default args
        match &node.body {
            NodeBody::SubDag(dag) => {
                let resolve = dag.get_node(&"resolve".into()).unwrap();
                match &resolve.body {
                    NodeBody::Opaque(ClippyOp::Run { args }) => {
                        assert!(args.contains(&"--all-targets".to_string()));
                        assert!(args.contains(&"-D".to_string()));
                        assert!(args.contains(&"warnings".to_string()));
                    }
                    _ => panic!("Expected Run operation"),
                }
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
