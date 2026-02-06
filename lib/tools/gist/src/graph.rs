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
    optional_str_list_strict, optional_str_strict, propagate_skipped, require_response,
    require_str, ExecError, Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::transport::{ShellRequest, TransportResponse};
use gunbc_ir::patterns::PatternOp;
use gunbc_ir::{
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value, WorkflowSignature,
};
use gunbc_lib_gist_ops::GistOps;
use gunbc_lib_git_ops::GitOps;
use gunbc_lib_markdown::MarkdownOp;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::filename;
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

    /// Recent mode: diff of changes from the last 3 days.
    ///
    /// The full pipeline:
    /// `rev-list → execute → parse-rev-list → diff → execute → parse-diff → render-diff → gist`
    ///
    /// Uses `git rev-list -1 --before="3 days ago" HEAD` to find the base commit,
    /// then diffs against it. If the repo is younger than 3 days (empty rev-list),
    /// the diff runs against HEAD → producing an empty diff.
    Recent,
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
    // Legacy batch approach — retained for potential bulk-read scenarios.
    // ========================================================================
    /// Prepare batch file read request (PURE - no I/O)
    /// Takes file list and repo_path, outputs shell command to read files
    PrepareReadFiles,
    /// Parse batch file read response (PURE - no I/O)
    /// Takes shell response, outputs contents map
    ParseReadFiles,

    // ========================================================================
    // Single-file operations (used by LoopBuilder in snapshot mode)
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
    // Pattern operations (for LoopBuilder integration)
    // ========================================================================
    /// Pattern operations (loop unpack/pack, branch merge, etc.)
    Pattern(PatternOp),

    // ========================================================================
    // Transport boundary (actual I/O)
    // ========================================================================
    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),
}

impl From<PatternOp> for GistGraphOp {
    fn from(op: PatternOp) -> Self {
        GistGraphOp::Pattern(op)
    }
}

