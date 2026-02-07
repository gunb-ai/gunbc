//! Graph builder for the bootstrap tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool follows the content upsert pattern:
//! - Pure ops prepare data and `TransportRequest` values
//! - `TransportOps::Execute` is the boundary type that does actual I/O
//! - Each file write chain includes a read→compare→skip upsert check
//!
//! Since Bootstrap writes two files (Makefile and .gitignore), we use two
//! separate read→compare→write chains that converge from the scan result.

use crate::bootstrap::ops::BootstrapOp;
use crate::file_ops_graph::FileOpsGraph;
use gunbc_ir::{
    add_content_upsert_chain, build::*, BuilderError, Cardinality, Dag, DagBuilder, Node,
    WorkflowSignature,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv, PrepareFileReadOp, PrepareFileWriteOp};

/// The operation type for bootstrap graphs - a union of bootstrap ops, primitives, and transport.
pub type BootstrapGraphOp = FileOpsGraph<BootstrapOp>;

/// Get the declared signature for the bootstrap workflow.
pub fn bootstrap_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs (entrypoints)
        .with_input("check_mode", "Bool", Cardinality::ZERO_OR_ONE)
        .with_input("path", "String", Cardinality::ONE)
        // Outputs from makefile write transport (boundary, skippable)
        .with_output("makefile_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("makefile_written_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("makefile_content", "String", Cardinality::ZERO_OR_ONE)
        // Outputs from gitignore write transport (boundary, skippable)
        .with_output("gitignore_response", "TransportResponse", Cardinality::ZERO_OR_ONE)
        .with_output("gitignore_written_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("gitignore_content", "String", Cardinality::ZERO_OR_ONE)
        // Freshness from compare nodes (terminal boundary outputs)
        .with_output("fresh", "Bool", Cardinality::ONE)
        // Skip from write transports (terminal boundary outputs)
        .with_output("skip", "Bool", Cardinality::ONE)
        .with_output("skip_reason", "String", Cardinality::ZERO_OR_ONE)
        // Informational outputs from scan_workspace
        .with_output("crate_count", "Int", Cardinality::ONE)
}

/// Build the bootstrap graph using DagBuilder.
///
/// Pipeline (follows content upsert pattern):
/// ```text
/// PrepareScan -> Execute -> ParseScanResult ─┬─→ GenerateMakefile ─┬─→ PrepareReadMakefile -> ExecuteReadMakefile -> CompareMakefileContent -> ExecuteMakefileTransport
///                                            │                     └─→ PrepareMakefileWrite ───────────────────────────────────────────────→ (request)
///                                            └─→ GenerateGitignore ─┬─→ PrepareReadGitignore -> ExecuteReadGitignore -> CompareGitignoreContent -> ExecuteGitignoreTransport
///                                                                   └─→ PrepareGitignoreWrite ────────────────────────────────────────────────→ (request)
/// ```
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs:write", "FilesystemHandle")],
        BootstrapGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
    ))?;

    // ========================================================================
    // ScanWorkspace chain: PrepareScanWorkspace -> Execute -> ParseScanResult
    // ========================================================================

    let prepare_scan = builder.add_root_node(Node::opaque(
        "prepare_scan_workspace",
        vec![],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        BootstrapGraphOp::Domain(BootstrapOp::PrepareScanWorkspace),
    ))?;

    let execute_scan = builder.add_node_after(
        Node::opaque(
            "execute_scan_workspace",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("fs", "FilesystemHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_scan,
    )?;

    let scan_workspace = builder.add_node_after(
        Node::opaque(
            "parse_scan_result",
            vec![port("response", "TransportResponse")],
            vec![port("crate_count", "Int"), list("crate_names", "StringList")],
            BootstrapGraphOp::Domain(BootstrapOp::ParseScanResult),
        ),
        &execute_scan,
    )?;

    // ========================================================================
    // Wire up the ScanWorkspace chain
    // ========================================================================
    builder.add_edge(prepare_scan.out("request"), execute_scan.in_port("request"))?;
    builder.add_edge(prepare_scan.out("skip"), execute_scan.in_port("skip"))?;
    builder.add_edge(
        execute_scan.out("response"),
        scan_workspace.in_port("response"),
    )?;

    // ========================================================================
    // Makefile upsert chain
    // ========================================================================

    let generate_makefile = builder.add_node_after(
        Node::opaque(
            "generate_makefile",
            vec![list("crate_names", "StringList")],
            vec![port("makefile_content", "String")],
            BootstrapGraphOp::Domain(BootstrapOp::GenerateMakefile),
        ),
        &scan_workspace,
    )?;

    builder.add_edge(
        scan_workspace.out("crate_names"),
        generate_makefile.in_port("crate_names"),
    )?;

    let makefile_read = resource("fs:Makefile", "FilesystemHandle", AccessMode::Read);
    let makefile_write = resource("fs:Makefile", "FilesystemHandle", AccessMode::Write);
    let makefile_chain = add_content_upsert_chain(
        &mut builder,
        "makefile",
        &generate_makefile,
        "makefile_content",
        vec![makefile_read],
        vec![makefile_write],
        BootstrapGraphOp::PrepareFileRead(PrepareFileReadOp),
        BootstrapGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        BootstrapGraphOp::Blob(BlobOps::CompareContent),
        BootstrapGraphOp::Transport(TransportOps::Execute),
    )?;

    // ========================================================================
    // Gitignore upsert chain
    // ========================================================================

    let generate_gitignore = builder.add_node_after(
        Node::opaque(
            "generate_gitignore",
            vec![list("crate_names", "StringList")],
            vec![port("gitignore_content", "String")],
            BootstrapGraphOp::Domain(BootstrapOp::GenerateGitignore),
        ),
        &scan_workspace,
    )?;

    builder.add_edge(
        scan_workspace.out("crate_names"),
        generate_gitignore.in_port("crate_names"),
    )?;

    let gitignore_read = resource("fs:.gitignore", "FilesystemHandle", AccessMode::Read);
    let gitignore_write = resource("fs:.gitignore", "FilesystemHandle", AccessMode::Write);
    let gitignore_chain = add_content_upsert_chain(
        &mut builder,
        "gitignore",
        &generate_gitignore,
        "gitignore_content",
        vec![gitignore_read],
        vec![gitignore_write],
        BootstrapGraphOp::PrepareFileRead(PrepareFileReadOp),
        BootstrapGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        BootstrapGraphOp::Blob(BlobOps::CompareContent),
        BootstrapGraphOp::Transport(TransportOps::Execute),
    )?;

    // Resource wiring
    builder.add_edge(fs_env.out("fs:write"), execute_scan.in_port("res:fs"))?;
    builder.add_edge(
        fs_env.out("fs:write"),
        makefile_chain.execute_read.in_port("res:fs:Makefile"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        makefile_chain.execute_write.in_port("res:fs:Makefile"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        gitignore_chain.execute_read.in_port("res:fs:.gitignore"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        gitignore_chain.execute_write.in_port("res:fs:.gitignore"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_bootstrap_graph().expect("graph should build");

        // Verify transport nodes exist
        assert!(dag.get_node(&"execute_scan_workspace".into()).is_some());
        assert!(dag.get_node(&"execute_read_makefile".into()).is_some());
        assert!(dag.get_node(&"execute_makefile_transport".into()).is_some());
        assert!(dag.get_node(&"execute_read_gitignore".into()).is_some());
        assert!(dag
            .get_node(&"execute_gitignore_transport".into())
            .is_some());
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // check_mode and read paths are entrypoints
        assert!(entrypoints.is_entrypoint_port(&"compare_makefile_content".into(), &"check_mode".into()));
        assert!(entrypoints.is_entrypoint_port(&"compare_gitignore_content".into(), &"check_mode".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_makefile".into(), &"path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_gitignore".into(), &"path".into()));
    }

    #[test]
    fn test_pure_nodes_not_boundaries() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Prepare and compare nodes are NOT boundaries - they're pure
        assert!(!boundaries.is_boundary_node(&"prepare_write_makefile".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_write_gitignore".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_makefile".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_gitignore".into()));
        // Generate nodes are NOT boundaries - all outputs connected
        assert!(!boundaries.is_boundary_node(&"generate_makefile".into()));
        assert!(!boundaries.is_boundary_node(&"generate_gitignore".into()));
    }

    // Signature validation tests are generated by testgen (via graph_mock).
}
