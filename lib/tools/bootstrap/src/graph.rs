//! Graph builder for the bootstrap tool.

use crate::ops::BootstrapOp;
use gunbc_ir::{build::*, Dag, Edge, Node};

/// Build the bootstrap graph.
///
/// Pipeline:
/// ```text
/// ScanWorkspace -> GenerateMakefile  -> WriteFiles
///               -> GenerateGitignore -/
///                                   (boundary)
/// ```
pub fn build_bootstrap_graph() -> Dag<BootstrapOp> {
    let mut dag = Dag::new();

    // Node: ScanWorkspace
    dag.add_node(Node::opaque(
        "scan_workspace",
        vec![],
        vec![
            port("crate_count", "Int"),
            port("crate_names", "StrList"),
        ],
        BootstrapOp::ScanWorkspace,
    ));

    // Node: GenerateMakefile
    dag.add_node(Node::opaque(
        "generate_makefile",
        vec![port("crate_names", "StrList")],
        vec![port("makefile_content", "String")],
        BootstrapOp::GenerateMakefile,
    ));

    // Node: GenerateGitignore
    dag.add_node(Node::opaque(
        "generate_gitignore",
        vec![port("crate_names", "StrList")],
        vec![port("gitignore_content", "String")],
        BootstrapOp::GenerateGitignore,
    ));

    // Node: WriteFiles (BOUNDARY)
    dag.add_node(Node::opaque(
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
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new(
        "scan_workspace",
        "crate_names",
        "generate_makefile",
        "crate_names",
    ));
    dag.add_edge(Edge::new(
        "scan_workspace",
        "crate_names",
        "generate_gitignore",
        "crate_names",
    ));
    dag.add_edge(Edge::new(
        "generate_makefile",
        "makefile_content",
        "write_files",
        "makefile_content",
    ));
    dag.add_edge(Edge::new(
        "generate_gitignore",
        "gitignore_content",
        "write_files",
        "gitignore_content",
    ));

    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_bootstrap_graph();
        let boundaries = detect_boundaries(&dag);

        // WriteFiles should be a boundary
        assert!(boundaries.is_boundary_node(&"write_files".into()));
    }

    #[test]
    fn test_graph_no_entrypoints() {
        let dag = build_bootstrap_graph();
        let entrypoints = detect_entrypoints(&dag);

        // ScanWorkspace has no inputs, so nothing should be an entrypoint
        // (entrypoints are inputs without upstream, but scan_workspace has no inputs at all)
        assert!(entrypoints.entrypoint_ports.is_empty());
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_bootstrap_graph();

        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 4);
    }
}
