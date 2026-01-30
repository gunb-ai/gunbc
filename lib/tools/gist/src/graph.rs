//! Graph builder for the gist tool.
//!
//! This graph is composed from primitives and library ops.
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! # Transport Pattern
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes:
//! - ListFiles: PrepareListFiles -> Execute -> ParseListFiles
//! - ReadFiles: PrepareReadFiles -> Execute -> ParseReadFiles (with loop)
//! - Gist creation: PrepareRequest -> Execute

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{ShellRequest, TransportRequest, TransportResponse};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_gist_ops::GistOps;
use gunbc_lib_markdown::MarkdownOp;
use gunbc_lib_transport::TransportOps;
use std::collections::{BTreeMap, HashMap};

/// The operation type for gist graphs - a union of pure ops, library ops, and transport.
///
/// Following the CI pattern: all I/O happens through `Transport(TransportOps::Execute)` nodes.
#[derive(Debug, Clone)]
pub enum GistGraphOp {
    // ========================================================================
    // ListFiles chain: PrepareListFiles -> Execute -> ParseListFiles
    // ========================================================================
    /// Prepare git ls-files request (PURE - no I/O)
    PrepareListFiles,
    /// Parse list files response to file list (PURE - no I/O)
    ParseListFiles,

    // ========================================================================
    // ReadFiles chain: PrepareReadFiles -> Execute -> ParseReadFiles
    // ========================================================================
    /// Prepare batch file read request (PURE - no I/O)
    /// Takes file list and repo_path, outputs shell command to read files
    PrepareReadFiles,
    /// Parse batch file read response (PURE - no I/O)
    /// Takes shell response, outputs contents map
    ParseReadFiles,

    // ========================================================================
    // Pure local ops
    // ========================================================================
    /// Filter by extension (local op - specialized for gist)
    FilterByExtension { extensions: Vec<String> },

    // ========================================================================
    // Library ops
    // ========================================================================
    /// Markdown operations
    Markdown(MarkdownOp),
    /// Gist operations (PrepareRequest is pure, ExecuteTransport will be removed)
    Gist(GistOps),

