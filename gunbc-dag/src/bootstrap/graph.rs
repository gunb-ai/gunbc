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
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{PrepareFileReadOp, PrepareFileWriteOp};
use std::collections::HashMap;

/// The operation type for bootstrap graphs - a union of bootstrap ops, primitives, and transport.
#[derive(Debug, Clone)]
pub enum BootstrapGraphOp {
    /// Bootstrap-specific operations
    Bootstrap(BootstrapOp),
    /// Prepare file read (primitive - PURE)
    PrepareFileRead(PrepareFileReadOp),
    /// Prepare file write (primitive - PURE)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Blob operations (compare content - PURE)
    Blob(BlobOps),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for BootstrapGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BootstrapGraphOp::Bootstrap(op) => op.execute(inputs),
            BootstrapGraphOp::PrepareFileRead(op) => op.execute(inputs),
            BootstrapGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            BootstrapGraphOp::Blob(op) => op.execute(inputs),
            BootstrapGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

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

    // ========================================================================
    // ScanWorkspace chain: PrepareScanWorkspace -> Execute -> ParseScanResult
    // ========================================================================

    let prepare_scan = builder.add_root_node(Node::opaque(
        "prepare_scan_workspace",
        vec![],
        vec![port("request", "TransportRequest")],
        BootstrapGraphOp::Bootstrap(BootstrapOp::PrepareScanWorkspace),
    ))?;

