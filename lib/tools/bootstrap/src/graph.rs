//! Graph builder for the bootstrap tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool follows the transport pattern:
//! - Pure ops prepare data and `TransportRequest` values
//! - `TransportOps::Execute` is the single boundary type that does actual I/O
//!
//! Since Bootstrap writes two files (Makefile and .gitignore), we use two
//! separate prepare→execute chains that converge at a CollectResults node.

use crate::ops::BootstrapOp;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::PrepareFileWriteOp;
use std::collections::HashMap;

/// The operation type for bootstrap graphs - a union of bootstrap ops, primitives, and transport.
#[derive(Debug, Clone)]
pub enum BootstrapGraphOp {
    /// Bootstrap-specific operations
    Bootstrap(BootstrapOp),
    /// Prepare file write (primitive - PURE)
    PrepareFileWrite(PrepareFileWriteOp),
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for BootstrapGraphOp {
    fn execute(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BootstrapGraphOp::Bootstrap(op) => op.execute(inputs),
            BootstrapGraphOp::PrepareFileWrite(op) => op.execute(inputs),
            BootstrapGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the bootstrap workflow.
pub fn bootstrap_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // No inputs (scan_workspace has no entrypoint inputs)
        // Outputs from makefile transport execution (boundary)
        .with_output("makefile_response", "TransportResponse", Cardinality::One)
        .with_output("makefile_written_path", "String", Cardinality::One)
        .with_output("makefile_content", "String", Cardinality::One)
        // Outputs from gitignore transport execution (boundary)
        .with_output("gitignore_response", "TransportResponse", Cardinality::One)
        .with_output("gitignore_written_path", "String", Cardinality::One)
        .with_output("gitignore_content", "String", Cardinality::One)
        // Informational outputs from scan_workspace
        .with_output("crate_count", "Int", Cardinality::One)
}

/// Build the bootstrap graph using DagBuilder.
///
/// Pipeline (follows transport pattern):
/// ```text
/// ScanWorkspace -> GenerateMakefile  -> PrepareMakefileWrite  -> ExecuteMakefileTransport
///      |                                      (PURE)                   (BOUNDARY)
///      |
///      +--------> GenerateGitignore -> PrepareGitignoreWrite -> ExecuteGitignoreTransport
///                                            (PURE)                   (BOUNDARY)
/// ```
///
/// Each file write has its own prepare→execute chain for proper transport interception.
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: ScanWorkspace (bootstrap-specific) - generation 0
    // NOTE: This still uses direct fs operations internally. A future refactor
    // could use PrepareShellOp + TransportOps::Execute with `find` or `ls`.
    let scan_workspace = builder.add_root_node(Node::opaque(
        "scan_workspace",
        vec![],
        vec![
            port("crate_count", "Int"),
            port("crate_names", "StrList"),
        ],
        BootstrapGraphOp::Bootstrap(BootstrapOp::ScanWorkspace),
    ))?;

    // === Makefile write chain ===

    // Node: GenerateMakefile (bootstrap-specific) - generation 1
    let generate_makefile = builder.add_node_after(
        Node::opaque(
            "generate_makefile",
            vec![port("crate_names", "StrList")],
            vec![port("makefile_content", "String")],
            BootstrapGraphOp::Bootstrap(BootstrapOp::GenerateMakefile),
        ),
        &scan_workspace,
    )?;

    // Node: PrepareMakefileWrite (primitive - PURE) - generation 2
    let prepare_makefile = builder.add_node_after(
        Node::opaque(
            "prepare_makefile_write",
            vec![port("content", "String")],
            vec![port("request", "TransportRequest")],
            BootstrapGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &generate_makefile,
    )?;

    // Node: ExecuteMakefileTransport (transport - BOUNDARY) - generation 3
    let execute_makefile = builder.add_node_after(
        Node::opaque(
            "execute_makefile_transport",
            vec![port("request", "TransportRequest")],
            vec![
                port("makefile_response", "TransportResponse"),
                port("makefile_written_path", "String"),
                port("makefile_content", "String"),
            ],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_makefile,
    )?;

    // === Gitignore write chain ===

    // Node: GenerateGitignore (bootstrap-specific) - generation 1 (parallel with makefile)
    let generate_gitignore = builder.add_node_after(
        Node::opaque(
            "generate_gitignore",
            vec![port("crate_names", "StrList")],
            vec![port("gitignore_content", "String")],
            BootstrapGraphOp::Bootstrap(BootstrapOp::GenerateGitignore),
        ),
        &scan_workspace,
    )?;

    // Node: PrepareGitignoreWrite (primitive - PURE) - generation 2
    let prepare_gitignore = builder.add_node_after(
        Node::opaque(
            "prepare_gitignore_write",
            vec![port("content", "String")],
            vec![port("request", "TransportRequest")],
            BootstrapGraphOp::PrepareFileWrite(PrepareFileWriteOp),
        ),
        &generate_gitignore,
    )?;

    // Node: ExecuteGitignoreTransport (transport - BOUNDARY) - generation 3
    let _execute_gitignore = builder.add_node_after(
        Node::opaque(
            "execute_gitignore_transport",
            vec![port("request", "TransportRequest")],
            vec![
                port("gitignore_response", "TransportResponse"),
                port("gitignore_written_path", "String"),
                port("gitignore_content", "String"),
            ],
            BootstrapGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_gitignore,
    )?;

    // Wire up the Makefile chain
    builder.add_edge(scan_workspace.out("crate_names"), generate_makefile.in_port("crate_names"))?;
    builder.add_edge(generate_makefile.out("makefile_content"), prepare_makefile.in_port("content"))?;
    builder.add_edge(prepare_makefile.out("request"), execute_makefile.in_port("request"))?;

    // Wire up the Gitignore chain
    builder.add_edge(scan_workspace.out("crate_names"), generate_gitignore.in_port("crate_names"))?;
    builder.add_edge(generate_gitignore.out("gitignore_content"), prepare_gitignore.in_port("content"))?;
    builder.add_edge(prepare_gitignore.out("request"), _execute_gitignore.in_port("request"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_bootstrap_graph().expect("graph should build");
        // 7 nodes: scan, gen_makefile, gen_gitignore, prep_makefile, prep_gitignore,
        //          exec_makefile, exec_gitignore
        assert_eq!(dag.nodes.len(), 7);
        // 6 edges: 3 for makefile chain, 3 for gitignore chain
        assert_eq!(dag.edges.len(), 6);
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Both transport execute nodes should be boundaries
        assert!(boundaries.is_boundary_node(&"execute_makefile_transport".into()));
        assert!(boundaries.is_boundary_node(&"execute_gitignore_transport".into()));
    }

    #[test]
    fn test_graph_no_entrypoints() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // ScanWorkspace has no inputs, so nothing should be an entrypoint
        // (entrypoints are inputs without upstream, but scan_workspace has no inputs at all)
        assert!(entrypoints.entrypoint_ports.is_empty());
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_bootstrap_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 7);
        assert_eq!(dag.edges.len(), 6);
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Prepare nodes are NOT boundaries - they're pure
        assert!(!boundaries.is_boundary_node(&"prepare_makefile_write".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gitignore_write".into()));
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
        
        // 0 inputs (no entrypoints)
        assert_eq!(inferred.inputs.len(), 0);
        // Boundary outputs: 3 from each transport execute (6 total) + 1 crate_count from scan
        assert_eq!(inferred.outputs.len(), 7);
    }
}