impl Executable for GistGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
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

            // Pattern ops (loop unpack/pack, etc.)
            GistGraphOp::Pattern(op) => op.execute(inputs),

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
/// - repo_path: base path
///
/// Outputs:
/// - request: TransportRequest (shell command to read files)
fn execute_prepare_read_files(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if matches!(inputs.get("files"), Some(Value::List(items)) if items.iter().any(|v| matches!(v, Value::Skipped))) {
        return OutputMap::new().value("request", Value::Skipped).ok();
    }
    if let Some(result) = propagate_skipped(&inputs, "files", &["request"]) {
        return result;
    }
    let files = optional_str_list_strict(&inputs, "files")?.unwrap_or_default();

    let repo_path = require_str(&inputs, "repo_path")?;

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

    let request = ShellRequest::new("sh")
        .args(["-c", &script])
        .into_transport_request();

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
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
    if let Some(result) = propagate_skipped(&inputs, "response", &["contents"]) {
        return result;
    }
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

    OutputMap::new().map_str_str("contents", contents).ok()
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
/// - repo_path: base path
///
/// Outputs:
/// - request: TransportRequest (shell command to read one file)
/// - filename: pass through for correlation
fn execute_prepare_read_file(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let filename = require_str(&inputs, "filename")?;
    let repo_path = optional_str_strict(&inputs, "repo_path")?.unwrap_or(".");

    let mut req = ShellRequest::new("cat").arg(filename);
    if repo_path != "." {
        req = req.cwd(repo_path);
    }
    let request = req.into_transport_request();

    OutputMap::new()
        .request("request", request)
        .str("filename", filename)
        .bool("skip", false)
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
    if let Some(result) = propagate_skipped(&inputs, "response", &["filename", "content"]) {
        return result;
    }
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
        .str("result", content)
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
    let filenames = optional_str_list_strict(&inputs, "filenames")?.unwrap_or_default();
    let contents_list = optional_str_list_strict(&inputs, "contents_list")?.unwrap_or_default();

    let mut contents = BTreeMap::new();
    for (filename, content) in filenames.iter().zip(contents_list.iter()) {
        if !content.is_empty() {
            contents.insert(filename.clone(), content.clone());
        }
    }

    OutputMap::new().map_str_str("contents", contents).ok()
}

// ============================================================================
// LoopBuilder body DAG
// ============================================================================

/// Build a body DAG for single-file read (used by LoopBuilder in snapshot mode).
///
/// This creates a DAG that reads a single file:
/// ```text
/// PrepareReadFile -> Execute -> ParseReadFile
/// ```
///
/// The DAG has:
/// - Input: `filename: String` (element from LoopBuilder's unpack node)
/// - Input: `repo_path: String` (optional; defaults to ".")
/// - Output: `result: String` (content, collected by LoopBuilder's pack node)
pub fn build_read_file_body_dag() -> Dag<GistGraphOp> {
    let mut dag = Dag::new();

    // PrepareReadFile node — needs both filename (element) and repo_path (extra input)
    dag.add_node(Node::opaque(
        "prepare",
        vec![port("filename", "String"), port("repo_path", "String")],
        vec![
            port("request", "TransportRequest"),
            port("filename", "String"),
            port("skip", "Bool"),
        ],
        GistGraphOp::PrepareReadFile,
    ));

    // Execute node
    dag.add_node(Node::opaque(
        "execute",
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        vec![port("response", "TransportResponse")],
        GistGraphOp::Transport(TransportOps::Execute),
    ));

    // ParseReadFile node — only outputs "result" (the loop pack collects these)
    dag.add_node(Node::opaque(
        "parse",
        vec![
            port("response", "TransportResponse"),
            port("filename", "String"),
        ],
        vec![port("result", "String")],
        GistGraphOp::ParseReadFile,
    ));

    // Wire the pipeline
    dag.add_edge(gunbc_ir::Edge::new(
        "prepare", "request", "execute", "request",
    ));
    dag.add_edge(gunbc_ir::Edge::new(
        "prepare", "skip", "execute", "skip",
    ));
    dag.add_edge(gunbc_ir::Edge::new(
        "execute", "response", "parse", "response",
    ));
    dag.add_edge(gunbc_ir::Edge::new(
        "prepare", "filename", "parse", "filename",
    ));

    dag
}

/// Get the declared signature for the gist workflow.
///
/// The signature adapts to the mode:
/// - Snapshot: `(repo_path) → url`
/// - Diff: `(repo_path, base_ref?) → url`
pub fn gist_signature(mode: &GistMode) -> WorkflowSignature {
    let mut sig = WorkflowSignature::new()
        .with_input("repo_path", "String", Cardinality::ONE)
        .with_output("url", "String", Cardinality::ONE);

    // base_ref is an entrypoint in snapshot and diff modes (unwired optional on
    // prepare_gist_request / prepare_diff). In recent mode, base_ref is wired
    // from parse_rev_list → not an entrypoint.
    if !matches!(mode, GistMode::Recent) {
        sig = sig.with_input("base_ref", "String", Cardinality::ZERO_OR_ONE);
    }

    sig
}

/// Build the gist graph using DagBuilder.
///
/// The mode determines the content acquisition strategy:
///
/// **Snapshot mode** (uses LoopBuilder for per-file reads):
/// ```text
/// PrepareLsFiles → Execute → ParseLsFiles → LoopBuilder(PrepareReadFile → Execute → ParseReadFile) → CollectFileContents → RenderCode → PrepareGist → Execute → ParseGistResponse
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
    // Environment: filesystem + clock
    // ========================================================================

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs:write", "FilesystemHandle")],
        GistGraphOp::Gist(GistOps::FsEnv {
            scope: filename::Scope::Write,
        }),
    ))?;

    let clock_env = builder.add_root_node(Node::opaque(
        "clock_env",
        vec![],
        vec![port("clock", "Timestamp")],
        GistGraphOp::Gist(GistOps::ClockEnv),
    ))?;

    // ========================================================================
    // Content acquisition (mode-dependent)
    // ========================================================================
    // Both modes produce a render_markdown node handle that outputs "markdown".

    let (render_markdown, recent_parse_rev_list) = match mode {
        GistMode::Snapshot => (build_snapshot_acquire(&mut builder, extensions)?, None),
        GistMode::Diff { base_ref } => {
            (build_diff_acquire(&mut builder, &base_ref, extensions)?, None)
        }
        GistMode::Recent => {
            let (md, rev) = build_recent_acquire(&mut builder, extensions)?;
            (md, Some(rev))
        }
    };

    // ========================================================================
    // Branch name acquisition (parallel to content acquisition)
    // ========================================================================

    // Node: PrepareCurrentBranch (PURE - builds git rev-parse request)
    let prepare_current_branch = builder.add_root_node(Node::opaque(
        "prepare_current_branch",
        vec![port("repo_path", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GistGraphOp::Git(GitOps::PrepareCurrentBranch),
    ))?;

    // Node: ExecuteCurrentBranch (BOUNDARY - actual I/O)
    let execute_current_branch = builder.add_node_after(
        Node::opaque(
            "execute_current_branch",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_current_branch,
    )?;

    // Node: ParseCurrentBranch (PURE - extracts branch name)
    let parse_current_branch = builder.add_node_after(
        Node::opaque(
            "parse_current_branch",
            vec![port("response", "TransportResponse")],
            vec![optional("branch", "String")],
            GistGraphOp::Git(GitOps::ParseCurrentBranch),
        ),
        &execute_current_branch,
    )?;

    // Wire branch acquisition chain
    builder.add_edge(
        prepare_current_branch.out("request"),
        execute_current_branch.in_port("request"),
    )?;
    builder.add_edge(
        prepare_current_branch.out("skip"),
        execute_current_branch.in_port("skip"),
    )?;
    builder.add_edge(
        execute_current_branch.out("response"),
        parse_current_branch.in_port("response"),
    )?;

    // ========================================================================
    // Remote branch resolution (parallel — for detached HEAD)
    // ========================================================================
    // This is a separate question from "what branch are we on?". When HEAD
    // is detached (e.g., `git checkout origin/main`), this chain resolves
    // which remote tracking branch points at the current commit.

    // Node: PrepareRemoteBranches (PURE - builds git branch -r --points-at HEAD request)
    let prepare_remote_branches = builder.add_root_node(Node::opaque(
        "prepare_remote_branches",
        vec![port("repo_path", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GistGraphOp::Git(GitOps::PrepareRemoteBranchesAtHead),
    ))?;

    // Node: ExecuteRemoteBranches (BOUNDARY - actual I/O)
    let execute_remote_branches = builder.add_node_after(
        Node::opaque(
            "execute_remote_branches",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_remote_branches,
    )?;

    // Node: ParseRemoteBranches (PURE - extracts remote branch name)
    let parse_remote_branches = builder.add_node_after(
        Node::opaque(
            "parse_remote_branches",
            vec![port("response", "TransportResponse")],
            vec![optional("remote_branch", "String")],
            GistGraphOp::Git(GitOps::ParseRemoteBranchesAtHead),
        ),
        &execute_remote_branches,
    )?;

    // Wire remote branch chain
    builder.add_edge(
        prepare_remote_branches.out("request"),
        execute_remote_branches.in_port("request"),
    )?;
    builder.add_edge(
        prepare_remote_branches.out("skip"),
        execute_remote_branches.in_port("skip"),
    )?;
    builder.add_edge(
        execute_remote_branches.out("response"),
        parse_remote_branches.in_port("response"),
    )?;

    // ========================================================================
    // Shared gist creation tail
    // ========================================================================

    // Node: PrepareGistRequest (PURE)
    let prepare_gist_request = builder.add_node_after(
        Node::opaque(
            "prepare_gist_request",
            vec![
                scalar("markdown", "String"),
                optional("branch", "String"),
                optional("remote_branch", "String"),
                optional("base_ref", "String"),
                resource("fs", "FilesystemHandle", AccessMode::Read),
                resource("clock", "Timestamp", AccessMode::Read),
            ],
            vec![scalar("request", "TransportRequest"), scalar("skip", "Bool")],
            GistGraphOp::Gist(GistOps::PrepareRequest { public }),
        ),
        &render_markdown,
    )?;

    // Node: ExecuteGist (BOUNDARY - actual I/O)
    let execute_gist = builder.add_node_after(
        Node::opaque(
            "execute_gist",
            vec![scalar("request", "TransportRequest"), scalar("skip", "Bool")],
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
        parse_current_branch.out("branch"),
        prepare_gist_request.in_port("branch"),
    )?;
    builder.add_edge(
        parse_remote_branches.out("remote_branch"),
        prepare_gist_request.in_port("remote_branch"),
    )?;
    builder.add_edge(
        fs_env.out("fs:write"),
        prepare_gist_request.in_port("res:fs"),
    )?;
    builder.add_edge(
        clock_env.out("clock"),
        prepare_gist_request.in_port("res:clock"),
    )?;
    // Wire commit range (recent mode only) so filename reflects the diff range
    if let Some(ref parse_rev_list) = recent_parse_rev_list {
        builder.add_edge(
            parse_rev_list.out("base_ref"),
            prepare_gist_request.in_port("base_ref"),
        )?;
    }
    builder.add_edge(
        prepare_gist_request.out("request"),
        execute_gist.in_port("request"),
    )?;
    builder.add_edge(
        prepare_gist_request.out("skip"),
        execute_gist.in_port("skip"),
    )?;
    builder.add_edge(
        execute_gist.out("response"),
        parse_gist_response.in_port("response"),
    )?;

    let dag = builder.build();
    if let Some(unwired) = gunbc_ir::validate_resource_wiring(&dag).first() {
        return Err(BuilderError::UnwiredResourceInput {
            node: unwired.node.clone(),
            port: unwired.port.clone(),
        });
    }
    Ok(dag)
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
        vec![port("repo_path", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GistGraphOp::Git(GitOps::PrepareLsFiles { extensions }),
    ))?;

    // Node: Execute list files (BOUNDARY)
    let execute_list_files = builder.add_node_after(
        Node::opaque(
            "execute_list_files",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
            vec![list("files", "String")],
            GistGraphOp::Git(GitOps::ParseLsFiles),
        ),
        &execute_list_files,
    )?;

    // Node: LoopBuilder for per-file reading
    use gunbc_ir::patterns::LoopBuilder;

    let body = build_read_file_body_dag();
    let loop_node: Node<GistGraphOp> = LoopBuilder::new("read_files_loop")
        .with_input("files", "String", Cardinality::ZERO_OR_MORE)
        .with_element("filename", "String")
        .with_body(body)
        .with_output("contents", "String")
        .build();

    let read_files_loop = builder.add_node_after(loop_node, &parse_list_files)?;

    // Node: CollectFileContents (PURE - zips filenames + contents into Map)
    let collect_file_contents = builder.add_node_after(
        Node::opaque(
            "collect_file_contents",
            vec![list("filenames", "String"), list("contents_list", "String")],
            vec![port("contents", "Map")],
            GistGraphOp::CollectFileContents,
        ),
        &read_files_loop,
    )?;

    // Node: RenderCodeSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![port("contents", "Map")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderCodeSnapshot),
        ),
        &collect_file_contents,
    )?;

    // Wire snapshot pipeline
    builder.add_edge(
        prepare_list_files.out("request"),
        execute_list_files.in_port("request"),
    )?;
    builder.add_edge(
        prepare_list_files.out("skip"),
        execute_list_files.in_port("skip"),
    )?;
    builder.add_edge(
        execute_list_files.out("response"),
        parse_list_files.in_port("response"),
    )?;
    builder.add_edge(
        parse_list_files.out("files"),
        read_files_loop.in_port("files"),
    )?;
    builder.add_edge(
        parse_list_files.out("files"),
        collect_file_contents.in_port("filenames"),
    )?;
    builder.add_edge(
        read_files_loop.out("contents"),
        collect_file_contents.in_port("contents_list"),
    )?;
    builder.add_edge(
        collect_file_contents.out("contents"),
        render_markdown.in_port("contents"),
    )?;

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
            port("repo_path", "String"),
            optional("base_ref", "String"),
        ],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GistGraphOp::Git(GitOps::PrepareDiff {
            base_ref: base_ref.to_string(),
            extensions,
        }),
    ))?;

    // Node: Execute diff (BOUNDARY)
    let execute_diff = builder.add_node_after(
        Node::opaque(
            "execute_diff",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
            vec![port("diff_files", "Map"), scalar("stats", "String")],
            GistGraphOp::Git(GitOps::ParseDiff),
        ),
        &execute_diff,
    )?;

    // Node: RenderDiffSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![port("diff_files", "Map"), optional("stats", "String")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderDiffSnapshot),
        ),
        &parse_diff,
    )?;

    // Wire diff pipeline
    builder.add_edge(prepare_diff.out("request"), execute_diff.in_port("request"))?;
    builder.add_edge(prepare_diff.out("skip"), execute_diff.in_port("skip"))?;
    builder.add_edge(execute_diff.out("response"), parse_diff.in_port("response"))?;
    builder.add_edge(
        parse_diff.out("diff_files"),
        render_markdown.in_port("diff_files"),
    )?;
    builder.add_edge(parse_diff.out("stats"), render_markdown.in_port("stats"))?;

    Ok(render_markdown)
}

