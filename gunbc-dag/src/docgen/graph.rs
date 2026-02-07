//! Graph builder for doc generation.
//!
//! Generates documentation artifacts from live code and test sources.

use crate::docgen::ops::DocgenOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    add_content_upsert_chain,
    build::*,
    BuilderError, Dag, DagBuilder, Node, Value,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{PrepareFileReadOp, PrepareFileWriteOp};
use std::collections::HashMap;

/// Union type for docgen graph operations.
#[derive(Debug, Clone)]
pub enum DocgenGraphOp {
    /// Docgen-specific pure operations.
    Docgen(DocgenOp),
    /// Prepare file read (pure).
    PrepareFileRead(PrepareFileReadOp),
    /// Prepare file write (pure).
    PrepareFileWrite(PrepareFileWriteOp),
    /// Blob operations (compare content - pure).
    Blob(BlobOps),
    /// Transport operations (boundary - actual I/O).
    Transport(TransportOps),
}

impl Executable for DocgenGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DocgenGraphOp::Docgen(op) => op.execute(inputs),
            DocgenGraphOp::PrepareFileRead(op) => op.execute(inputs),
            DocgenGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            DocgenGraphOp::Blob(op) => op.execute(inputs),
            DocgenGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build the docgen graph.
///
/// One content-upsert chain:
/// - docs/ab-writing-workflows.md (handwritten template + generated sections)
pub fn build_docgen_graph() -> Result<Dag<DocgenGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Generate main doc (with generated sections)
    let render_ab_doc = builder.add_root_node(Node::opaque(
        "render_ab_workflows_doc",
        vec![],
        vec![scalar("content", "String"), scalar("path", "String")],
        DocgenGraphOp::Docgen(DocgenOp::RenderAbWorkflowsDoc),
    ))?;

    let chain_ab_doc = add_content_upsert_chain(
        &mut builder,
        "ab_workflows_doc",
        &render_ab_doc,
        "content",
        DocgenGraphOp::PrepareFileRead(PrepareFileReadOp),
        DocgenGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        DocgenGraphOp::Blob(BlobOps::CompareContent),
        DocgenGraphOp::Transport(TransportOps::Execute),
    )?;

    builder.add_edge(
        render_ab_doc.out("path"),
        chain_ab_doc.prepare_read.in_port("path"),
    )?;
    builder.add_edge(
        render_ab_doc.out("path"),
        chain_ab_doc.prepare_write.in_port("path"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::detect_boundaries;

    #[test]
    fn test_graph_builds() {
        let dag = build_docgen_graph().expect("graph should build");
        // 1 render node + 1 content upsert chain (5 nodes) = 6 nodes
        assert_eq!(dag.nodes.len(), 6);
        // One chain wires 8 internal edges + 2 path edges
        assert_eq!(dag.edges.len(), 10);
    }

    #[test]
    fn test_transport_boundaries_present() {
        let dag = build_docgen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);
        assert!(boundaries.is_boundary_node(&"execute_ab_workflows_doc_transport".into()));
    }
}
