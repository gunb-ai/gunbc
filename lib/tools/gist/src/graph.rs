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

use gunbc_exec::{
    ExecError, Executable, OutputMap, TransportResponseExt,
    optional_str, optional_str_list,
    require_response, require_str, require_str_list,
};
use gunbc_ir::transport::{ShellRequest, TransportRequest, TransportResponse};
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_git_ops::GitOps;
use gunbc_lib_gist_ops::GistOps;
use gunbc_lib_markdown::MarkdownOp;
use gunbc_lib_transport::TransportOps;
use std::collections::{BTreeMap, HashMap};

/// Gist content acquisition mode.
///
/// Determines how the gist acquires content — either a full snapshot of files
/// in the repo, or a diff of changes against a base branch.
///
/// This is a **build-time** parameter: the mode is known when the tool is
/// invoked (e.g., `make gist` vs `make gist-diff`), not a runtime value
/// computed by the graph.
#[derive(Debug, Clone)]
pub enum GistMode {
    /// Snapshot mode: list files → read contents → render as code blocks.
    ///
    /// The full pipeline:
    /// `ls-files → execute → parse → read-files → execute → parse → render-code → gist`
    Snapshot,

    /// Diff mode: get unified diff against base_ref → render as diff blocks.
    ///
    /// The full pipeline:
    /// `git-diff → execute → parse-diff → render-diff → gist`
    Diff {
        /// The branch to diff against (e.g., "main", "origin/main").
        base_ref: String,
    },
}

/// The operation type for gist graphs - a union of pure ops, library ops, and transport.
///
/// Following the CI pattern: all I/O happens through `Transport(TransportOps::Execute)` nodes.
#[derive(Debug, Clone)]
pub enum GistGraphOp {
    // ========================================================================
    // Git operations (via git-ops crate)
    // ========================================================================
    /// Git operations (PURE - builds requests, parses responses)
    Git(GitOps),

    // ========================================================================
    // ReadFiles chain (batch): PrepareReadFiles -> Execute -> ParseReadFiles
    // Note: This uses a batch approach for efficiency. A LoopBuilder-based
    // approach with per-file reads would provide better observability at the
    // cost of N shell calls instead of 1.
    // ========================================================================
    /// Prepare batch file read request (PURE - no I/O)
    /// Takes file list and repo_path, outputs shell command to read files
    PrepareReadFiles,
    /// Parse batch file read response (PURE - no I/O)
    /// Takes shell response, outputs contents map
    ParseReadFiles,

    // ========================================================================
    // Single-file operations (for LoopBuilder integration)
    // These can be used with LoopBuilder for per-file observability.
    // ========================================================================
    /// Prepare single file read request (PURE - no I/O)
    /// Takes filename and repo_path, outputs shell command to read one file
    PrepareReadFile,
    /// Parse single file read response (PURE - no I/O)
    /// Takes shell response, outputs filename and content
    ParseReadFile,
    /// Collect file results into a map (PURE - no I/O)
    /// Takes list of (filename, content) pairs, outputs Map
    CollectFileContents,

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
            // Git operations (delegated to git-ops crate)
            GistGraphOp::Git(op) => op.execute(inputs),

            // ReadFiles chain - batch (pure)
            GistGraphOp::PrepareReadFiles => execute_prepare_read_files(inputs),
            GistGraphOp::ParseReadFiles => execute_parse_read_files(inputs),

            // Single-file operations (pure)
            GistGraphOp::PrepareReadFile => execute_prepare_read_file(inputs),
            GistGraphOp::ParseReadFile => execute_parse_read_file(inputs),
            GistGraphOp::CollectFileContents => execute_collect_file_contents(inputs),

            // Library ops
            GistGraphOp::Markdown(op) => op.execute(inputs),
            GistGraphOp::Gist(op) => op.execute(inputs),

