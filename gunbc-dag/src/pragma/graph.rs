//! Graph builder for the pragma tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool follows the content upsert pattern with three independent chains:
//! - clippy.toml: render → read → compare → write
//! - disallowed-methods-allowlist: render → read → compare → write
//! - pragma-lint-policy: render → read → compare → write

use crate::file_ops_graph::FileOpsGraph;
use crate::pragma::ops::PragmaOp;
use gunbc_ir::{
    add_content_upsert_chain, build::*, BuilderError, Cardinality, Dag, DagBuilder, Node,
    WorkflowSignature,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv, PrepareFileReadOp, PrepareFileWriteOp};

/// The operation type for pragma graphs - a union of pragma ops, primitives, and transport.
pub type PragmaGraphOp = FileOpsGraph<PragmaOp>;

/// Get the declared signature for the pragma workflow.
pub fn pragma_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("check_mode", "OptionalBool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        // Outputs from clippy write transport (boundary, skippable)
        .with_output("clippy_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("clippy_written_path", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output("clippy_content", "OptionalString", Cardinality::ZERO_OR_ONE)
        // Outputs from allowlist write transport (boundary, skippable)
        .with_output("allowlist_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("allowlist_written_path", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output("allowlist_content", "OptionalString", Cardinality::ZERO_OR_ONE)
        // Outputs from policy write transport (boundary, skippable)
        .with_output("policy_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("policy_written_path", "OptionalString", Cardinality::ZERO_OR_ONE)
        .with_output("policy_content", "OptionalString", Cardinality::ZERO_OR_ONE)
        // Freshness from compare nodes (terminal boundary outputs)
        .with_output("fresh", "Bool", Cardinality::ONE)
        // Skip from write transports (terminal boundary outputs)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "OptionalString", Cardinality::ZERO_OR_ONE)
}

/// Build the pragma graph using DagBuilder.
///
/// Pipeline (three parallel content upsert chains):
/// ```text
/// render_clippy ──→ prepare_read_clippy → execute_read_clippy → compare_clippy_content → execute_clippy_transport
///                 └→ prepare_write_clippy ─────────────────────────────────────────────→ (request)
///
/// render_allowlist → prepare_read_allowlist → execute_read_allowlist → compare_allowlist_content → execute_allowlist_transport
///                  └→ prepare_write_allowlist ──────────────────────────────────────────────────→ (request)
///
/// render_policy ──→ prepare_read_policy → execute_read_policy → compare_policy_content → execute_policy_transport
///                 └→ prepare_write_policy ──────────────────────────────────────────────→ (request)
/// ```
pub fn build_pragma_graph() -> Result<Dag<PragmaGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs:write", "FilesystemHandle")],
        PragmaGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
    ))?;

    // Clippy upsert chain
    let render_clippy = builder.add_root_node(Node::opaque(
        "render_clippy",
        vec![],
        vec![port("content", "String")],
        PragmaGraphOp::Domain(PragmaOp::RenderClippy),
    ))?;

    let clippy_read = resource("fs:clippy.toml", "FilesystemHandle", AccessMode::Read);
    let clippy_write = resource("fs:clippy.toml", "FilesystemHandle", AccessMode::Write);
    let clippy_chain = add_content_upsert_chain(
        &mut builder,
        "clippy",
        &render_clippy,
        "content",
        vec![clippy_read],
        vec![clippy_write],
        PragmaGraphOp::PrepareFileRead(PrepareFileReadOp),
        PragmaGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        PragmaGraphOp::Blob(BlobOps::CompareContent),
        PragmaGraphOp::Transport(TransportOps::Execute),
    )?;

    // Allowlist upsert chain
    let render_allowlist = builder.add_root_node(Node::opaque(
        "render_allowlist",
        vec![],
        vec![port("content", "String")],
        PragmaGraphOp::Domain(PragmaOp::RenderAllowlist),
    ))?;

    let allowlist_read =
        resource("fs:tools/disallowed-methods-allowlist.txt", "FilesystemHandle", AccessMode::Read);
    let allowlist_write =
        resource("fs:tools/disallowed-methods-allowlist.txt", "FilesystemHandle", AccessMode::Write);
    let allowlist_chain = add_content_upsert_chain(
        &mut builder,
        "allowlist",
        &render_allowlist,
        "content",
        vec![allowlist_read],
        vec![allowlist_write],
        PragmaGraphOp::PrepareFileRead(PrepareFileReadOp),
        PragmaGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        PragmaGraphOp::Blob(BlobOps::CompareContent),
        PragmaGraphOp::Transport(TransportOps::Execute),
    )?;

    // Policy upsert chain
    let render_policy = builder.add_root_node(Node::opaque(
        "render_policy",
        vec![],
        vec![port("content", "String")],
        PragmaGraphOp::Domain(PragmaOp::RenderLintPolicy),
    ))?;

    let policy_read =
        resource("fs:tools/pragma-lint-policy.txt", "FilesystemHandle", AccessMode::Read);
    let policy_write =
        resource("fs:tools/pragma-lint-policy.txt", "FilesystemHandle", AccessMode::Write);
    let policy_chain = add_content_upsert_chain(
        &mut builder,
        "policy",
        &render_policy,
        "content",
        vec![policy_read],
        vec![policy_write],
        PragmaGraphOp::PrepareFileRead(PrepareFileReadOp),
        PragmaGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        PragmaGraphOp::Blob(BlobOps::CompareContent),
        PragmaGraphOp::Transport(TransportOps::Execute),
    )?;

    // Resource wiring
    builder.add_edge(
        fs_env.out("fs:write"),
        clippy_chain.execute_read.in_port("res:fs:clippy.toml"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        clippy_chain.execute_write.in_port("res:fs:clippy.toml"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        allowlist_chain
            .execute_read
            .in_port("res:fs:tools/disallowed-methods-allowlist.txt"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        allowlist_chain
            .execute_write
            .in_port("res:fs:tools/disallowed-methods-allowlist.txt"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        policy_chain
            .execute_read
            .in_port("res:fs:tools/pragma-lint-policy.txt"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        policy_chain
            .execute_write
            .in_port("res:fs:tools/pragma-lint-policy.txt"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

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
        assert!(!boundaries.is_boundary_node(&"prepare_write_clippy".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_write_allowlist".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_write_policy".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_clippy".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_allowlist".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_policy".into()));
        // Render nodes are NOT boundaries
        assert!(!boundaries.is_boundary_node(&"render_clippy".into()));
        assert!(!boundaries.is_boundary_node(&"render_allowlist".into()));
        assert!(!boundaries.is_boundary_node(&"render_policy".into()));
    }

    // Signature validation tests are generated by testgen (via graph_mock).
}