/// Build the recent-mode acquisition subgraph.
///
/// Resolves the commit from 3 days ago via `git rev-list`, then diffs against it.
/// If the repo is younger than 3 days (empty rev-list output), PrepareDiff
/// falls back to its default base_ref of "HEAD", producing an empty diff.
///
/// Returns `(render_markdown, parse_rev_list)` — the caller wires `parse_rev_list`
/// to `prepare_gist_request` so the commit range appears in the gist filename.
fn build_recent_acquire(
    builder: &mut DagBuilder<GistGraphOp>,
    extensions: Vec<String>,
) -> Result<
    (
        gunbc_ir::builder::NodeRef<GistGraphOp>,
        gunbc_ir::builder::NodeRef<GistGraphOp>,
    ),
    BuilderError,
> {
    // ========================================================================
    // Rev-list chain: find commit from 3 days ago
    // ========================================================================

    // Node: PrepareRevListBefore (PURE)
    let prepare_rev_list = builder.add_root_node(Node::opaque(
        "prepare_rev_list",
        vec![port("repo_path", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GistGraphOp::Git(GitOps::PrepareRevListBefore {
            before: "3 days ago".to_string(),
        }),
    ))?;

    // Node: ExecuteRevList (BOUNDARY)
    let execute_rev_list = builder.add_node_after(
        Node::opaque(
            "execute_rev_list",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            vec![port("response", "TransportResponse")],
            GistGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_rev_list,
    )?;

    // Node: ParseRevListBefore (PURE)
    let parse_rev_list = builder.add_node_after(
        Node::opaque(
            "parse_rev_list",
            vec![port("response", "TransportResponse")],
            vec![optional("base_ref", "String")],
            GistGraphOp::Git(GitOps::ParseRevListBefore),
        ),
        &execute_rev_list,
    )?;

    // Wire rev-list chain
    builder.add_edge(
        prepare_rev_list.out("request"),
        execute_rev_list.in_port("request"),
    )?;
    builder.add_edge(
        prepare_rev_list.out("skip"),
        execute_rev_list.in_port("skip"),
    )?;
    builder.add_edge(
        execute_rev_list.out("response"),
        parse_rev_list.in_port("response"),
    )?;

    // ========================================================================
    // Diff chain: diff against the resolved base_ref
    // ========================================================================

    // Node: PrepareDiff (PURE - default base_ref is "HEAD" for young repos)
    let prepare_diff = builder.add_node_after(
        Node::opaque(
            "prepare_diff",
            vec![
                port("repo_path", "String"),
                optional("base_ref", "String"),
            ],
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
            GistGraphOp::Git(GitOps::PrepareDiff {
                base_ref: "HEAD".to_string(),
                extensions,
            }),
        ),
        &parse_rev_list,
    )?;

    // Node: Execute diff (BOUNDARY)
    let execute_diff = builder.add_node_after(
        Node::opaque(
            "execute_diff",
            vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
            vec![port("diff_files", "Map"), scalar("stats", "String")],
            GistGraphOp::Git(GitOps::ParseDiff),
        ),
        &execute_diff,
    )?;

    // Node: RenderDiffSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![port("diff_files", "Map"), optional("stats", "String")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderDiffSnapshot),
        ),
        &parse_diff,
    )?;

    // Wire rev-list → diff (base_ref flows from parse_rev_list to prepare_diff)
    builder.add_edge(
        parse_rev_list.out("base_ref"),
        prepare_diff.in_port("base_ref"),
    )?;

    // Wire diff pipeline
    builder.add_edge(prepare_diff.out("request"), execute_diff.in_port("request"))?;
    builder.add_edge(prepare_diff.out("skip"), execute_diff.in_port("skip"))?;
    builder.add_edge(execute_diff.out("response"), parse_diff.in_port("response"))?;
    builder.add_edge(
        parse_diff.out("diff_files"),
        render_markdown.in_port("diff_files"),
    )?;
    builder.add_edge(parse_diff.out("stats"), render_markdown.in_port("stats"))?;

    Ok((render_markdown, parse_rev_list))
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
                        let request =
                            gunbc_ir::transport::git::GitRequest::ls_files().to_shell_request();
                        OutputMap::new()
                            .request("request", request)
                            .bool("skip", false)
                            .build()
                    }
                    GitOps::ParseLsFiles => OutputMap::new()
                        .str_list(
                            "files",
                            vec!["src/main.rs".to_string(), "README.md".to_string()],
                        )
                        .build(),
                    GitOps::PrepareDiff { .. }
                    | GitOps::PrepareDiffNameOnly { .. }
                    | GitOps::PrepareCurrentBranch
                    | GitOps::PrepareRemoteBranchesAtHead
                    | GitOps::PrepareRevListBefore { .. } => OutputMap::new()
                        .request(
                            "request",
                            ShellRequest::new("git").arg("mock").into_transport_request(),
                        )
                        .bool("skip", false)
                        .build(),
                    GitOps::ParseDiff => OutputMap::new()
                        .map_str_str("diff_files", std::collections::BTreeMap::new())
                        .str("stats", "+0 -0 across 0 files")
                        .build(),
                    GitOps::ParseDiffNameOnly => OutputMap::new().str_list("files", vec![]).build(),
                    GitOps::ParseCurrentBranch => OutputMap::new().str("branch", "main").build(),
                    GitOps::ParseRemoteBranchesAtHead => {
                        OutputMap::new().str("remote_branch", "main").build()
                    }
                    GitOps::ParseRevListBefore => {
                        OutputMap::new().str("base_ref", "abc123def456").build()
                    }
                }
            }

            // ReadFiles chain
            GistGraphOp::PrepareReadFiles => OutputMap::new()
                .request(
                    "request",
                    ShellRequest::new("sh")
                        .args(["-c", "echo file contents"])
                        .into_transport_request(),
                )
                .bool("skip", false)
                .build(),
            GistGraphOp::ParseReadFiles => {
                let mut contents = std::collections::BTreeMap::new();
                contents.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                OutputMap::new().map_str_str("contents", contents).build()
            }

            // Single-file operations
            GistGraphOp::PrepareReadFile => OutputMap::new()
                .request(
                    "request",
                    ShellRequest::new("cat")
                        .arg("src/main.rs")
                        .into_transport_request(),
                )
                .str("filename", "src/main.rs")
                .bool("skip", false)
                .build(),
            GistGraphOp::ParseReadFile => OutputMap::new()
                .str("filename", "src/main.rs")
                .str("result", "fn main() {}")
                .build(),
            GistGraphOp::CollectFileContents => {
                let mut contents = std::collections::BTreeMap::new();
                contents.insert("src/main.rs".to_string(), "fn main() {}".to_string());
                OutputMap::new().map_str_str("contents", contents).build()
            }

            // Pattern ops (mock outputs match what the pattern ops produce)
            GistGraphOp::Pattern(op) => match op {
                PatternOp::LoopUnpack { element_port, .. } => OutputMap::new()
                    .str(element_port, "mock_element")
                    .int("index", 0)
                    .int("count", 1)
                    .build(),
                PatternOp::LoopPack { output_port } => OutputMap::new()
                    .str(output_port, "mock_result")
                    .int("iterations", 1)
                    .build(),
                PatternOp::BranchMerge { output_port } => OutputMap::new()
                    .str(output_port, "mock_merge")
                    .build(),
                _ => HashMap::new(),
            },

            // Pure ops
            GistGraphOp::Markdown(_) => OutputMap::new()
                .str("markdown", "# Code Snapshot\n```rust\nfn main() {}\n```")
                .build(),
            GistGraphOp::Gist(op) => match op {
                GistOps::FsEnv { scope } => {
                    let fs = filename::FilesystemHandle::cross_platform(*scope);
                    let port = match scope {
                        filename::Scope::Read => "fs:read",
                        filename::Scope::Write => "fs:write",
                    };
                    OutputMap::new().value(port, fs.into()).build()
                }
                GistOps::ClockEnv => {
                    let ts =
                        gunbc_ir::Timestamp::from_system_time(std::time::SystemTime::UNIX_EPOCH);
                    OutputMap::new().value("clock", ts.into()).build()
                }
                GistOps::PrepareRequest { .. } => OutputMap::new()
                    .request(
                        "request",
                        ShellRequest::new("gh")
                            .args(["gist", "create"])
                            .into_transport_request(),
                    )
                    .bool("skip", false)
                    .build(),
                GistOps::ParseGistResponse => OutputMap::new()
                    .str("url", "https://gist.github.com/mock/123")
                    .build(),
            },

            // Transport boundary
            GistGraphOp::Transport(_) => OutputMap::new()
                .response(
                    "response",
                    TransportResponse::Shell(gunbc_ir::transport::ShellResponse::ok("src/main.rs\nREADME.md\n")),
                )
                .build(),
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
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");
        // 17 nodes: fs_env, clock_env, prepare_list, execute_list, parse_list,
        //           read_files_loop (SubDag), collect_file_contents, render_markdown,
        //           prepare_current_branch, execute_current_branch, parse_current_branch,
        //           prepare_remote_branches, execute_remote_branches, parse_remote_branches,
        //           prepare_gist, execute_gist, parse_gist_response
        assert_eq!(dag.nodes.len(), 17);
        // 21 edges across snapshot, branch, remote branch, and gist tail wiring
        assert_eq!(dag.edges.len(), 21);
    }

    #[test]
    fn test_snapshot_graph_has_transport_boundaries() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");

        assert!(dag.get_node(&"execute_list_files".into()).is_some());
        assert!(dag.get_node(&"read_files_loop".into()).is_some());
        assert!(dag.get_node(&"execute_current_branch".into()).is_some());
        assert!(dag.get_node(&"execute_remote_branches".into()).is_some());
        assert!(dag.get_node(&"execute_gist".into()).is_some());
    }

    #[test]
    fn test_snapshot_graph_has_entrypoints() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints.is_entrypoint_port(&"prepare_list_files".into(), &"repo_path".into()));
        assert!(
            entrypoints.is_entrypoint_port(&"prepare_current_branch".into(), &"repo_path".into())
        );
        assert!(
            entrypoints
                .is_entrypoint_port(&"prepare_remote_branches".into(), &"repo_path".into())
        );
    }

    #[test]
    fn test_snapshot_pure_nodes_not_boundaries() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"prepare_list_files".into()));
        assert!(!boundaries.is_boundary_node(&"parse_list_files".into()));
        assert!(!boundaries.is_boundary_node(&"collect_file_contents".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_current_branch".into()));
        assert!(!boundaries.is_boundary_node(&"parse_current_branch".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_remote_branches".into()));
        assert!(!boundaries.is_boundary_node(&"parse_remote_branches".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
        // parse_gist_response is a terminal node, so it's a boundary
        assert!(boundaries.is_boundary_node(&"parse_gist_response".into()));
    }

    #[test]
    fn test_snapshot_transport_nodes_have_correct_ports() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");

        let execute_list = dag.get_node(&"execute_list_files".into()).unwrap();
        assert!(execute_list
            .inputs
            .iter()
            .any(|p| p.type_id.0 == "TransportRequest"));
        assert!(execute_list
            .outputs
            .iter()
            .any(|p| p.type_id.0 == "TransportResponse"));

        // read_files_loop is a SubDag (LoopBuilder) — transport node is inside
    }

    #[test]
    fn test_snapshot_signature_matches_dag() {
        let mode = GistMode::Snapshot;
        let dag = build_gist_graph(mode.clone(), vec![], false).expect("graph should build");
        let sig = gist_signature(&mode);

        sig.validate(&dag).expect("signature should match DAG");
    }

    #[test]
    fn test_snapshot_inferred_signature() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");
        let inferred = infer_signature(&dag);

        // Should have five inputs (repo_path on prepare_list_files,
        // read_files_loop, prepare_current_branch, prepare_remote_branches,
        // and base_ref on prepare_gist_request)
        assert_eq!(inferred.inputs.len(), 5);

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
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![],
            false,
        )
        .expect("diff graph should build");
        // 15 nodes: fs_env, clock_env, prepare_diff, execute_diff, parse_diff,
        //           render_markdown,
        //           prepare_current_branch, execute_current_branch, parse_current_branch,
        //           prepare_remote_branches, execute_remote_branches, parse_remote_branches,
        //           prepare_gist, execute_gist, parse_gist_response
        assert_eq!(dag.nodes.len(), 15);
        // 19 edges across diff, branch, remote branch, and gist tail wiring
        assert_eq!(dag.edges.len(), 19);
    }

    #[test]
    fn test_diff_graph_has_transport_boundaries() {
        let dag = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![],
            false,
        )
        .expect("diff graph should build");

        assert!(dag.get_node(&"execute_diff".into()).is_some());
        assert!(dag.get_node(&"execute_current_branch".into()).is_some());
        assert!(dag.get_node(&"execute_remote_branches".into()).is_some());
        assert!(dag.get_node(&"execute_gist".into()).is_some());
    }

    #[test]
    fn test_diff_graph_has_entrypoints() {
        let dag = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![],
            false,
        )
        .expect("diff graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints.is_entrypoint_port(&"prepare_diff".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"prepare_diff".into(), &"base_ref".into()));
        assert!(
            entrypoints.is_entrypoint_port(&"prepare_current_branch".into(), &"repo_path".into())
        );
        assert!(
            entrypoints
                .is_entrypoint_port(&"prepare_remote_branches".into(), &"repo_path".into())
        );
    }

    #[test]
    fn test_diff_graph_node_ids() {
        let dag = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![".rs".to_string()],
            false,
        )
        .expect("diff graph should build");

        let expected_nodes = vec![
            "prepare_diff",
            "execute_diff",
            "parse_diff",
            "render_markdown",
            "prepare_current_branch",
            "execute_current_branch",
            "parse_current_branch",
            "prepare_remote_branches",
            "execute_remote_branches",
            "parse_remote_branches",
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
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![],
            false,
        )
        .expect("diff graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"prepare_diff".into()));
        assert!(!boundaries.is_boundary_node(&"parse_diff".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_current_branch".into()));
        assert!(!boundaries.is_boundary_node(&"parse_current_branch".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_remote_branches".into()));
        assert!(!boundaries.is_boundary_node(&"parse_remote_branches".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
        // parse_gist_response is terminal → boundary
        assert!(boundaries.is_boundary_node(&"parse_gist_response".into()));
    }

    #[test]
    fn test_diff_signature_matches_dag() {
        let mode = GistMode::Diff {
            base_ref: "main".to_string(),
        };
        let dag = build_gist_graph(mode.clone(), vec![], false).expect("diff graph should build");
        let sig = gist_signature(&mode);

        sig.validate(&dag).expect("diff signature should match DAG");
    }

    #[test]
    fn test_diff_inferred_signature() {
        let dag = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![],
            false,
        )
        .expect("diff graph should build");
        let inferred = infer_signature(&dag);

        // Should have five inputs (repo_path and base_ref on prepare_diff,
        // repo_path on prepare_current_branch, repo_path on prepare_remote_branches,
        // base_ref on prepare_gist_request)
        assert_eq!(inferred.inputs.len(), 5);

        // Should have one output (url from parse_gist_response)
        assert_eq!(inferred.outputs.len(), 1);
        let output_names: Vec<_> = inferred.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(output_names.contains(&"url"));
    }

    // ========================================================================
    // Shared / cross-mode tests
    // ========================================================================

    #[test]
    fn test_all_modes_share_gist_tail() {
        let snap =
            build_gist_graph(GistMode::Snapshot, vec![], false).expect("snapshot should build");
        let diff = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![],
            false,
        )
        .expect("diff should build");
        let recent =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent should build");

        // All must have the same gist tail + branch/remote acquisition nodes
        for node_id in &[
            "prepare_current_branch",
            "execute_current_branch",
            "parse_current_branch",
            "prepare_remote_branches",
            "execute_remote_branches",
            "parse_remote_branches",
            "prepare_gist_request",
            "execute_gist",
            "parse_gist_response",
        ] {
            assert!(
                snap.get_node(&(*node_id).into()).is_some(),
                "snapshot missing {}",
                node_id
            );
            assert!(
                diff.get_node(&(*node_id).into()).is_some(),
                "diff missing {}",
                node_id
            );
            assert!(
                recent.get_node(&(*node_id).into()).is_some(),
                "recent missing {}",
                node_id
            );
        }
    }

    #[test]
    fn test_snapshot_has_no_diff_nodes() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");

        assert!(dag.get_node(&"prepare_diff".into()).is_none());
        assert!(dag.get_node(&"execute_diff".into()).is_none());
    }

    #[test]
    fn test_diff_has_no_snapshot_nodes() {
        let dag = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            vec![],
            false,
        )
        .expect("graph should build");

        assert!(dag.get_node(&"prepare_list_files".into()).is_none());
        assert!(dag.get_node(&"execute_list_files".into()).is_none());
        assert!(dag.get_node(&"read_files_loop".into()).is_none());
    }

    // ========================================================================
    // Recent mode tests
    // ========================================================================

    #[test]
    fn test_recent_graph_builds_successfully() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");
        // 18 nodes: fs_env, clock_env,
        //           prepare_rev_list, execute_rev_list, parse_rev_list,
        //           prepare_diff, execute_diff, parse_diff, render_markdown,
        //           prepare_current_branch, execute_current_branch, parse_current_branch,
        //           prepare_remote_branches, execute_remote_branches, parse_remote_branches,
        //           prepare_gist, execute_gist, parse_gist_response
        assert_eq!(dag.nodes.len(), 18);
        // 24 edges: 3 (rev-list chain) + 1 (rev-list→diff) + 5 (diff chain)
        //         + 2 (branch chain) + 2 (remote chain)
        //         + 7 (gist tail: markdown→gist, branch→gist, remote→gist, fs→gist, clock→gist, gist→execute, execute→parse)
        //         + 1 (parse_rev_list→prepare_gist_request base_ref)
        //         + 3 (skip wiring for skippable transports)
        assert_eq!(dag.edges.len(), 24);
    }

    #[test]
    fn test_recent_graph_has_transport_boundaries() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        assert!(dag.get_node(&"execute_rev_list".into()).is_some());
        assert!(dag.get_node(&"execute_diff".into()).is_some());
        assert!(dag.get_node(&"execute_current_branch".into()).is_some());
        assert!(dag.get_node(&"execute_remote_branches".into()).is_some());
        assert!(dag.get_node(&"execute_gist".into()).is_some());
    }

    #[test]
    fn test_recent_graph_has_entrypoints() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(
            entrypoints.is_entrypoint_port(&"prepare_rev_list".into(), &"repo_path".into())
        );
        assert!(
            entrypoints.is_entrypoint_port(&"prepare_current_branch".into(), &"repo_path".into())
        );
        assert!(
            entrypoints
                .is_entrypoint_port(&"prepare_remote_branches".into(), &"repo_path".into())
        );
    }

    #[test]
    fn test_recent_graph_node_ids() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        let expected_nodes = vec![
            "prepare_rev_list",
            "execute_rev_list",
            "parse_rev_list",
            "prepare_diff",
            "execute_diff",
            "parse_diff",
            "render_markdown",
            "prepare_current_branch",
            "execute_current_branch",
            "parse_current_branch",
            "prepare_remote_branches",
            "execute_remote_branches",
            "parse_remote_branches",
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
    fn test_recent_pure_nodes_not_boundaries() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");
        let boundaries = detect_boundaries(&dag);

        assert!(!boundaries.is_boundary_node(&"prepare_rev_list".into()));
        assert!(!boundaries.is_boundary_node(&"parse_rev_list".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_diff".into()));
        assert!(!boundaries.is_boundary_node(&"parse_diff".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        assert!(!boundaries.is_boundary_node(&"prepare_gist_request".into()));
        // parse_gist_response is terminal → boundary
        assert!(boundaries.is_boundary_node(&"parse_gist_response".into()));
    }

    #[test]
    fn test_recent_signature_matches_dag() {
        let mode = GistMode::Recent;
        let dag = build_gist_graph(mode.clone(), vec![], false).expect("recent graph should build");
        let sig = gist_signature(&mode);

        sig.validate(&dag)
            .expect("recent signature should match DAG");
    }

    #[test]
    fn test_recent_has_no_snapshot_nodes() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        assert!(dag.get_node(&"prepare_list_files".into()).is_none());
        assert!(dag.get_node(&"execute_list_files".into()).is_none());
        assert!(dag.get_node(&"read_files_loop".into()).is_none());
    }

    #[test]
    fn test_recent_has_rev_list_nodes() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        assert!(dag.get_node(&"prepare_rev_list".into()).is_some());
        assert!(dag.get_node(&"execute_rev_list".into()).is_some());
        assert!(dag.get_node(&"parse_rev_list".into()).is_some());
    }

    #[test]
    fn test_loop_builder_with_gist_ops() {
        use gunbc_ir::patterns::LoopBuilder;

        let body = build_read_file_body_dag();
        let node: Node<GistGraphOp> = LoopBuilder::new("read_files_loop")
            .with_input("files", "String", Cardinality::ZERO_OR_MORE)
            .with_element("filename", "String")
            .with_body(body)
            .with_output("contents", "String")
            .build();

        assert!(node.is_subdag());
        assert!(node.inputs.iter().any(|p| p.name.0 == "files"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "repo_path"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "contents"));
    }

    #[test]
    fn test_read_file_body_dag_structure() {
        let dag = build_read_file_body_dag();

        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 4);

        assert!(dag.get_node(&"prepare".into()).is_some());
        assert!(dag.get_node(&"execute".into()).is_some());
        assert!(dag.get_node(&"parse".into()).is_some());
    }
}