            // Transport boundary
            GistGraphOp::Transport(op) => op.execute(inputs),
        }
    }
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
    let files = require_str_list(&inputs, "files")?;

    let repo_path = optional_str(&inputs, "repo_path").unwrap_or(".");

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

    OutputMap::new().request("request", request).ok()
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
    let response = require_response(&inputs, "response")?;
    let shell = response.require_shell()?;
    let stdout = shell.stdout.clone();

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

    OutputMap::new().value("contents", Value::str_map(contents)).ok()
}

// ============================================================================
// Single-file operations (for LoopBuilder integration)
// ============================================================================

/// Prepare single file read request (PURE - no I/O).
///
/// This is the per-file version of PrepareReadFiles, designed for use with
/// LoopBuilder. Each iteration reads one file.
///
/// Inputs:
/// - filename: the file to read
/// - repo_path: optional base path (defaults to ".")
///
/// Outputs:
/// - request: TransportRequest (shell command to read one file)
/// - filename: pass through for correlation
fn execute_prepare_read_file(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let filename = require_str(&inputs, "filename")?;

    let repo_path = optional_str(&inputs, "repo_path").unwrap_or(".");

    // Build full path
    let full_path = if repo_path == "." {
        filename.to_string()
    } else {
        format!("{}/{}", repo_path, filename)
    };

    let request = TransportRequest::Shell(ShellRequest {
        command: "cat".to_string(),
        args: vec![full_path],
        cwd: None,
        env: HashMap::new(),
        stdin: None,
    });

    OutputMap::new()
        .request("request", request)
        .str("filename", filename)
        .ok()
}

/// Parse single file read response (PURE - no I/O).
///
/// This is the per-file version of ParseReadFiles, designed for use with
/// LoopBuilder. Each iteration parses one file's content.
///
/// Inputs:
/// - response: TransportResponse from cat command
/// - filename: the filename (for correlation)
///
/// Outputs:
/// - filename: the original filename
/// - content: the file content
fn execute_parse_read_file(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = require_response(&inputs, "response")?;
    let filename = require_str(&inputs, "filename")?;

    let shell = response.require_shell()?;
    let content = if shell.success() {
        shell.stdout.clone()
    } else {
        // Return empty content on failure
        String::new()
    };

    OutputMap::new()
        .str("filename", filename)
        .str("content", content)
        .ok()
}

/// Collect file results into a map (PURE - no I/O).
///
/// This is a post-processing step for LoopBuilder output. It converts
/// a list of (filename, content) pairs into a Map.
///
/// Inputs:
/// - results: list of tuples (from LoopBuilder pack node)
///
/// Outputs:
/// - contents: Map (filename -> content)
fn execute_collect_file_contents(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // For now, this expects a list of filename/content pairs
    // The actual format depends on how LoopBuilder outputs results
    // This is a placeholder for when LoopBuilder integration is complete
    let filenames = optional_str_list(&inputs, "filenames").unwrap_or_default();
    let contents_list = optional_str_list(&inputs, "contents_list").unwrap_or_default();

    let mut contents = BTreeMap::new();
    for (filename, content) in filenames.iter().zip(contents_list.iter()) {
        if !content.is_empty() {
            contents.insert(filename.clone(), content.clone());
        }
    }

    OutputMap::new().value("contents", Value::str_map(contents)).ok()
}

// ============================================================================
// LoopBuilder Integration (Future Enhancement)
// ============================================================================

