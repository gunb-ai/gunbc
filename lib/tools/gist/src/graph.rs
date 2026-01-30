//! Graph builder for the gist tool.
//!
//! This graph is composed from primitives and library ops.
//! Uses DagBuilder for compile-time cycle prevention and edge validation.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_gist_ops::GistOps;
use gunbc_lib_markdown::MarkdownOp;
use gunbc_primitives::{ListFilesOp, ReadFilesOp};
use std::collections::HashMap;

/// The operation type for gist graphs - a union of primitives and library ops.
#[derive(Debug, Clone)]
pub enum GistGraphOp {
    /// List files (primitive)
    ListFiles(ListFilesOp),
    /// Read multiple files (primitive)
    ReadFiles(ReadFilesOp),
    /// Filter by extension (local op - specialized for gist)
    FilterByExtension { extensions: Vec<String> },
    /// Markdown operations
    Markdown(MarkdownOp),
    /// Gist operations
    Gist(GistOps),
}

impl Executable for GistGraphOp {
    fn execute(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistGraphOp::ListFiles(op) => op.execute(inputs),
            GistGraphOp::ReadFiles(op) => op.execute(inputs),
            GistGraphOp::FilterByExtension { extensions } => {
                // Local specialized filter
                let files = inputs
                    .get("files")
                    .and_then(|v| v.as_str_list())
                    .ok_or_else(|| ExecError::new("missing or invalid 'files' input"))?;

                let filtered: Vec<String> = if extensions.is_empty() {
                    files
                } else {
                    files
                        .into_iter()
                        .filter(|f| extensions.iter().any(|ext| f.ends_with(ext)))
                        .collect()
                };

                let mut out = HashMap::new();
                out.insert("files".to_string(), Value::StrList(filtered));
                Ok(out)
            }
            GistGraphOp::Markdown(op) => op.execute(inputs),
            GistGraphOp::Gist(op) => op.execute(inputs),
        }
    }
}

/// Get the declared signature for the gist workflow.
///
/// Inputs (entrypoints):
/// - repo_path: optional path to scan for files
///
/// Outputs (boundaries):
/// - response: transport response from gist creation
/// - url: the created gist URL
pub fn gist_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        // Inputs - repo_path appears on both list_files and read_files
        .with_input("repo_path", "String", Cardinality::ZeroOrOne)
        // Outputs from execute_transport (boundary)
        .with_output("response", "TransportResponse", Cardinality::One)
        .with_output("url", "String", Cardinality::One)
}