    // ========================================================================
    // Transport boundary (actual I/O)
    // ========================================================================
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl Executable for GistGraphOp {
    fn execute(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // ListFiles chain (pure)
            GistGraphOp::PrepareListFiles => execute_prepare_list_files(inputs),
            GistGraphOp::ParseListFiles => execute_parse_list_files(inputs),

            // ReadFiles chain (pure)
            GistGraphOp::PrepareReadFiles => execute_prepare_read_files(inputs),
            GistGraphOp::ParseReadFiles => execute_parse_read_files(inputs),

            // Pure local ops
            GistGraphOp::FilterByExtension { extensions } => {
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

            // Library ops
            GistGraphOp::Markdown(op) => op.execute(inputs),
            GistGraphOp::Gist(op) => op.execute(inputs),

            // Transport boundary
            GistGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

// ============================================================================
// PrepareListFiles - PURE (builds TransportRequest)
// ============================================================================

/// Prepare git ls-files request (PURE - no I/O).
///
/// Inputs:
/// - repo_path: optional path to scan (defaults to ".")
///
/// Outputs:
/// - request: TransportRequest for git ls-files
fn execute_prepare_list_files(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let repo_path = inputs
        .get("repo_path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

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

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    Ok(out)
}

// ============================================================================
// ParseListFiles - PURE (parses TransportResponse)
// ============================================================================

/// Parse list files response to file list (PURE - no I/O).
///
/// Inputs:
/// - response: TransportResponse from git ls-files
///
/// Outputs:
/// - files: list of file paths
fn execute_parse_list_files(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing or invalid 'response' input"))?;

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
                // Return empty list on failure (could be non-git repo)
                // In future, could use BranchBuilder for fallback to find
                Vec::new()
            }
        }
        _ => return Err(ExecError::new("unexpected response type")),
    };

    let mut out = HashMap::new();
    out.insert("files".to_string(), Value::StrList(files));
    Ok(out)
}

// ============================================================================
// PrepareReadFiles - PURE (builds batch file read shell command)
// ============================================================================

/// File marker used to delimit files in batch read output.
const FILE_MARKER: &str = "===GUNBC_FILE:";
const FILE_MARKER_END: &str = "===";

/// Prepare batch file read request (PURE - no I/O).
///
/// Creates a shell command that reads all files with markers for parsing.
///
/// Inputs:
/// - files: list of file paths to read
/// - repo_path: optional base path (defaults to ".")
///
/// Outputs:
/// - request: TransportRequest (shell command to read files)
fn execute_prepare_read_files(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let files = inputs
        .get("files")
        .and_then(|v| v.as_str_list())
        .ok_or_else(|| ExecError::new("missing or invalid 'files' input"))?;

    let repo_path = inputs
        .get("repo_path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    // Build full paths
    let full_paths: Vec<String> = files
        .iter()
        .map(|f| {
            if repo_path == "." {
                f.clone()
            } else {
                format!("{}/{}", repo_path, f)
            }
        })
        .collect();

    // Create a shell command that reads each file with markers
    // Format: for each file, output "===GUNBC_FILE:filename===" then file content
    // Use bash -c with a heredoc-style approach for reliability
    let script = full_paths
        .iter()
        .zip(files.iter())
        .map(|(full_path, original_name)| {
            // Use the original name (not full path) as the key for the map
            format!(
                "echo '{}{}{}'; cat '{}' 2>/dev/null || true",
                FILE_MARKER, original_name, FILE_MARKER_END, full_path
            )
        })
        .collect::<Vec<_>>()
        .join("; ");

    let request = TransportRequest::Shell(ShellRequest {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script],
        cwd: None,
        env: HashMap::new(),
        stdin: None,
    });

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(request));
    Ok(out)
}

// ============================================================================
// ParseReadFiles - PURE (parses batch file read response)
// ============================================================================

/// Parse batch file read response to contents map (PURE - no I/O).
///
/// Inputs:
/// - response: TransportResponse from batch file read
///
/// Outputs:
/// - contents: map of filename -> content
fn execute_parse_read_files(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = inputs
        .get("response")
        .and_then(|v| v.as_response())
        .ok_or_else(|| ExecError::new("missing or invalid 'response' input"))?;

    let stdout = match response {
        TransportResponse::Shell(shell) => shell.stdout.clone(),
        _ => return Err(ExecError::new("unexpected response type")),
    };

    // Parse the output: look for ===GUNBC_FILE:name=== markers
    let mut contents = BTreeMap::new();
    let mut current_file: Option<String> = None;
    let mut current_content = String::new();

    for line in stdout.lines() {
        if line.starts_with(FILE_MARKER) && line.ends_with(FILE_MARKER_END) {
            // Save previous file if any
            if let Some(filename) = current_file.take() {
                // Trim trailing newline from content
                let content = current_content.trim_end().to_string();
                if !content.is_empty() {
                    contents.insert(filename, content);
                }
            }

            // Extract new filename
            let name = line
                .strip_prefix(FILE_MARKER)
                .and_then(|s| s.strip_suffix(FILE_MARKER_END))
                .unwrap_or("")
                .to_string();
            current_file = Some(name);
            current_content = String::new();
        } else if current_file.is_some() {
            // Append to current file content
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        }
    }

    // Save last file
    if let Some(filename) = current_file {
        let content = current_content.trim_end().to_string();
        if !content.is_empty() {
            contents.insert(filename, content);
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
        // Inputs - repo_path appears on prepare_list_files and prepare_read_files
        .with_input("repo_path", "String", Cardinality::ZeroOrOne)
        // Outputs - url from parse_gist_response (boundary)
        .with_output("url", "String", Cardinality::One)
}

/// Build the gist generation graph using DagBuilder.
///
/// Pipeline (with explicit transport nodes):
/// ```text
/// PrepareListFiles -> Execute -> ParseListFiles -> Filter -> PrepareReadFiles -> Execute -> ParseReadFiles -> Render -> PrepareGist -> Execute
///                       ↑                                                          ↑                                                    ↑
///                   (boundary)                                                 (boundary)                                          (boundary)
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

    // ========================================================================
    // ListFiles chain: PrepareListFiles -> Execute -> ParseListFiles
    // ========================================================================

    // Node: PrepareListFiles (PURE - builds TransportRequest)
    let prepare_list_files = builder.add_root_node(Node::opaque(
        "prepare_list_files",
        vec![optional("repo_path", "String")],
        vec![port("request", "TransportRequest")],
        GistGraphOp::PrepareListFiles,
    ))?;

    // Node: Execute list files (BOUNDARY - actual I/O)
    let execute_list_files = builder.add_node_after(
        Node::opaque(
            "execute_list_files",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_list_files,
    )?;

    // Node: ParseListFiles (PURE - parses response to file list)
    let parse_list_files = builder.add_node_after(
        Node::opaque(
            "parse_list_files",
            vec![port("response", "TransportResponse")],
            vec![list("files", "StrList")],
            GistGraphOp::ParseListFiles,
        ),
        &execute_list_files,
    )?;

    // ========================================================================
    // Filter
    // ========================================================================

    // Node: FilterByExtension (PURE)
    let filter_files = builder.add_node_after(
        Node::opaque(
            "filter_files",
            vec![list("files", "StrList")],
            vec![list("files", "StrList")],
            GistGraphOp::FilterByExtension { extensions },
        ),
        &parse_list_files,
    )?;

    // ========================================================================
    // ReadFiles chain: PrepareReadFiles -> Execute -> ParseReadFiles
    // ========================================================================

    // Node: PrepareReadFiles (PURE - builds batch read shell command)
    let prepare_read_files = builder.add_node_after(
        Node::opaque(
            "prepare_read_files",
            vec![list("files", "StrList"), optional("repo_path", "String")],
            vec![port("request", "TransportRequest")],
            GistGraphOp::PrepareReadFiles,
        ),
        &filter_files,
    )?;

    // Node: Execute read files (BOUNDARY - actual I/O)
    let execute_read_files = builder.add_node_after(
        Node::opaque(
            "execute_read_files",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_read_files,
    )?;

    // Node: ParseReadFiles (PURE - extracts contents map from response)
    let parse_read_files = builder.add_node_after(
        Node::opaque(
            "parse_read_files",
            vec![port("response", "TransportResponse")],
            vec![list("contents", "MapStrStr")],
            GistGraphOp::ParseReadFiles,
        ),
        &execute_read_files,
    )?;

    // ========================================================================
    // Render and Gist creation
    // ========================================================================

    // Node: RenderCodeSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![list("contents", "MapStrStr")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderCodeSnapshot),
        ),
        &parse_read_files,
    )?;

    // Node: PrepareGistRequest (PURE)
    let prepare_gist_request = builder.add_node_after(
        Node::opaque(
            "prepare_gist_request",
            vec![scalar("markdown", "String")],
            vec![scalar("request", "TransportRequest")],
            GistGraphOp::Gist(GistOps::PrepareRequest { public }),
        ),
        &render_markdown,
    )?;

    // Node: ExecuteGist (BOUNDARY - actual I/O via TransportOps::Execute)
    let execute_gist = builder.add_node_after(
        Node::opaque(
            "execute_gist",
            vec![scalar("request", "TransportRequest")],
            vec![scalar("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_gist_request,
    )?;

    // Node: ParseGistResponse (PURE - extracts URL from response)
    let parse_gist_response = builder.add_node_after(
        Node::opaque(
            "parse_gist_response",
            vec![scalar("response", "TransportResponse")],
            vec![scalar("url", "String")],
            GistGraphOp::Gist(GistOps::ParseGistResponse),
        ),
        &execute_gist,
    )?;

    // ========================================================================
    // Wire up the pipeline
    // ========================================================================

    // ListFiles chain
    builder.add_edge(
        prepare_list_files.out("request"),
        execute_list_files.in_port("request"),
    )?;
    builder.add_edge(
        execute_list_files.out("response"),
        parse_list_files.in_port("response"),
    )?;

    // Filter
    builder.add_edge(parse_list_files.out("files"), filter_files.in_port("files"))?;

    // ReadFiles chain
    builder.add_edge(filter_files.out("files"), prepare_read_files.in_port("files"))?;
    builder.add_edge(
        prepare_read_files.out("request"),
        execute_read_files.in_port("request"),
    )?;
    builder.add_edge(
        execute_read_files.out("response"),
        parse_read_files.in_port("response"),
    )?;

    // Render and Gist chain
    builder.add_edge(parse_read_files.out("contents"), render_markdown.in_port("contents"))?;
    builder.add_edge(
        render_markdown.out("markdown"),
        prepare_gist_request.in_port("markdown"),
    )?;
    builder.add_edge(
        prepare_gist_request.out("request"),
        execute_gist.in_port("request"),
    )?;
    builder.add_edge(
        execute_gist.out("response"),
        parse_gist_response.in_port("response"),
    )?;

    Ok(builder.build())
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for GistGraphOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            // ListFiles chain
            GistGraphOp::PrepareListFiles => {
                let mut out = HashMap::new();
                out.insert(
                    "request".to_string(),
                    Value::Request(TransportRequest::Shell(ShellRequest {
                        command: "git".to_string(),
                        args: vec!["ls-files".to_string()],
                        cwd: Some(".".to_string()),
                        env: HashMap::new(),
                        stdin: None,
                    })),
                );
                out
            }
            GistGraphOp::ParseListFiles => {
                let mut out = HashMap::new();
                out.insert(
                    "files".to_string(),
                    Value::StrList(vec!["src/main.rs".to_string(), "README.md".to_string()]),
                );
                out
            }

            // ReadFiles chain
            GistGraphOp::PrepareReadFiles => {
                let mut out = HashMap::new();
                out.insert(
                    "request".to_string(),
                    Value::Request(TransportRequest::Shell(ShellRequest {
                        command: "sh".to_string(),
                        args: vec!["-c".to_string(), "echo file contents".to_string()],
                        cwd: None,
                        env: HashMap::new(),
                        stdin: None,
                    })),
                );
                out
            }
            GistGraphOp::ParseReadFiles => {
                let mut out = HashMap::new();
                let mut contents = std::collections::BTreeMap::new();
                contents.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                out.insert("contents".to_string(), Value::MapStrStr(contents));
                out
            }

            // Pure ops
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
            GistGraphOp::Gist(op) => match op {
                GistOps::PrepareRequest { .. } => {
                    let mut out = HashMap::new();
                    out.insert(
                        "request".to_string(),
                        Value::Request(TransportRequest::Shell(ShellRequest {
                            command: "gh".to_string(),
                            args: vec!["gist".to_string(), "create".to_string()],
                            cwd: None,
                            env: HashMap::new(),
                            stdin: None,
                        })),
                    );
                    out
                }
                GistOps::ParseGistResponse => {
                    let mut out = HashMap::new();
                    out.insert(
                        "url".to_string(),
                        Value::Str("https://gist.github.com/mock/123".to_string()),
                    );
                    out
                }
            }

            // Transport boundary
            GistGraphOp::Transport(_) => {
                let mut out = HashMap::new();
                out.insert(
                    "response".to_string(),
                    Value::Response(TransportResponse::Shell(
                        gunbc_ir::transport::ShellResponse {
                            exit_code: 0,
                            stdout: "src/main.rs\nREADME.md\n".to_string(),
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
        // 11 nodes: prepare_list, execute_list, parse_list, filter,
        //           prepare_read, execute_read, parse_read, render,
        //           prepare_gist, execute_gist, parse_gist_response
        assert_eq!(dag.nodes.len(), 11);
        // 10 edges
        assert_eq!(dag.edges.len(), 10);
    }

    #[test]
    fn test_graph_has_transport_boundaries() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");

        // Verify all transport nodes exist
        assert!(dag.get_node(&"execute_list_files".into()).is_some());
        assert!(dag.get_node(&"execute_read_files".into()).is_some());
        assert!(dag.get_node(&"execute_gist".into()).is_some());
    }

    #[test]
    fn test_graph_has_entrypoints() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // repo_path on prepare_list_files and prepare_read_files are entrypoints
        assert!(entrypoints.is_entrypoint_port(&"prepare_list_files".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_files".into(), &"repo_path".into()));
    }

    #[test]
    fn test_graph_structure() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");

        // Should have 11 nodes
        assert_eq!(dag.nodes.len(), 11);

        // Should have 10 edges (pipeline)
        assert_eq!(dag.edges.len(), 10);
    }

    #[test]
    fn test_pure_nodes_not_boundaries() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Pure intermediate nodes should not be boundaries
        // Note: parse_gist_response IS a boundary because it's a terminal node (no outgoing edges)
        assert!(!boundaries.is_boundary_node(&"prepare_list_files".into()));
        assert!(!boundaries.is_boundary_node(&"parse_list_files".into()));
        assert!(!boundaries.is_boundary_node(&"filter_files".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_files".into()));
        assert!(!boundaries.is_boundary_node(&"parse_read_files".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
        // parse_gist_response is a terminal node, so it's a boundary
        assert!(boundaries.is_boundary_node(&"parse_gist_response".into()));
    }

    #[test]
    fn test_transport_nodes_have_correct_ports() {
        let dag = build_gist_graph(vec![], false).expect("graph should build");

        // execute_list_files should have TransportRequest input
        let execute_list = dag.get_node(&"execute_list_files".into()).unwrap();
        assert!(execute_list.inputs.iter().any(|p| p.type_id.0 == "TransportRequest"));
        assert!(execute_list.outputs.iter().any(|p| p.type_id.0 == "TransportResponse"));

        // execute_read_files should have TransportRequest input
        let execute_read = dag.get_node(&"execute_read_files".into()).unwrap();
        assert!(execute_read.inputs.iter().any(|p| p.type_id.0 == "TransportRequest"));
        assert!(execute_read.outputs.iter().any(|p| p.type_id.0 == "TransportResponse"));
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

        // Should have two inputs (repo_path on prepare_list_files and prepare_read_files)
        assert_eq!(inferred.inputs.len(), 2);

        // Should have one output (url from parse_gist_response)
        assert_eq!(inferred.outputs.len(), 1);
        let output_names: Vec<_> = inferred.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"url"));
    }
}
