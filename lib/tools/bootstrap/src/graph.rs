//! Graph builder for the bootstrap tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.

use crate::ops::BootstrapOp;
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, WorkflowSignature,
};

/// Get the declared signature for the bootstrap workflow.
pub fn bootstrap_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // No inputs (scan_workspace has no entrypoint inputs)
        // Outputs - boundary outputs
        .with_output("crate_count", "Int", Cardinality::One)
        .with_output("files_written", "StrList", Cardinality::One)
        .with_output("write_count", "Int", Cardinality::One)
}

/// Build the bootstrap graph using DagBuilder.
///
/// Pipeline:
/// ```text
/// ScanWorkspace -> GenerateMakefile  -> WriteFiles
///               -> GenerateGitignore -/
///                                   (boundary)
/// ```
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: ScanWorkspace - generation 0
    let scan_workspace = builder.add_root_node(Node::opaque(
        "scan_workspace",
        vec![],
        vec![
            port("crate_count", "Int"),
            port("crate_names", "StrList"),
        ],
        BootstrapOp::ScanWorkspace,
    ))?;

    // Node: GenerateMakefile - generation 1
    let generate_makefile = builder.add_node_after(
        Node::opaque(
            "generate_makefile",
            vec![port("crate_names", "StrList")],
            vec![port("makefile_content", "String")],
            BootstrapOp::GenerateMakefile,
        ),
        &scan_workspace,
    )?;

    // Node: GenerateGitignore - generation 1 (parallel with makefile)
    let generate_gitignore = builder.add_node_after(
        Node::opaque(
            "generate_gitignore",
            vec![port("crate_names", "StrList")],
            vec![port("gitignore_content", "String")],
            BootstrapOp::GenerateGitignore,
        ),
        &scan_workspace,
    )?;

    // Node: WriteFiles (BOUNDARY) - generation 2
    let write_files = builder.add_node_after_all(
        Node::opaque(
            "write_files",
            vec![
                port("makefile_content", "String"),
                port("gitignore_content", "String"),
            ],
            vec![
                port("files_written", "StrList"),
                port("write_count", "Int"),
            ],
            BootstrapOp::WriteFiles,
        ),
        &[&generate_makefile, &generate_gitignore],
    )?;

    // Wire up the pipeline
    builder.add_edge(scan_workspace.out("crate_names"), generate_makefile.in_port("crate_names"))?;
    builder.add_edge(scan_workspace.out("crate_names"), generate_gitignore.in_port("crate_names"))?;
    builder.add_edge(generate_makefile.out("makefile_content"), write_files.in_port("makefile_content"))?;
    builder.add_edge(generate_gitignore.out("gitignore_content"), write_files.in_port("gitignore_content"))?;

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_bootstrap_graph().expect("graph should build");
        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 4);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_bootstrap_graph().expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // WriteFiles should be a boundary
        assert!(boundaries.is_boundary_node(&"write_files".into()));
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

        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 4);
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
        
        // 0 inputs (no entrypoints), 3 boundary outputs
        assert_eq!(inferred.inputs.len(), 0);
        assert_eq!(inferred.outputs.len(), 3);
    }
}
