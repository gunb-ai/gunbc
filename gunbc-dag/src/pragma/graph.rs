//! Graph builder for the pragma tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool follows the content upsert pattern with three independent chains:
//! - clippy.toml: render → read → compare → write
//! - disallowed-methods-allowlist: render → read → compare → write
//! - pragma-lint-policy: render → read → compare → write

use crate::pragma::ops::PragmaOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{PrepareFileReadOp, PrepareFileWriteOp};
use std::collections::HashMap;

/// The operation type for pragma graphs - a union of pragma ops, primitives, and transport.
#[derive(Debug, Clone)]
pub enum PragmaGraphOp {
    /// Pragma-specific operations
    Pragma(PragmaOp),
    /// Prepare file read (primitive - PURE)
    PrepareFileRead(PrepareFileReadOp),
    /// Prepare file write (primitive - PURE)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Blob operations (compare content - PURE)
    Blob(BlobOps),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for PragmaGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            PragmaGraphOp::Pragma(op) => op.execute(inputs),
            PragmaGraphOp::PrepareFileRead(op) => op.execute(inputs),
            PragmaGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            PragmaGraphOp::Blob(op) => op.execute(inputs),
            PragmaGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the pragma workflow.
pub fn pragma_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("check_mode", "Bool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        // Outputs from clippy write transport (boundary, skippable)
        .with_output("clippy_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("clippy_written_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("clippy_content", "String", Cardinality::ZERO_OR_ONE)
        // Outputs from allowlist write transport (boundary, skippable)
        .with_output("allowlist_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("allowlist_written_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("allowlist_content", "String", Cardinality::ZERO_OR_ONE)
        // Outputs from policy write transport (boundary, skippable)
        .with_output("policy_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("policy_written_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("policy_content", "String", Cardinality::ZERO_OR_ONE)
        // Freshness from compare nodes (terminal boundary outputs)
        .with_output("fresh", "Bool", Cardinality::ONE)
        // Skip from write transports (terminal boundary outputs)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "String", Cardinality::ZERO_OR_ONE)
}

/// Build the pragma graph using DagBuilder.
///
/// Pipeline (three parallel content upsert chains):
/// ```text
/// render_clippy ──→ prepare_read_clippy → execute_read_clippy → compare_clippy_content → execute_clippy_transport
///                 └→ prepare_clippy_write ─────────────────────────────────────────────→ (request)
///
/// render_allowlist → prepare_read_allowlist → execute_read_allowlist → compare_allowlist_content → execute_allowlist_transport
///                  └→ prepare_allowlist_write ──────────────────────────────────────────────────→ (request)
///
/// render_policy ──→ prepare_read_policy → execute_read_policy → compare_policy_content → execute_policy_transport
///                 └→ prepare_policy_write ──────────────────────────────────────────────→ (request)
/// ```
pub fn build_pragma_graph() -> Result<Dag<PragmaGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Clippy upsert chain
    // ========================================================================

    // Render
    let render_clippy = builder.add_root_node(Node::opaque(
        "render_clippy",
        vec![],
        vec![port("content", "String")],
        PragmaGraphOp::Pragma(PragmaOp::RenderClippy),
    ))?;

    // Read chain
    let prepare_read_clippy = builder.add_node_after(
        Node::opaque(
            "prepare_read_clippy",
            vec![port("path", "String")],
            vec![port("request", "TransportRequest")],
            PragmaGraphOp::PrepareFileRead(PrepareFileReadOp),
        ),
        &render_clippy,
    )?;

    let execute_read_clippy = builder.add_node_after(
        Node::opaque(
            "execute_read_clippy",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            PragmaGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_read_clippy,
    )?;

    // Compare
    let compare_clippy_content = builder.add_node_after(
        Node::opaque(
            "compare_clippy_content",
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
            PragmaGraphOp::Blob(BlobOps::CompareContent),
        ),
        &execute_read_clippy,
    )?;

    // Write chain
    let prepare_clippy_write = builder.add_node_after(
        Node::opaque(
            "prepare_clippy_write",
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest")],
            PragmaGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &render_clippy,
    )?;

    let execute_clippy_transport = builder.add_node_after(
        Node::opaque(
            "execute_clippy_transport",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional("clippy_response", "TransportResponse"),
                optional("clippy_written_path", "String"),
                optional("clippy_content", "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            PragmaGraphOp::Transport(TransportOps::Execute),
        ),
        &compare_clippy_content,
    )?;

    // ========================================================================
    // Allowlist upsert chain
    // ========================================================================

    // Render
    let render_allowlist = builder.add_root_node(Node::opaque(
        "render_allowlist",
        vec![],
        vec![port("content", "String")],
        PragmaGraphOp::Pragma(PragmaOp::RenderAllowlist),
    ))?;

    // Read chain
    let prepare_read_allowlist = builder.add_node_after(
        Node::opaque(
            "prepare_read_allowlist",
            vec![port("path", "String")],
            vec![port("request", "TransportRequest")],
            PragmaGraphOp::PrepareFileRead(PrepareFileReadOp),
        ),
        &render_allowlist,
    )?;

    let execute_read_allowlist = builder.add_node_after(
        Node::opaque(
            "execute_read_allowlist",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            PragmaGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_read_allowlist,
    )?;

    // Compare
    let compare_allowlist_content = builder.add_node_after(
        Node::opaque(
            "compare_allowlist_content",
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
            PragmaGraphOp::Blob(BlobOps::CompareContent),
        ),
        &execute_read_allowlist,
    )?;

    // Write chain
    let prepare_allowlist_write = builder.add_node_after(
        Node::opaque(
            "prepare_allowlist_write",
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest")],
            PragmaGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &render_allowlist,
    )?;

    let execute_allowlist_transport = builder.add_node_after(
        Node::opaque(
            "execute_allowlist_transport",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional("allowlist_response", "TransportResponse"),
                optional("allowlist_written_path", "String"),
                optional("allowlist_content", "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            PragmaGraphOp::Transport(TransportOps::Execute),
        ),
        &compare_allowlist_content,
    )?;

    // ========================================================================
    // Policy upsert chain
    // ========================================================================

    // Render
    let render_policy = builder.add_root_node(Node::opaque(
        "render_policy",
        vec![],
        vec![port("content", "String")],
        PragmaGraphOp::Pragma(PragmaOp::RenderLintPolicy),
    ))?;

    // Read chain
    let prepare_read_policy = builder.add_node_after(
        Node::opaque(
            "prepare_read_policy",
            vec![port("path", "String")],
            vec![port("request", "TransportRequest")],
            PragmaGraphOp::PrepareFileRead(PrepareFileReadOp),
        ),
        &render_policy,
    )?;

    let execute_read_policy = builder.add_node_after(
        Node::opaque(
            "execute_read_policy",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            PragmaGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_read_policy,
    )?;

    // Compare
    let compare_policy_content = builder.add_node_after(
        Node::opaque(
            "compare_policy_content",
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
            PragmaGraphOp::Blob(BlobOps::CompareContent),
        ),
        &execute_read_policy,
    )?;

    // Write chain
    let prepare_policy_write = builder.add_node_after(
        Node::opaque(
            "prepare_policy_write",
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest")],
            PragmaGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &render_policy,
    )?;

    let execute_policy_transport = builder.add_node_after(
        Node::opaque(
            "execute_policy_transport",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional("policy_response", "TransportResponse"),
                optional("policy_written_path", "String"),
                optional("policy_content", "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            PragmaGraphOp::Transport(TransportOps::Execute),
        ),
        &compare_policy_content,
    )?;

    // ========================================================================
    // Wire up the Clippy upsert chain
    // ========================================================================

    // Render content -> Compare expected_content
    builder.add_edge(
        render_clippy.out("content"),
        compare_clippy_content.in_port("expected_content"),
    )?;

    // Render content -> PrepareWrite content
    builder.add_edge(
        render_clippy.out("content"),
        prepare_clippy_write.in_port("content"),
    )?;

    // PrepareRead -> ExecuteRead
    builder.add_edge(
        prepare_read_clippy.out("request"),
        execute_read_clippy.in_port("request"),
    )?;

    // ExecuteRead -> Compare
    builder.add_edge(
        execute_read_clippy.out("response"),
        compare_clippy_content.in_port("response"),
    )?;

    // Compare skip -> ExecuteWrite skip
    builder.add_edge(
        compare_clippy_content.out("skip"),
        execute_clippy_transport.in_port("skip"),
    )?;

    // Compare skip_reason -> ExecuteWrite skip_reason
    builder.add_edge(
        compare_clippy_content.out("skip_reason"),
        execute_clippy_transport.in_port("skip_reason"),
    )?;

    // PrepareWrite -> ExecuteWrite
    builder.add_edge(
        prepare_clippy_write.out("request"),
        execute_clippy_transport.in_port("request"),
    )?;

    // ========================================================================
    // Wire up the Allowlist upsert chain
    // ========================================================================

    // Render content -> Compare expected_content
    builder.add_edge(
        render_allowlist.out("content"),
        compare_allowlist_content.in_port("expected_content"),
    )?;

    // Render content -> PrepareWrite content
    builder.add_edge(
        render_allowlist.out("content"),
        prepare_allowlist_write.in_port("content"),
    )?;

    // PrepareRead -> ExecuteRead
    builder.add_edge(
        prepare_read_allowlist.out("request"),
        execute_read_allowlist.in_port("request"),
    )?;

    // ExecuteRead -> Compare
    builder.add_edge(
        execute_read_allowlist.out("response"),
        compare_allowlist_content.in_port("response"),
    )?;

    // Compare skip -> ExecuteWrite skip
    builder.add_edge(
        compare_allowlist_content.out("skip"),
        execute_allowlist_transport.in_port("skip"),
    )?;

    // Compare skip_reason -> ExecuteWrite skip_reason
    builder.add_edge(
        compare_allowlist_content.out("skip_reason"),
        execute_allowlist_transport.in_port("skip_reason"),
    )?;

    // PrepareWrite -> ExecuteWrite
    builder.add_edge(
        prepare_allowlist_write.out("request"),
        execute_allowlist_transport.in_port("request"),
    )?;

    // ========================================================================
    // Wire up the Policy upsert chain
    // ========================================================================

    // Render content -> Compare expected_content
    builder.add_edge(
        render_policy.out("content"),
        compare_policy_content.in_port("expected_content"),
    )?;

    // Render content -> PrepareWrite content
    builder.add_edge(
        render_policy.out("content"),
        prepare_policy_write.in_port("content"),
    )?;

    // PrepareRead -> ExecuteRead
    builder.add_edge(
        prepare_read_policy.out("request"),
        execute_read_policy.in_port("request"),
    )?;

    // ExecuteRead -> Compare
    builder.add_edge(
        execute_read_policy.out("response"),
        compare_policy_content.in_port("response"),
    )?;

    // Compare skip -> ExecuteWrite skip
    builder.add_edge(
        compare_policy_content.out("skip"),
        execute_policy_transport.in_port("skip"),
    )?;

    // Compare skip_reason -> ExecuteWrite skip_reason
    builder.add_edge(
        compare_policy_content.out("skip_reason"),
        execute_policy_transport.in_port("skip_reason"),
    )?;

    // PrepareWrite -> ExecuteWrite
    builder.add_edge(
        prepare_policy_write.out("request"),
        execute_policy_transport.in_port("request"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_pragma_graph().expect("graph should build");
        // 18 nodes: 3 chains x 6 nodes (render, prepare_read, execute_read, compare, prepare_write, execute_write)
        assert_eq!(dag.nodes.len(), 18);
        // 21 edges: 3 chains x 7 edges each
        assert_eq!(dag.edges.len(), 21);
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_pragma_graph().expect("graph should build");

        // Verify transport nodes exist
        assert!(dag.get_node(&"execute_read_clippy".into()).is_some());
        assert!(dag.get_node(&"execute_clippy_transport".into()).is_some());
        assert!(dag.get_node(&"execute_read_allowlist".into()).is_some());
        assert!(dag.get_node(&"execute_allowlist_transport".into()).is_some());
        assert!(dag.get_node(&"execute_read_policy".into()).is_some());
        assert!(dag.get_node(&"execute_policy_transport".into()).is_some());
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_pragma_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // check_mode entrypoints on each compare node
        assert!(entrypoints.is_entrypoint_port(&"compare_clippy_content".into(), &"check_mode".into()));
        assert!(entrypoints.is_entrypoint_port(&"compare_allowlist_content".into(), &"check_mode".into()));
        assert!(entrypoints.is_entrypoint_port(&"compare_policy_content".into(), &"check_mode".into()));
        // path entrypoints on each read prepare node
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_clippy".into(), &"path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_allowlist".into(), &"path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_policy".into(), &"path".into()));
    }

    #[test]
    fn test_pure_nodes_not_boundaries() {
        let dag = build_pragma_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Prepare and compare nodes are NOT boundaries
        assert!(!boundaries.is_boundary_node(&"prepare_clippy_write".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_allowlist_write".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_policy_write".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_clippy".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_allowlist".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_policy".into()));
        // Render nodes are NOT boundaries
        assert!(!boundaries.is_boundary_node(&"render_clippy".into()));
        assert!(!boundaries.is_boundary_node(&"render_allowlist".into()));
        assert!(!boundaries.is_boundary_node(&"render_policy".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_pragma_graph().expect("graph should build");
        let sig = pragma_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_pragma_graph().expect("graph should build");
        let inferred = infer_signature(&dag);

        // 9 inputs: 3 read paths, 3 write paths, 3 check_modes
        assert_eq!(inferred.inputs.len(), 9);
    }
}
