//! Graph builder for the makegen tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool follows the content upsert pattern:
//! - Generate content (pure)
//! - Read existing file (transport boundary)
//! - Compare content (pure) — check phase of upsert
//! - Write file if stale (transport boundary, skippable)

use crate::file_ops_graph::FileOpsGraph;
use crate::makegen::ops::MakegenOp;
use gunbc_ir::{
    add_content_upsert_chain, build::*, BuilderError, Cardinality, Dag, DagBuilder, Node,
    WorkflowSignature,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{PrepareFileReadOp, PrepareFileWriteOp};

/// The operation type for makegen graphs - a union of makegen ops, primitives, and transport.
pub type MakegenGraphOp = FileOpsGraph<MakegenOp>;

/// Get the declared signature for the makegen workflow.
pub fn makegen_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("check_mode", "Bool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        // Outputs from execute_makegen_transport (boundary)
        .with_output("makegen_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("makegen_written_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("makegen_content", "String", Cardinality::ZERO_OR_ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "String", Cardinality::ZERO_OR_ONE)
        // Outputs from compare_makegen_content (freshness)
        .with_output("fresh", "Bool", Cardinality::ONE)
        // Informational outputs from load_registry (secondary boundaries)
        .with_output("tool_count", "Int", Cardinality::ONE)
        .with_output("tool_names", "String", Cardinality::ONE_OR_MORE)
}

/// Build the makegen graph using DagBuilder.
///
/// Pipeline (follows content upsert pattern):
/// ```text
/// LoadRegistry -> RenderMakefile ─┬─→ PrepareFileRead -> ExecuteRead -> CompareContent -> ExecuteWrite
///                                 └─→ PrepareFileWrite ──────────────────────────────────→ (request)
/// ```
///
/// Key wiring:
/// - render_makefile.makefile_content → compare_content.expected_content (for comparison)
/// - render_makefile.makefile_content → prepare_file_write.content (for write request)
/// - execute_read.response → compare_content.response
/// - compare_content.skip → execute_write.skip
/// - compare_content.skip_reason → execute_write.skip_reason
/// - prepare_file_write.request → execute_write.request
/// - path entrypoint → prepare_file_read.path AND prepare_file_write.path
/// - check_mode entrypoint → compare_content.check_mode
pub fn build_makegen_graph() -> Result<Dag<MakegenGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: LoadRegistry (makegen-specific) - generation 0
    let load_registry = builder.add_root_node(Node::opaque(
        "load_registry",
        vec![],
        vec![
            scalar("tool_count", "Int"),
            non_empty_list("tool_names", "String"),
            scalar("registry", "Json"),
        ],
        MakegenGraphOp::Domain(MakegenOp::LoadRegistry),
    ))?;

    // Node: RenderMakefile (makegen-specific) - generation 1
    let render_makefile = builder.add_node_after(
        Node::opaque(
            "render_makefile",
            vec![scalar("registry", "Json")],
            vec![scalar("makefile_content", "String")],
            MakegenGraphOp::Domain(MakegenOp::RenderMakefile),
        ),
        &load_registry,
    )?;

    // LoadRegistry -> RenderMakefile
    builder.add_edge(
        load_registry.out("registry"),
        render_makefile.in_port("registry"),
    )?;

    // Content upsert chain
    add_content_upsert_chain(
        &mut builder,
        "makegen",
        &render_makefile,
        "makefile_content",
        MakegenGraphOp::PrepareFileRead(PrepareFileReadOp),
        MakegenGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        MakegenGraphOp::Blob(BlobOps::CompareContent),
        MakegenGraphOp::Transport(TransportOps::Execute),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_makegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // ExecuteWrite is a boundary (terminal transport node with unconnected outputs)
        assert!(boundaries.is_boundary_node(&"execute_makegen_transport".into()));
        // CompareContent is a boundary (fresh output is terminal)
        assert!(boundaries.is_boundary_node(&"compare_makegen_content".into()));
        // ExecuteRead is NOT a boundary (its response output is connected to compare)
        assert!(!boundaries.is_boundary_node(&"execute_read_makegen".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_makegen_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // path is an entrypoint (input to prepare_write_makegen with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"prepare_write_makegen".into(), &"path".into()));
        // path is an entrypoint (input to prepare_read_makegen with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_makegen".into(), &"path".into()));
        // check_mode is an entrypoint (input to compare_makegen_content with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"compare_makegen_content".into(), &"check_mode".into()));
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_makegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Pure prepare nodes are NOT boundaries (all outputs connected)
        assert!(!boundaries.is_boundary_node(&"prepare_write_makegen".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_makegen".into()));
        assert!(!boundaries.is_boundary_node(&"render_makefile".into()));
    }

    // Signature validation tests are generated by testgen (via graph_mock).
}