    let execute_scan = builder.add_node_after(
        Node::opaque(
            "execute_scan_workspace",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_scan,
    )?;

    let scan_workspace = builder.add_node_after(
        Node::opaque(
            "parse_scan_result",
            vec![port("response", "TransportResponse")],
            vec![port("crate_count", "Int"), list("crate_names", "String")],
            BootstrapGraphOp::Bootstrap(BootstrapOp::ParseScanResult),
        ),
        &execute_scan,
    )?;

    // ========================================================================
    // Makefile upsert chain
    // ========================================================================

    // Generate
    let generate_makefile = builder.add_node_after(
        Node::opaque(
            "generate_makefile",
            vec![list("crate_names", "String")],
            vec![port("makefile_content", "String")],
            BootstrapGraphOp::Bootstrap(BootstrapOp::GenerateMakefile),
        ),
        &scan_workspace,
    )?;

    // Read chain
    let prepare_makefile_read = builder.add_node_after(
        Node::opaque(
            "prepare_makefile_read",
            vec![port("path", "String")],
            vec![port("request", "TransportRequest")],
            BootstrapGraphOp::PrepareFileRead(PrepareFileReadOp),
        ),
        &generate_makefile,
    )?;

    let execute_makefile_read = builder.add_node_after(
        Node::opaque(
            "execute_makefile_read",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_makefile_read,
    )?;

    // Compare — uses BlobOps::CompareContent (response + expected_content compat path)
    let compare_makefile_content = builder.add_node_after(
        Node::opaque(
            "compare_makefile_content",
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
            BootstrapGraphOp::Blob(BlobOps::CompareContent),
        ),
        &execute_makefile_read,
    )?;

    // Write chain
    let prepare_makefile = builder.add_node_after(
        Node::opaque(
            "prepare_makefile_write",
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest")],
            BootstrapGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &generate_makefile,
    )?;

    let execute_makefile = builder.add_node_after(
        Node::opaque(
            "execute_makefile_transport",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional("makefile_response", "TransportResponse"),
                optional("makefile_written_path", "String"),
                optional("makefile_content", "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &compare_makefile_content,
    )?;

    // ========================================================================
    // Gitignore upsert chain
    // ========================================================================

    // Generate
    let generate_gitignore = builder.add_node_after(
        Node::opaque(
            "generate_gitignore",
            vec![list("crate_names", "String")],
            vec![port("gitignore_content", "String")],
            BootstrapGraphOp::Bootstrap(BootstrapOp::GenerateGitignore),
        ),
        &scan_workspace,
    )?;

    // Read chain
    let prepare_gitignore_read = builder.add_node_after(
        Node::opaque(
            "prepare_gitignore_read",
            vec![port("path", "String")],
            vec![port("request", "TransportRequest")],
            BootstrapGraphOp::PrepareFileRead(PrepareFileReadOp),
        ),
        &generate_gitignore,
    )?;

    let execute_gitignore_read = builder.add_node_after(
        Node::opaque(
            "execute_gitignore_read",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_gitignore_read,
    )?;

    // Compare — uses BlobOps::CompareContent (response + expected_content compat path)
    let compare_gitignore_content = builder.add_node_after(
        Node::opaque(
            "compare_gitignore_content",
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
            BootstrapGraphOp::Blob(BlobOps::CompareContent),
        ),
        &execute_gitignore_read,
    )?;

    // Write chain
    let prepare_gitignore = builder.add_node_after(
        Node::opaque(
            "prepare_gitignore_write",
            vec![port("path", "String"), port("content", "String")],
            vec![port("request", "TransportRequest")],
            BootstrapGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &generate_gitignore,
    )?;

    let execute_gitignore = builder.add_node_after(
        Node::opaque(
            "execute_gitignore_transport",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            vec![
                optional("gitignore_response", "TransportResponse"),
                optional("gitignore_written_path", "String"),
                optional("gitignore_content", "String"),
                port("skip", "Bool"),
                optional("skip_reason", "String"),
            ],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &compare_gitignore_content,
    )?;

    // ========================================================================
    // Wire up the ScanWorkspace chain
    // ========================================================================
    builder.add_edge(prepare_scan.out("request"), execute_scan.in_port("request"))?;
    builder.add_edge(
        execute_scan.out("response"),
        scan_workspace.in_port("response"),
    )?;

    // ========================================================================
    // Wire up the Makefile upsert chain
    // ========================================================================

    // ParseScanResult -> GenerateMakefile
    builder.add_edge(
        scan_workspace.out("crate_names"),
        generate_makefile.in_port("crate_names"),
    )?;

    // GenerateMakefile content -> CompareMakefileContent expected_content
    builder.add_edge(
        generate_makefile.out("makefile_content"),
        compare_makefile_content.in_port("expected_content"),
    )?;

    // GenerateMakefile content -> PrepareMakefileWrite content
    builder.add_edge(
        generate_makefile.out("makefile_content"),
        prepare_makefile.in_port("content"),
    )?;

    // PrepareReadMakefile -> ExecuteReadMakefile
    builder.add_edge(
        prepare_makefile_read.out("request"),
        execute_makefile_read.in_port("request"),
    )?;

    // ExecuteReadMakefile -> CompareMakefileContent
    builder.add_edge(
        execute_makefile_read.out("response"),
        compare_makefile_content.in_port("response"),
    )?;

    // CompareMakefileContent skip -> ExecuteMakefileTransport skip
    builder.add_edge(
        compare_makefile_content.out("skip"),
        execute_makefile.in_port("skip"),
    )?;

    // CompareMakefileContent skip_reason -> ExecuteMakefileTransport skip_reason
    builder.add_edge(
        compare_makefile_content.out("skip_reason"),
        execute_makefile.in_port("skip_reason"),
    )?;

    // PrepareMakefileWrite -> ExecuteMakefileTransport
    builder.add_edge(
        prepare_makefile.out("request"),
        execute_makefile.in_port("request"),
    )?;

    // ========================================================================
    // Wire up the Gitignore upsert chain
    // ========================================================================

    // ParseScanResult -> GenerateGitignore
    builder.add_edge(
        scan_workspace.out("crate_names"),
        generate_gitignore.in_port("crate_names"),
    )?;

    // GenerateGitignore content -> CompareGitignoreContent expected_content
    builder.add_edge(
        generate_gitignore.out("gitignore_content"),
        compare_gitignore_content.in_port("expected_content"),
    )?;

    // GenerateGitignore content -> PrepareGitignoreWrite content
    builder.add_edge(
        generate_gitignore.out("gitignore_content"),
        prepare_gitignore.in_port("content"),
    )?;

    // PrepareReadGitignore -> ExecuteReadGitignore
    builder.add_edge(
        prepare_gitignore_read.out("request"),
        execute_gitignore_read.in_port("request"),
    )?;

    // ExecuteReadGitignore -> CompareGitignoreContent
    builder.add_edge(
        execute_gitignore_read.out("response"),
        compare_gitignore_content.in_port("response"),
    )?;

    // CompareGitignoreContent skip -> ExecuteGitignoreTransport skip
    builder.add_edge(
        compare_gitignore_content.out("skip"),
        execute_gitignore.in_port("skip"),
    )?;

    // CompareGitignoreContent skip_reason -> ExecuteGitignoreTransport skip_reason
    builder.add_edge(
        compare_gitignore_content.out("skip_reason"),
        execute_gitignore.in_port("skip_reason"),
    )?;

    // PrepareGitignoreWrite -> ExecuteGitignoreTransport
    builder.add_edge(
        prepare_gitignore.out("request"),
        execute_gitignore.in_port("request"),
    )?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_bootstrap_graph().expect("graph should build");
        // 15 nodes: prepare_scan, execute_scan, parse_scan,
        //           gen_makefile, prep_makefile_read, exec_makefile_read, compare_makefile, prep_makefile_write, exec_makefile_transport,
        //           gen_gitignore, prep_gitignore_read, exec_gitignore_read, compare_gitignore, prep_gitignore_write, exec_gitignore_transport
        assert_eq!(dag.nodes.len(), 15);
        // 18 edges: 2 scan + 8 makefile + 8 gitignore
        assert_eq!(dag.edges.len(), 18);
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_bootstrap_graph().expect("graph should build");

        // Verify transport nodes exist
        assert!(dag.get_node(&"execute_scan_workspace".into()).is_some());
        assert!(dag.get_node(&"execute_makefile_read".into()).is_some());
        assert!(dag.get_node(&"execute_makefile_transport".into()).is_some());
        assert!(dag.get_node(&"execute_gitignore_read".into()).is_some());
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
        assert!(entrypoints.is_entrypoint_port(&"prepare_makefile_read".into(), &"path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_gitignore_read".into(), &"path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_bootstrap_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 15);
        assert_eq!(dag.edges.len(), 18);
    }

    #[test]
    fn test_pure_nodes_not_boundaries() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Prepare and compare nodes are NOT boundaries - they're pure
        assert!(!boundaries.is_boundary_node(&"prepare_makefile_write".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gitignore_write".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_makefile_read".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gitignore_read".into()));
        // Generate nodes are NOT boundaries - all outputs connected
        assert!(!boundaries.is_boundary_node(&"generate_makefile".into()));
        assert!(!boundaries.is_boundary_node(&"generate_gitignore".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let sig = bootstrap_signature();
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let inferred = infer_signature(&dag);

        // 6 inputs: makefile read path, gitignore read path,
        //           makefile write path, gitignore write path,
        //           makefile check_mode, gitignore check_mode
        assert_eq!(inferred.inputs.len(), 6);
    }
}
