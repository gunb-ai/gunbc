//! Build SubDag builder.
//!
//! Wraps the build tool as a SubDag node using WorkspaceOp.

use crate::build::{build_build_graph, BuildGraphOp};
use crate::workspace::convert::convert_dag;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

fn convert_build_op(op: BuildGraphOp) -> WorkspaceOp {
    match op {
        BuildGraphOp::Build(build_op) => WorkspaceOp::Build(build_op),
        BuildGraphOp::FsEnv(env) => WorkspaceOp::FsEnv(env),
        BuildGraphOp::Transport(transport) => WorkspaceOp::Transport(transport),
    }
}

/// Build the build SubDag node.
pub fn build_build_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let original = build_build_graph()?;
    let converted_dag = convert_dag(original, &convert_build_op);
    Ok(Node::subdag("build", converted_dag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_build_subdag_is_subdag() {
        let node = build_build_subdag().expect("build subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "build");
    }

    #[test]
    fn test_build_subdag_has_core_nodes() {
        let node = build_build_subdag().expect("build subdag should build");
        match &node.body {
            NodeBody::SubDag(dag) => {
                assert!(dag.get_node(&"build".into()).is_some());
                assert!(dag.get_node(&"test".into()).is_some());
                assert!(dag.get_node(&"clippy".into()).is_some());
                assert!(dag.get_node(&"summary".into()).is_some());
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