/// Build a body DAG for single-file read (for LoopBuilder integration).
///
/// This creates a DAG that reads a single file:
/// ```text
/// PrepareReadFile -> Execute -> ParseReadFile
/// ```
///
/// The DAG has:
/// - Input: `filename: String` (from LoopBuilder's unpack node)
/// - Output: `result: String` (content, for LoopBuilder's pack node)
///
/// # Note
///
/// This is provided for documentation and future use. The current graph
/// uses the batch approach (PrepareReadFiles -> Execute -> ParseReadFiles)
/// for efficiency (single shell call instead of N calls).
///
/// When LoopBuilder integration is needed, use this with:
/// ```ignore
/// let body = build_read_file_body_dag();
/// let loop_node = LoopBuilder::new("read_files_loop")
///     .with_input("files", "List", Cardinality::ZERO_OR_MORE)
///     .with_element("filename", "String")
///     .with_body(body)
///     .with_output("contents", "List")
///     .build();
/// ```
pub fn build_read_file_body_dag() -> Dag<GistGraphOp> {
    let mut dag = Dag::new();

    // PrepareReadFile node
    dag.add_node(Node::opaque(
        "prepare",
        vec![port("filename", "String")],
        vec![port("request", "TransportRequest"), port("filename", "String")],
        GistGraphOp::PrepareReadFile,
    ));

    // Execute node
    dag.add_node(Node::opaque(
        "execute",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        GistGraphOp::Transport(TransportOps::Execute),
    ));

    // ParseReadFile node
    dag.add_node(Node::opaque(
        "parse",
        vec![port("response", "TransportResponse"), port("filename", "String")],
        vec![port("filename", "String"), port("content", "String")],
        GistGraphOp::ParseReadFile,
    ));

    // Wire the pipeline
    dag.add_edge(gunbc_ir::Edge::new("prepare", "request", "execute", "request"));
    dag.add_edge(gunbc_ir::Edge::new("execute", "response", "parse", "response"));
    dag.add_edge(gunbc_ir::Edge::new("prepare", "filename", "parse", "filename"));

    dag
}

/// Get the declared signature for the gist workflow.
///
/// The signature adapts to the mode:
/// - Snapshot: `(repo_path?) → url`
/// - Diff: `(repo_path?, base_ref?) → url`
pub fn gist_signature(mode: &GistMode) -> WorkflowSignature {
    let mut sig = WorkflowSignature::new()
        .with_input("repo_path", "String", Cardinality::ZERO_OR_ONE)
        .with_output("url", "String", Cardinality::ONE);

    if matches!(mode, GistMode::Diff { .. }) {
        sig = sig.with_input("base_ref", "String", Cardinality::ZERO_OR_ONE);
    }

    sig
}

