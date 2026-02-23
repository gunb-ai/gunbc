//! Makegen SubDag builder.
//!
//! Wraps the makegen tool as a SubDag node using WorkspaceOp.

use crate::dsl_builder::build_makegen_graph_dsl;
use crate::workspace::WorkspaceOp;
use gunbc_ir::Node;

/// Build the makegen SubDag node.
pub fn build_makegen_subdag() -> Node<WorkspaceOp> {
    let dsl_dag = build_makegen_graph_dsl().expect("makegen DSL graph should build");
    Node::subdag("makegen", dsl_dag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_makegen_subdag_is_subdag() {
        let node = build_makegen_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "makegen");
    }
}