/// Build the gist generation graph using DagBuilder.
///
/// Pipeline:
/// ```text
/// ListFiles -> FilterByExtension -> ReadFiles -> RenderCodeSnapshot -> PrepareRequest -> ExecuteTransport
///     ↓                                                                                        ↓
/// (primitive)                                                                              (boundary)
/// ```
///
/// # Benefits of DagBuilder
///
/// - Cycles are prevented by construction (generational tracking)
/// - Type and cardinality mismatches are caught at edge creation
/// - Signature validation ensures interface stability
///
/// # Future: RetryBuilder for HTTP
///
/// The execute_transport node could be wrapped in RetryBuilder for resilient HTTP:
/// ```ignore
/// let transport_with_retry = RetryBuilder::new("gist_transport_retry")
///     .with_body(transport_subdag)
///     .with_policy(RepeatPolicy::exponential(3, Duration::from_secs(1), 2.0))
///     .build();
/// ```
pub fn build_gist_graph(extensions: Vec<String>, public: bool) -> Result<Dag<GistGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: ListFiles (primitive) - generation 0
    let list_files = builder.add_root_node(Node::opaque(
        "list_files",
        vec![optional("repo_path", "String")],
        vec![list("files", "StrList")],
        GistGraphOp::ListFiles(ListFilesOp),
    ))?;

    // Node: FilterByExtension (local specialized op) - generation 1
    let filter_files = builder.add_node_after(
        Node::opaque(
            "filter_files",
            vec![list("files", "StrList")],
            vec![list("files", "StrList")],
            GistGraphOp::FilterByExtension { extensions },
        ),
        &list_files,
    )?;

    // Node: ReadFiles (primitive) - generation 2
    // Also has repo_path input (entrypoint, not connected)
    let read_files = builder.add_node_after(
        Node::opaque(
            "read_files",
            vec![list("files", "StrList"), optional("repo_path", "String")],
            vec![list("contents", "MapStrStr")],
            GistGraphOp::ReadFiles(ReadFilesOp),
        ),
        &filter_files,
    )?;

    // Node: RenderCodeSnapshot (from markdown flavor) - generation 3
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![list("contents", "MapStrStr")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderCodeSnapshot),
        ),
        &read_files,
    )?;

    // Node: PrepareGistRequest (from gist flavor - PURE) - generation 4
    let prepare_gist_request = builder.add_node_after(
        Node::opaque(
            "prepare_gist_request",
            vec![scalar("markdown", "String")],
            vec![scalar("request", "TransportRequest")],
            GistGraphOp::Gist(GistOps::PrepareRequest { public }),
        ),
        &render_markdown,
    )?;

    // Node: ExecuteTransport (from gist flavor - BOUNDARY) - generation 5
    let execute_transport = builder.add_node_after(
        Node::opaque(
            "execute_transport",
            vec![scalar("request", "TransportRequest")],
            vec![
                scalar("response", "TransportResponse"),
                scalar("url", "String"),
            ],
            GistGraphOp::Gist(GistOps::ExecuteTransport),
        ),
        &prepare_gist_request,
    )?;

    // Wire up the pipeline (validated at edge creation)
    builder.add_edge(list_files.out("files"), filter_files.in_port("files"))?;
    builder.add_edge(filter_files.out("files"), read_files.in_port("files"))?;
    builder.add_edge(read_files.out("contents"), render_markdown.in_port("contents"))?;
    builder.add_edge(render_markdown.out("markdown"), prepare_gist_request.in_port("markdown"))?;
    builder.add_edge(prepare_gist_request.out("request"), execute_transport.in_port("request"))?;

    Ok(builder.build())
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for GistGraphOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            GistGraphOp::ListFiles(_) => {
                let mut out = HashMap::new();
                out.insert("files".to_string(), Value::StrList(vec!["src/main.rs".to_string(), "README.md".to_string()]));
                out
            }
            GistGraphOp::ReadFiles(_) => {
                let mut out = HashMap::new();
                let mut contents = std::collections::BTreeMap::new();
                contents.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                out.insert("contents".to_string(), Value::MapStrStr(contents));
                out
            }
            GistGraphOp::FilterByExtension { .. } => {
                let mut out = HashMap::new();
                out.insert("files".to_string(), Value::StrList(vec!["src/main.rs".to_string()]));
                out
            }
            GistGraphOp::Markdown(_) => {
                let mut out = HashMap::new();
                out.insert("markdown".to_string(), Value::Str("# Code Snapshot\n```rust\nfn main() {}\n```".to_string()));
                out
            }
            GistGraphOp::Gist(_) => {
                let mut out = HashMap::new();
                out.insert("url".to_string(), Value::Str("https://gist.github.com/mock/123".to_string()));
                out.insert("response".to_string(), Value::Response(gunbc_ir::transport::TransportResponse::Shell(gunbc_ir::transport::ShellResponse {
                    exit_code: 0,
                    stdout: "https://gist.github.com/mock/123".to_string(),
                    stderr: String::new(),
                })));
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    #[test]
    fn test_graph_builds_successfully() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        assert_eq!(dag.nodes.len(), 6);
        assert_eq!(dag.edges.len(), 5);
    }

    #[test]
    fn test_graph_has_boundary() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // ExecuteTransport should be the only boundary
        assert_eq!(boundaries.boundary_nodes.len(), 1);
        assert!(boundaries.is_boundary_node(&"execute_transport".into()));
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // repo_path on list_files and read_files are entrypoints
        assert!(entrypoints.is_entrypoint_port(&"list_files".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"read_files".into(), &"repo_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");

        // Should have 6 nodes
        assert_eq!(dag.nodes.len(), 6);

        // Should have 5 edges (pipeline)
        assert_eq!(dag.edges.len(), 5);
    }

    #[test]
    fn test_intermediate_nodes_not_boundaries() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
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
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // PrepareGistRequest is pure - not a boundary
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
    }

    #[test]
    fn test_signature_matches_dag() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        let sig = gist_signature();
        
        // Validate declared signature matches inferred
        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_inferred_signature() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        let inferred = infer_signature(&dag);
        
        // Should have two inputs (repo_path on list_files and read_files)
        assert_eq!(inferred.inputs.len(), 2);
        
        // Should have two outputs (response, url)
        assert_eq!(inferred.outputs.len(), 2);
        let output_names: Vec<_> = inferred.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"response"));
        assert!(output_names.contains(&"url"));
    }
}