/// Build the gist graph using DagBuilder.
///
/// The mode determines the content acquisition strategy:
///
/// **Snapshot mode** (3 boundaries):
/// ```text
/// PrepareLsFiles → Execute → ParseLsFiles → PrepareReadFiles → Execute → ParseReadFiles → RenderCode → PrepareGist → Execute → ParseGistResponse
/// ```
///
/// **Diff mode** (2 boundaries):
/// ```text
/// PrepareDiff → Execute → ParseDiff → RenderDiff → PrepareGist → Execute → ParseGistResponse
/// ```
///
/// Both share the same gist creation tail (PrepareGist → Execute → ParseGistResponse).
/// Extension filtering is handled by git pathspecs (pushed into the git command),
/// not by separate filter nodes.
pub fn build_gist_graph(
    mode: GistMode,
    extensions: Vec<String>,
    public: bool,
) -> Result<Dag<GistGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Content acquisition (mode-dependent)
    // ========================================================================
    // Both modes produce a render_markdown node handle that outputs "markdown".

    let render_markdown = match mode {
        GistMode::Snapshot => {
            build_snapshot_acquire(&mut builder, extensions)?
        }
        GistMode::Diff { base_ref } => {
            build_diff_acquire(&mut builder, &base_ref, extensions)?
        }
    };

    // ========================================================================
    // Shared gist creation tail
    // ========================================================================

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

    // Node: ExecuteGist (BOUNDARY - actual I/O)
    let execute_gist = builder.add_node_after(
        Node::opaque(
            "execute_gist",
            vec![scalar("request", "TransportRequest")],
            vec![scalar("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_gist_request,
    )?;

    // Node: ParseGistResponse (PURE - extracts URL)
    let parse_gist_response = builder.add_node_after(
        Node::opaque(
            "parse_gist_response",
            vec![scalar("response", "TransportResponse")],
            vec![scalar("url", "String")],
            GistGraphOp::Gist(GistOps::ParseGistResponse),
        ),
        &execute_gist,
    )?;

    // Wire gist tail
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

/// Build the snapshot-mode acquisition subgraph.
///
/// Returns the render_markdown node ref (output: "markdown").
fn build_snapshot_acquire(
    builder: &mut DagBuilder<GistGraphOp>,
    extensions: Vec<String>,
) -> Result<gunbc_ir::builder::NodeRef<GistGraphOp>, BuilderError> {
    // Node: PrepareLsFiles (PURE - extensions pushed into git pathspec)
    let prepare_list_files = builder.add_root_node(Node::opaque(
        "prepare_list_files",
        vec![optional("repo_path", "String")],
        vec![port("request", "TransportRequest")],
        GistGraphOp::Git(GitOps::PrepareLsFiles { extensions }),
    ))?;

    // Node: Execute list files (BOUNDARY)
    let execute_list_files = builder.add_node_after(
        Node::opaque(
            "execute_list_files",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_list_files,
    )?;

    // Node: ParseLsFiles (PURE)
    let parse_list_files = builder.add_node_after(
        Node::opaque(
            "parse_list_files",
            vec![port("response", "TransportResponse")],
            vec![list("files", "List")],
            GistGraphOp::Git(GitOps::ParseLsFiles),
        ),
        &execute_list_files,
    )?;

    // Node: PrepareReadFiles (PURE - builds batch read shell command)
    let prepare_read_files = builder.add_node_after(
        Node::opaque(
            "prepare_read_files",
            vec![list("files", "List"), optional("repo_path", "String")],
            vec![port("request", "TransportRequest")],
            GistGraphOp::PrepareReadFiles,
        ),
        &parse_list_files,
    )?;

    // Node: Execute read files (BOUNDARY)
    let execute_read_files = builder.add_node_after(
        Node::opaque(
            "execute_read_files",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_read_files,
    )?;

    // Node: ParseReadFiles (PURE)
    let parse_read_files = builder.add_node_after(
        Node::opaque(
            "parse_read_files",
            vec![port("response", "TransportResponse")],
            vec![list("contents", "Map")],
            GistGraphOp::ParseReadFiles,
        ),
        &execute_read_files,
    )?;

    // Node: RenderCodeSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![list("contents", "Map")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderCodeSnapshot),
        ),
        &parse_read_files,
    )?;

    // Wire snapshot pipeline
    builder.add_edge(
        prepare_list_files.out("request"),
        execute_list_files.in_port("request"),
    )?;
    builder.add_edge(
        execute_list_files.out("response"),
        parse_list_files.in_port("response"),
    )?;
    builder.add_edge(parse_list_files.out("files"), prepare_read_files.in_port("files"))?;
    builder.add_edge(
        prepare_read_files.out("request"),
        execute_read_files.in_port("request"),
    )?;
    builder.add_edge(
        execute_read_files.out("response"),
        parse_read_files.in_port("response"),
    )?;
    builder.add_edge(parse_read_files.out("contents"), render_markdown.in_port("contents"))?;

    Ok(render_markdown)
}

/// Build the diff-mode acquisition subgraph.
///
/// Returns the render_markdown node ref (output: "markdown").
fn build_diff_acquire(
    builder: &mut DagBuilder<GistGraphOp>,
    base_ref: &str,
    extensions: Vec<String>,
) -> Result<gunbc_ir::builder::NodeRef<GistGraphOp>, BuilderError> {
    // Node: PrepareDiff (PURE - extensions pushed into git pathspec)
    let prepare_diff = builder.add_root_node(Node::opaque(
        "prepare_diff",
        vec![
            optional("repo_path", "String"),
            optional("base_ref", "String"),
        ],
        vec![port("request", "TransportRequest")],
        GistGraphOp::Git(GitOps::PrepareDiff {
            base_ref: base_ref.to_string(),
            extensions,
        }),
    ))?;

    // Node: Execute diff (BOUNDARY)
    let execute_diff = builder.add_node_after(
        Node::opaque(
            "execute_diff",
            vec![port("request", "TransportRequest")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_diff,
    )?;

    // Node: ParseDiff (PURE)
    let parse_diff = builder.add_node_after(
        Node::opaque(
            "parse_diff",
            vec![port("response", "TransportResponse")],
            vec![
                list("diff_files", "Map"),
                scalar("stats", "String"),
            ],
            GistGraphOp::Git(GitOps::ParseDiff),
        ),
        &execute_diff,
    )?;

    // Node: RenderDiffSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![
                list("diff_files", "Map"),
                optional("stats", "String"),
            ],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderDiffSnapshot),
        ),
        &parse_diff,
    )?;

    // Wire diff pipeline
    builder.add_edge(
        prepare_diff.out("request"),
        execute_diff.in_port("request"),
    )?;
    builder.add_edge(
        execute_diff.out("response"),
        parse_diff.in_port("response"),
    )?;
    builder.add_edge(
        parse_diff.out("diff_files"),
        render_markdown.in_port("diff_files"),
    )?;
    builder.add_edge(
        parse_diff.out("stats"),
        render_markdown.in_port("stats"),
    )?;

    Ok(render_markdown)
}

// Mockable implementation for test generation
use gunbc_test::Mockable;

impl Mockable for GistGraphOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            // Git operations (delegated)
            GistGraphOp::Git(op) => {
                // Return appropriate mock outputs based on the git op variant
                match op {
                    GitOps::PrepareLsFiles { .. } => {
                        let request = gunbc_ir::transport::git::GitRequest::ls_files()
                            .to_shell_request();
                        OutputMap::new().request("request", request).build()
                    }
                    GitOps::ParseLsFiles => {
                        OutputMap::new()
                            .value("files", Value::str_list(vec![
                                "src/main.rs".to_string(),
                                "README.md".to_string(),
                            ]))
                            .build()
                    }
                    GitOps::PrepareDiff { .. } | GitOps::PrepareDiffNameOnly { .. }
                    | GitOps::PrepareCurrentBranch => {
                        OutputMap::new()
                            .request("request", TransportRequest::Shell(ShellRequest {
                                command: "git".to_string(),
                                args: vec!["mock".to_string()],
                                cwd: None,
                                env: HashMap::new(),
                                stdin: None,
                            }))
                            .build()
                    }
                    GitOps::ParseDiff => {
                        OutputMap::new()
                            .value("diff_files", Value::str_map(std::collections::BTreeMap::new()))
                            .str("stats", "+0 -0 across 0 files")
                            .build()
                    }
                    GitOps::ParseDiffNameOnly => {
                        OutputMap::new().value("files", Value::str_list(vec![])).build()
                    }
                    GitOps::ParseCurrentBranch => {
                        OutputMap::new().str("branch", "main").build()
                    }
                }
            }

            // ReadFiles chain
            GistGraphOp::PrepareReadFiles => {
                OutputMap::new()
                    .request("request", TransportRequest::Shell(ShellRequest {
                        command: "sh".to_string(),
                        args: vec!["-c".to_string(), "echo file contents".to_string()],
                        cwd: None,
                        env: HashMap::new(),
                        stdin: None,
                    }))
                    .build()
            }
            GistGraphOp::ParseReadFiles => {
                let mut contents = std::collections::BTreeMap::new();
                contents.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                OutputMap::new().value("contents", Value::str_map(contents)).build()
            }

            // Single-file operations
            GistGraphOp::PrepareReadFile => {
                OutputMap::new()
                    .request("request", TransportRequest::Shell(ShellRequest {
                        command: "cat".to_string(),
                        args: vec!["src/main.rs".to_string()],
                        cwd: None,
                        env: HashMap::new(),
                        stdin: None,
                    }))
                    .str("filename", "src/main.rs")
                    .build()
            }
            GistGraphOp::ParseReadFile => {
                OutputMap::new()
                    .str("filename", "src/main.rs")
                    .str("content", "fn main() {}")
                    .build()
            }
            GistGraphOp::CollectFileContents => {
                let mut contents = std::collections::BTreeMap::new();
                contents.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                OutputMap::new().value("contents", Value::str_map(contents)).build()
            }

            // Pure ops
            GistGraphOp::Markdown(_) => {
                OutputMap::new()
                    .str("markdown", "# Code Snapshot\n```rust\nfn main() {}\n```")
                    .build()
            }
            GistGraphOp::Gist(op) => match op {
                GistOps::PrepareRequest { .. } => {
                    OutputMap::new()
                        .request("request", TransportRequest::Shell(ShellRequest {
                            command: "gh".to_string(),
                            args: vec!["gist".to_string(), "create".to_string()],
                            cwd: None,
                            env: HashMap::new(),
                            stdin: None,
                        }))
                        .build()
                }
                GistOps::ParseGistResponse => {
                    OutputMap::new()
                        .str("url", "https://gist.github.com/mock/123")
                        .build()
                }
            }

            // Transport boundary
            GistGraphOp::Transport(_) => {
                OutputMap::new()
                    .response("response", TransportResponse::Shell(
                        gunbc_ir::transport::ShellResponse {
                            exit_code: 0,
                            stdout: "src/main.rs\nREADME.md\n".to_string(),
                            stderr: String::new(),
                        },
                    ))
                    .build()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints, infer_signature};

    // ========================================================================
    // Snapshot mode tests
    // ========================================================================

    #[test]
    fn test_snapshot_graph_builds_successfully() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");
        // 10 nodes: prepare_list, execute_list, parse_list,
        //           prepare_read, execute_read, parse_read, render_markdown,
        //           prepare_gist, execute_gist, parse_gist_response
        assert_eq!(dag.nodes.len(), 10);
        // 9 edges
        assert_eq!(dag.edges.len(), 9);
    }

    #[test]
    fn test_snapshot_graph_has_transport_boundaries() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");

        assert!(dag.get_node(&"execute_list_files".into()).is_some());
        assert!(dag.get_node(&"execute_read_files".into()).is_some());
        assert!(dag.get_node(&"execute_gist".into()).is_some());
    }

    #[test]
    fn test_snapshot_graph_has_entrypoints() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints.is_entrypoint_port(&"prepare_list_files".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_read_files".into(), &"repo_path".into()));
    }

    #[test]
    fn test_snapshot_pure_nodes_not_boundaries() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"prepare_list_files".into()));
        assert!(!boundaries.is_boundary_node(&"parse_list_files".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_read_files".into()));
        assert!(!boundaries.is_boundary_node(&"parse_read_files".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
        // parse_gist_response is a terminal node, so it's a boundary
        assert!(boundaries.is_boundary_node(&"parse_gist_response".into()));
    }

    #[test]
    fn test_snapshot_transport_nodes_have_correct_ports() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");

        let execute_list = dag.get_node(&"execute_list_files".into()).unwrap();
        assert!(execute_list.inputs.iter().any(|p| p.type_id.0 == "TransportRequest"));
        assert!(execute_list.outputs.iter().any(|p| p.type_id.0 == "TransportResponse"));

        let execute_read = dag.get_node(&"execute_read_files".into()).unwrap();
        assert!(execute_read.inputs.iter().any(|p| p.type_id.0 == "TransportRequest"));
        assert!(execute_read.outputs.iter().any(|p| p.type_id.0 == "TransportResponse"));
    }

    #[test]
    fn test_snapshot_signature_matches_dag() {
        let mode = GistMode::Snapshot;
        let dag = build_gist_graph(mode.clone(), vec![], false)
            .expect("graph should build");
        let sig = gist_signature(&mode);

        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_snapshot_inferred_signature() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");
        let inferred = infer_signature(&dag);

        // Should have two inputs (repo_path on prepare_list_files and prepare_read_files)
        assert_eq!(inferred.inputs.len(), 2);

        // Should have one output (url from parse_gist_response)
        assert_eq!(inferred.outputs.len(), 1);
        let output_names: Vec<_> = inferred.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"url"));
    }

    // ========================================================================
    // Diff mode tests
    // ========================================================================

    #[test]
    fn test_diff_graph_builds_successfully() {
        let dag = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![],
            false,
        ).expect("diff graph should build");
        // 7 nodes: prepare_diff, execute_diff, parse_diff,
        //          render_markdown, prepare_gist, execute_gist, parse_gist_response
        assert_eq!(dag.nodes.len(), 7);
        // 6 edges (linear pipeline + stats branch to render)
        assert_eq!(dag.edges.len(), 7);
    }

    #[test]
    fn test_diff_graph_has_transport_boundaries() {
        let dag = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![],
            false,
        ).expect("diff graph should build");

        assert!(dag.get_node(&"execute_diff".into()).is_some());
        assert!(dag.get_node(&"execute_gist".into()).is_some());
    }

    #[test]
    fn test_diff_graph_has_entrypoints() {
        let dag = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![],
            false,
        ).expect("diff graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints.is_entrypoint_port(&"prepare_diff".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_diff".into(), &"base_ref".into()));
    }

    #[test]
    fn test_diff_graph_node_ids() {
        let dag = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![".rs".to_string()],
            false,
        ).expect("diff graph should build");

        let expected_nodes = vec![
            "prepare_diff",
            "execute_diff",
            "parse_diff",
            "render_markdown",
            "prepare_gist_request",
            "execute_gist",
            "parse_gist_response",
        ];

        for name in expected_nodes {
            assert!(
                dag.get_node(&name.into()).is_some(),
                "missing node: {}",
                name
            );
        }
    }

    #[test]
    fn test_diff_pure_nodes_not_boundaries() {
        let dag = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![],
            false,
        ).expect("diff graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"prepare_diff".into()));
        assert!(!boundaries.is_boundary_node(&"parse_diff".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
        // parse_gist_response is terminal → boundary
        assert!(boundaries.is_boundary_node(&"parse_gist_response".into()));
    }

    #[test]
    fn test_diff_signature_matches_dag() {
        let mode = GistMode::Diff { base_ref: "main".to_string() };
        let dag = build_gist_graph(mode.clone(), vec![], false)
            .expect("diff graph should build");
        let sig = gist_signature(&mode);

        sig.validate(&dag).expect("diff signature should match DAG");
    }

    #[test]
    fn test_diff_inferred_signature() {
        let dag = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![],
            false,
        ).expect("diff graph should build");
        let inferred = infer_signature(&dag);

        // Should have two inputs (repo_path and base_ref on prepare_diff)
        assert_eq!(inferred.inputs.len(), 2);

        // Should have one output (url from parse_gist_response)
        assert_eq!(inferred.outputs.len(), 1);
        let output_names: Vec<_> = inferred.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"url"));
    }

    // ========================================================================
    // Shared / cross-mode tests
    // ========================================================================

    #[test]
    fn test_both_modes_share_gist_tail() {
        let snap = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("snapshot should build");
        let diff = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![],
            false,
        ).expect("diff should build");

        // Both must have the same gist tail nodes
        for node_id in &["prepare_gist_request", "execute_gist", "parse_gist_response"] {
            assert!(snap.get_node(&(*node_id).into()).is_some(), "snapshot missing {}", node_id);
            assert!(diff.get_node(&(*node_id).into()).is_some(), "diff missing {}", node_id);
        }
    }

    #[test]
    fn test_snapshot_has_no_diff_nodes() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false)
            .expect("graph should build");

        assert!(dag.get_node(&"prepare_diff".into()).is_none());
        assert!(dag.get_node(&"execute_diff".into()).is_none());
    }

    #[test]
    fn test_diff_has_no_snapshot_nodes() {
        let dag = build_gist_graph(
            GistMode::Diff { base_ref: "main".to_string() },
            vec![],
            false,
        ).expect("graph should build");

        assert!(dag.get_node(&"prepare_list_files".into()).is_none());
        assert!(dag.get_node(&"execute_list_files".into()).is_none());
        assert!(dag.get_node(&"execute_read_files".into()).is_none());
    }

    #[test]
    fn test_read_file_body_dag_structure() {
        let dag = build_read_file_body_dag();

        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 3);

        assert!(dag.get_node(&"prepare".into()).is_some());
        assert!(dag.get_node(&"execute".into()).is_some());
        assert!(dag.get_node(&"parse".into()).is_some());
    }
}
