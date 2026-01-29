//! Graph builder for the gist tool.

use crate::ops::GistOp;
use gunbc_ir::{build::*, Dag, Edge, Node};

/// Build the gist generation graph.
///
/// Pipeline:
/// ```text
/// ListFiles -> FilterFiles -> ReadFiles -> RenderMarkdown -> PrepareGistRequest -> ExecuteTransport
///                                                                                       ↓
///                                                                                  (boundary)
/// ```
///
/// The transport layer separates pure business logic (PrepareGistRequest) from I/O
/// (ExecuteTransport). The boundary is now at the transport level, making dry-run
/// interception uniform across all I/O operations.
///
/// # Port Cardinalities
///
/// - `repo_path`: One (optional input, defaults to ".")
/// - `files`: ZeroOrMore (list of files, may be empty after filtering)
/// - `contents`: ZeroOrMore (map of file contents)
/// - `markdown`: One (single markdown document)
/// - `request`: One (transport request)
/// - `response`, `url`: One (transport response)
pub fn build_gist_graph(extensions: Vec<String>, public: bool) -> Dag<GistOp> {
    let mut dag = Dag::new();

    // Node: ListFiles
    // Input: optional repo_path (defaults to ".")
    // Output: list of files (may be empty if directory is empty)
    dag.add_node(Node::opaque(
        "list_files",
        vec![optional("repo_path", "String")],
        vec![list("files", "StrList")],
        GistOp::ListFiles,
    ));

    // Node: FilterFiles
    // Input/Output: list of files (may be empty after filtering)
    dag.add_node(Node::opaque(
        "filter_files",
        vec![list("files", "StrList")],
        vec![list("files", "StrList")],
        GistOp::FilterFiles { extensions },
    ));

    // Node: ReadFiles
    // Input: list of files, optional repo_path
    // Output: map of file contents (ZeroOrMore entries)
    dag.add_node(Node::opaque(
        "read_files",
        vec![list("files", "StrList"), optional("repo_path", "String")],
        vec![list("contents", "MapStrStr")],
        GistOp::ReadFiles,
    ));

    // Node: RenderMarkdown
    // Input: map of contents (ZeroOrMore entries)
    // Output: single markdown document
    dag.add_node(Node::opaque(
        "render_markdown",
        vec![list("contents", "MapStrStr")],
        vec![scalar("markdown", "String")],
        GistOp::RenderMarkdown,
    ));

    // Node: PrepareGistRequest (PURE - no I/O)
    // Input: single markdown document
    // Output: single transport request
    dag.add_node(Node::opaque(
        "prepare_gist_request",
        vec![scalar("markdown", "String")],
        vec![scalar("request", "TransportRequest")],
        GistOp::PrepareGistRequest { public },
    ));

    // Node: ExecuteTransport (BOUNDARY - world write)
    // Input: single transport request
    // Output: transport response and URL
    dag.add_node(Node::opaque(
        "execute_transport",
        vec![scalar("request", "TransportRequest")],
        vec![
            scalar("response", "TransportResponse"),
            scalar("url", "String"),
        ],
        GistOp::ExecuteTransport,
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

/// Build a graph with a context node that provides the repo_path.
pub fn build_gist_graph_with_context(
    _repo_path: &str,
    extensions: Vec<String>,
    public: bool,
) -> Dag<GistOp> {
    let dag = build_gist_graph(extensions, public);

    // Add a context node that provides the repo_path
    // This is needed because list_files and read_files both need repo_path
    // For now, we'll inject it via the seed inputs mechanism

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

        // Should have 6 nodes (added prepare_gist_request and execute_transport)
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
