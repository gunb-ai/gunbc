//! Graph builder for the gist tool.
//!
//! This graph is composed from library ops - it doesn't define any ops itself,
//! just wires together existing functionality.

use gunbc_ir::{build::*, Dag, Edge, Node};
use gunbc_lib_fs::FsOp;
use gunbc_lib_gist_ops::GistOps;
use gunbc_lib_markdown::MarkdownOp;

/// The operation type for gist graphs - a union of library ops.
#[derive(Debug, Clone)]
pub enum GistGraphOp {
    /// Filesystem operations
    Fs(FsOp),
    /// Markdown operations
    Markdown(MarkdownOp),
    /// Gist operations
    Gist(GistOps),
}

impl gunbc_exec::Executable for GistGraphOp {
    fn execute(
        &self,
        inputs: std::collections::HashMap<String, gunbc_ir::Value>,
    ) -> Result<std::collections::HashMap<String, gunbc_ir::Value>, gunbc_exec::ExecError> {
        match self {
            GistGraphOp::Fs(op) => op.execute(inputs),
            GistGraphOp::Markdown(op) => op.execute(inputs),
            GistGraphOp::Gist(op) => op.execute(inputs),
        }
    }
}

/// Build the gist generation graph.
///
/// Pipeline:
/// ```text
/// ListFiles -> FilterByExtension -> ReadFiles -> RenderCodeSnapshot -> PrepareRequest -> ExecuteTransport
///     ↓                                                                                        ↓
/// (fs::FsOp)                                                                              (boundary)
/// ```
///
/// This graph is composed entirely from library ops:
/// - `gunbc_ops::fs` for file operations
/// - `gunbc_ops::markdown` for markdown generation
/// - `gunbc_ops::gist` for gist-specific operations
pub fn build_gist_graph(extensions: Vec<String>, public: bool) -> Dag<GistGraphOp> {
    let mut dag = Dag::new();

    // Node: ListFiles (from fs flavor)
    dag.add_node(Node::opaque(
        "list_files",
        vec![optional("repo_path", "String")],
        vec![list("files", "StrList")],
        GistGraphOp::Fs(FsOp::ListFiles),
    ));

    // Node: FilterByExtension (from fs flavor)
    dag.add_node(Node::opaque(
        "filter_files",
        vec![list("files", "StrList")],
        vec![list("files", "StrList")],
        GistGraphOp::Fs(FsOp::FilterByExtension { extensions }),
    ));

    // Node: ReadFiles (from fs flavor)
    dag.add_node(Node::opaque(
        "read_files",
        vec![list("files", "StrList"), optional("repo_path", "String")],
        vec![list("contents", "MapStrStr")],
        GistGraphOp::Fs(FsOp::ReadFiles),
    ));

    // Node: RenderCodeSnapshot (from markdown flavor)
    dag.add_node(Node::opaque(
        "render_markdown",
        vec![list("contents", "MapStrStr")],
        vec![scalar("markdown", "String")],
        GistGraphOp::Markdown(MarkdownOp::RenderCodeSnapshot),
    ));

    // Node: PrepareGistRequest (from gist flavor - PURE)
    dag.add_node(Node::opaque(
        "prepare_gist_request",
        vec![scalar("markdown", "String")],
        vec![scalar("request", "TransportRequest")],
        GistGraphOp::Gist(GistOps::PrepareRequest { public }),
    ));

    // Node: ExecuteTransport (from gist flavor - BOUNDARY)
    dag.add_node(Node::opaque(
        "execute_transport",
        vec![scalar("request", "TransportRequest")],
        vec![
            scalar("response", "TransportResponse"),
            scalar("url", "String"),
        ],
        GistGraphOp::Gist(GistOps::ExecuteTransport),
    ));

    // Wire up the pipeline
    dag.add_edge(Edge::new("list_files", "files", "filter_files", "files"));
    dag.add_edge(Edge::new("filter_files", "files", "read_files", "files"));
    dag.add_edge(Edge::new("read_files", "contents", "render_markdown", "contents"));
    dag.add_edge(Edge::new(
        "render_markdown",
        "markdown",
        "prepare_gist_request",
        "markdown",
    ));
    dag.add_edge(Edge::new(
        "prepare_gist_request",
        "request",
        "execute_transport",
        "request",
    ));

    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_gist_graph(vec![], false);
        let boundaries = detect_boundaries(&dag);

        // ExecuteTransport should be the only boundary
        assert_eq!(boundaries.boundary_nodes.len(), 1);
        assert!(boundaries.is_boundary_node(&"execute_transport".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_gist_graph(vec![], false);
        let entrypoints = detect_entrypoints(&dag);

        // repo_path on list_files and read_files are entrypoints
        assert!(entrypoints.is_entrypoint_port(&"list_files".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"read_files".into(), &"repo_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_gist_graph(vec![], false);

        // Should have 6 nodes
        assert_eq!(dag.nodes.len(), 6);

        // Should have 5 edges (pipeline)
        assert_eq!(dag.edges.len(), 5);
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_gist_graph(vec![], false);
        let boundaries = detect_boundaries(&dag);

        // Intermediate nodes should not be boundaries
        assert!(!boundaries.is_boundary_node(&"list_files".into()));
        assert!(!boundaries.is_boundary_node(&"filter_files".into()));
        assert!(!boundaries.is_boundary_node(&"read_files".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
    }

    #[test]
    fn test_prepare_gist_request_not_boundary() {
        let dag = build_gist_graph(vec![], false);
        let boundaries = detect_boundaries(&dag);

        // PrepareGistRequest is pure - not a boundary
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
    }
}
