//! Graph builder for the gist tool.
//!
//! This graph is composed from primitives and library ops.
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! # Transport Pattern
//!
//! File listing and reading use the transport layer for consistent I/O handling:
//! - ListFiles: uses ShellRequest("git ls-files") via transport
//! - ReadFiles: uses FileRequest::read for each file via transport

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{FileRequest, ShellRequest, TransportRequest, TransportResponse};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_gist_ops::GistOps;
use gunbc_lib_markdown::MarkdownOp;
use gunbc_lib_transport::execute_transport;
use std::collections::{BTreeMap, HashMap};

/// The operation type for gist graphs - a union of primitives and library ops.
#[derive(Debug, Clone)]
pub enum GistGraphOp {
    /// List files using git ls-files via transport
    ListFiles,
    /// Read multiple files using transport
    ReadFiles,
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
            GistGraphOp::ListFiles => execute_list_files(inputs),
            GistGraphOp::ReadFiles => execute_read_files(inputs),
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

/// List files using git ls-files via transport layer.
fn execute_list_files(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let repo_path = inputs
        .get("repo_path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    // Use git ls-files via transport
    let request = TransportRequest::Shell(ShellRequest {
        command: "git".to_string(),
        args: vec![
            "ls-files".to_string(),
            "--cached".to_string(),
            "--others".to_string(),
            "--exclude-standard".to_string(),
        ],
        cwd: Some(repo_path.to_string()),
        env: HashMap::new(),
        stdin: None,
    });

    let response = execute_transport(&request)
        .map_err(|e| ExecError::new(format!("failed to list files: {}", e)))?;

    let files = match response {
        TransportResponse::Shell(shell) => {
            if shell.success() {
                shell
                    .stdout
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| l.to_string())
                    .collect()
            } else {
                // Fallback: try ls command if not in a git repo
                list_files_fallback(repo_path)?
            }
        }
        _ => return Err(ExecError::new("unexpected response type")),
    };

    let mut out = HashMap::new();
    out.insert("files".to_string(), Value::StrList(files));
    Ok(out)
}

/// Fallback file listing when not in a git repo.
fn list_files_fallback(path: &str) -> Result<Vec<String>, ExecError> {
    let request = TransportRequest::Shell(ShellRequest {
        command: "find".to_string(),
        args: vec![path.to_string(), "-type".to_string(), "f".to_string()],
        cwd: None,
        env: HashMap::new(),
        stdin: None,
    });

    let response = execute_transport(&request)
        .map_err(|e| ExecError::new(format!("failed to list files: {}", e)))?;

    match response {
        TransportResponse::Shell(shell) => {
            Ok(shell
                .stdout
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with('.'))
                .map(|l| l.to_string())
                .collect())
        }
        _ => Err(ExecError::new("unexpected response type")),
    }
}

/// Read multiple files using transport layer.
fn execute_read_files(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let files = inputs
        .get("files")
        .and_then(|v| v.as_str_list())
        .ok_or_else(|| ExecError::new("missing or invalid 'files' input"))?;

    let repo_path = inputs
        .get("repo_path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    let mut contents = BTreeMap::new();

    for file in &files {
        let full_path = if repo_path == "." {
            file.clone()
        } else {
            format!("{}/{}", repo_path, file)
        };

        let request = TransportRequest::File(FileRequest::read(&full_path));

        match execute_transport(&request) {
            Ok(TransportResponse::File(file_resp)) => {
                if let Some(content) = file_resp.content {
                    contents.insert(file.clone(), content);
                }
                // Silently skip files that can't be read (binary, permissions, etc.)
            }
            _ => {
                // Silently skip files that can't be read
            }
        }
    }

    let mut out = HashMap::new();
    out.insert("contents".to_string(), Value::MapStrStr(contents));
    Ok(out)
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
/// (transport)                                                                              (boundary)
/// ```
///
/// # Benefits of DagBuilder
///
/// - Cycles are prevented by construction (generational tracking)
/// - Type and cardinality mismatches are caught at edge creation
/// - Signature validation ensures interface stability
pub fn build_gist_graph(
    extensions: Vec<String>,
    public: bool,
) -> Result<Dag<GistGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // Node: ListFiles (uses transport internally) - generation 0
    let list_files = builder.add_root_node(Node::opaque(
        "list_files",
        vec![optional("repo_path", "String")],
        vec![list("files", "StrList")],
        GistGraphOp::ListFiles,
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

    // Node: ReadFiles (uses transport internally) - generation 2
    // Also has repo_path input (entrypoint, not connected)
    let read_files = builder.add_node_after(
        Node::opaque(
            "read_files",
            vec![list("files", "StrList"), optional("repo_path", "String")],
            vec![list("contents", "MapStrStr")],
            GistGraphOp::ReadFiles,
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
    builder.add_edge(
        render_markdown.out("markdown"),
        prepare_gist_request.in_port("markdown"),
    )?;
    builder.add_edge(
        prepare_gist_request.out("request"),
        execute_transport.in_port("request"),
    )?;

    Ok(builder.build())
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for GistGraphOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            GistGraphOp::ListFiles => {
                let mut out = HashMap::new();
                out.insert(
                    "files".to_string(),
                    Value::StrList(vec!["src/main.rs".to_string(), "README.md".to_string()]),
                );
                out
            }
            GistGraphOp::ReadFiles => {
                let mut out = HashMap::new();
                let mut contents = std::collections::BTreeMap::new();
                contents.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                out.insert("contents".to_string(), Value::MapStrStr(contents));
                out
            }
            GistGraphOp::FilterByExtension { .. } => {
                let mut out = HashMap::new();
                out.insert(
                    "files".to_string(),
                    Value::StrList(vec!["src/main.rs".to_string()]),
                );
                out
            }
            GistGraphOp::Markdown(_) => {
                let mut out = HashMap::new();
                out.insert(
                    "markdown".to_string(),
                    Value::Str("# Code Snapshot\n```rust\nfn main() {}\n```".to_string()),
                );
                out
            }
            GistGraphOp::Gist(_) => {
                let mut out = HashMap::new();
                out.insert(
                    "url".to_string(),
                    Value::Str("https://gist.github.com/mock/123".to_string()),
                );
                out.insert(
                    "response".to_string(),
                    Value::Response(gunbc_ir::transport::TransportResponse::Shell(
                        gunbc_ir::transport::ShellResponse {
                            exit_code: 0,
                            stdout: "https://gist.github.com/mock/123".to_string(),
                            stderr: String::new(),
                        },
                    )),
                );
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
