//! SDLC SubDag builder.
//!
//! Wraps the SDLC pipeline as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_sdlc_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::{BuilderError, Node};

/// Build the SDLC SubDag node.
pub fn build_sdlc_subdag() -> Result<Node<WorkspaceOp>, BuilderError> {
    let dsl_dag = build_sdlc_graph_dsl()?;
    Ok(Node::subdag("sdlc", dsl_dag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdlc_subdag_is_subdag() {
        let node = build_sdlc_subdag().expect("sdlc subdag should build");
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "sdlc");
    }
}
