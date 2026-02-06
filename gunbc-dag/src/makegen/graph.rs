//! Graph builder for the makegen tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool follows the content upsert pattern:
//! - Generate content (pure)
//! - Read existing file (transport boundary)
//! - Compare content (pure) — check phase of upsert
//! - Write file if stale (transport boundary, skippable)

use crate::makegen::ops::MakegenOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{CompareContentOp, PrepareFileReadOp, PrepareFileWriteOp};
use std::collections::HashMap;

/// The operation type for makegen graphs - a union of makegen ops, primitives, and transport.
#[derive(Debug, Clone)]
pub enum MakegenGraphOp {
    /// Makegen-specific operations
    Makegen(MakegenOp),
    /// Prepare file read (primitive - PURE)
    PrepareFileRead(PrepareFileReadOp),
    /// Prepare file write (primitive - PURE)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Compare content (primitive - PURE)
    CompareContent(CompareContentOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for MakegenGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            MakegenGraphOp::Makegen(op) => op.execute(inputs),
            MakegenGraphOp::PrepareFileRead(op) => op.execute(inputs),
            MakegenGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            MakegenGraphOp::CompareContent(op) => op.execute(inputs),
            MakegenGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the makegen workflow.
pub fn makegen_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("output_path", "String", Cardinality::ZERO_OR_ONE)
        .with_input("check_mode", "Bool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        // Outputs from execute_write (boundary)
        .with_output("response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("written_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("content", "String", Cardinality::ZERO_OR_ONE)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "String", Cardinality::ZERO_OR_ONE)
        // Outputs from compare_content (freshness)
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
/// - output_path entrypoint → prepare_file_read.path AND prepare_file_write.output_path
/// - check_mode entrypoint → compare_content.check_mode
#[allow(clippy::result_large_err)]
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
        MakegenGraphOp::Makegen(MakegenOp::LoadRegistry),
    ))?;

    // Node: RenderMakefile (makegen-specific) - generation 1
    let render_makefile = builder.add_node_after(
        Node::opaque(
            "render_makefile",
            vec![scalar("registry", "Json")],
            vec![scalar("makefile_content", "String")],
            MakegenGraphOp::Makegen(MakegenOp::RenderMakefile),
        ),
        &load_registry,
    )?;

    // === Read chain: PrepareFileRead -> ExecuteRead ===

    // Node: PrepareFileRead (primitive - PURE)
    let prepare_file_read = builder.add_node_after(
        Node::opaque(
            "prepare_file_read",
            vec![port("path", "String")],
            vec![port("request", "TransportRequest")],
            MakegenGraphOp::PrepareFileRead(PrepareFileReadOp),
        ),
        &render_makefile,
    )?;

    // Node: ExecuteRead (transport - BOUNDARY)
    let execute_read = builder.add_node_after(
        Node::opaque(
            "execute_read",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            MakegenGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_file_read,
    )?;

    // === Compare content (PURE) ===

    // Node: CompareContent - takes read response + expected content, outputs skip signal
    let compare_content = builder.add_node_after(
        Node::opaque(
            "compare_content",
            vec![
                port("response", "TransportResponse"),
                port("expected_content", "String"),
                optional("check_mode", "Bool"),
            ],
            vec![
                port("fresh", "Bool"),
                port("skip", "Bool"),
                port("skip_reason", "String"),
            ],
            MakegenGraphOp::CompareContent(CompareContentOp),
        ),
        &execute_read,
    )?;

    // === Write chain: PrepareFileWrite -> ExecuteWrite ===

    // Node: PrepareFileWrite (primitive - PURE)
    let prepare_file_write = builder.add_node_after(
        Node::opaque(
            "prepare_file_write",
            vec![port("content", "String"), optional("output_path", "String")],
            vec![port("request", "TransportRequest")],
            MakegenGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &render_makefile,
    )?;

    // Node: ExecuteWrite (transport - BOUNDARY, skippable)
    let execute_write = builder.add_node_after(
        Node::opaque(
            "execute_write",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional("response", "TransportResponse"),
                optional("written_path", "String"),
                optional("content", "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            MakegenGraphOp::Transport(TransportOps::Execute),
        ),
        &compare_content,
    )?;

    // === Wire up the pipeline ===

    // LoadRegistry -> RenderMakefile
    builder.add_edge(
        load_registry.out("registry"),
        render_makefile.in_port("registry"),
    )?;

    // RenderMakefile -> PrepareFileRead (via expected_content to compare_content)
    // RenderMakefile content -> CompareContent expected_content
    builder.add_edge(
        render_makefile.out("makefile_content"),
        compare_content.in_port("expected_content"),
    )?;

    // RenderMakefile content -> PrepareFileWrite content
    builder.add_edge(
        render_makefile.out("makefile_content"),
        prepare_file_write.in_port("content"),
    )?;

    // PrepareFileRead -> ExecuteRead
    builder.add_edge(
        prepare_file_read.out("request"),
        execute_read.in_port("request"),
    )?;

    // ExecuteRead -> CompareContent
    builder.add_edge(
        execute_read.out("response"),
        compare_content.in_port("response"),
    )?;

    // CompareContent skip -> ExecuteWrite skip
    builder.add_edge(
        compare_content.out("skip"),
        execute_write.in_port("skip"),
    )?;

    // CompareContent skip_reason -> ExecuteWrite skip_reason
    builder.add_edge(
        compare_content.out("skip_reason"),
        execute_write.in_port("skip_reason"),
    )?;

    // PrepareFileWrite -> ExecuteWrite
    builder.add_edge(
        prepare_file_write.out("request"),
        execute_write.in_port("request"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_makegen_graph().expect("graph should build");
        // 7 nodes: LoadRegistry, RenderMakefile, PrepareFileRead, ExecuteRead,
        //          CompareContent, PrepareFileWrite, ExecuteWrite
        assert_eq!(dag.nodes.len(), 7);
        // 8 edges: registry->render, content->compare, content->prepare_write,
        //          request->execute_read, response->compare, skip->execute_write,
        //          skip_reason->execute_write, request->execute_write
        assert_eq!(dag.edges.len(), 8);
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_makegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // ExecuteWrite is a boundary (terminal transport node with unconnected outputs)
        assert!(boundaries.is_boundary_node(&"execute_write".into()));
        // CompareContent is a boundary (fresh output is terminal)
        assert!(boundaries.is_boundary_node(&"compare_content".into()));
        // ExecuteRead is NOT a boundary (its response output is connected to compare_content)
        assert!(!boundaries.is_boundary_node(&"execute_read".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_makegen_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // output_path is an entrypoint (input to prepare_file_write with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"prepare_file_write".into(), &"output_path".into()));
        // path is an entrypoint (input to prepare_file_read with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"prepare_file_read".into(), &"path".into()));
        // check_mode is an entrypoint (input to compare_content with no upstream)
        assert!(entrypoints.is_entrypoint_port(&"compare_content".into(), &"check_mode".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_makegen_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 7);
        assert_eq!(dag.edges.len(), 8);
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_makegen_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Pure prepare nodes are NOT boundaries (all outputs connected)
        assert!(!boundaries.is_boundary_node(&"prepare_file_write".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_file_read".into()));
        assert!(!boundaries.is_boundary_node(&"render_makefile".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_makegen_graph().expect("graph should build");
        let sig = makegen_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_makegen_graph().expect("graph should build");
        let inferred = infer_signature(&dag);

        // 3 inputs: output_path (prepare_file_write), path (prepare_file_read), check_mode (compare_content)
        assert_eq!(inferred.inputs.len(), 3);
    }
}
