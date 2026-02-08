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
use gunbc_ir::transport::gist::GistRequest;
use gunbc_ir::transport::{ShellRequest, TransportResponse};
use gunbc_ir::patterns::PatternOp;
use gunbc_ir::{
    add_transport_triplet,
    build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, NodeBody, Value,
    WorkflowSignature,
};
use gunbc_lib_cloud_ops::{
    build_cloud_secret_manager_credential_graph_gcp_github, CloudEnv, CloudOps,
    CloudSecretManagerGraphOp,
};
use gunbc_lib_gist_ops::GistOps;
use gunbc_lib_git_ops::GitOps;
use gunbc_lib_markdown::MarkdownOp;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, ClockEnv, FsEnv};
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
    // Environment ops (resource acquisition)
    // ========================================================================
    /// Filesystem environment (resource acquisition)
    FsEnv(FsEnv),
    /// Clock environment (timestamp snapshot)
    ClockEnv(ClockEnv),
    /// Cloud environment (config + runtime credential inputs)
    CloudEnv(CloudEnv),
    /// Cloud credential lifecycle operations
    Cloud(CloudSecretManagerGraphOp),
    /// Resolve auth contract for gist actions.
    ResolveAuth,

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

            // Environment ops (resource acquisition)
            GistGraphOp::FsEnv(op) => op.execute(inputs),
            GistGraphOp::ClockEnv(op) => op.execute(inputs),
            GistGraphOp::CloudEnv(op) => op.execute(inputs),
            GistGraphOp::Cloud(op) => op.execute(inputs),
            GistGraphOp::ResolveAuth => execute_resolve_auth(inputs),

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

/// Resolve auth requirements from the gist interface scope contract.
///
/// This is intentionally strict: credentialed actions without a valid scope
/// contract fail before any transport execute node can run.
fn execute_resolve_auth(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let intent = GistRequest::new().credential_intent();
    intent
        .validate()
        .map_err(|e| ExecError::new(format!("invalid gist credential contract: {e}")))?;

    OutputMap::new()
        .str("service", intent.service)
        .str("scheme", intent.scheme)
        .str("header_name", intent.header_name)
        .str_list("required_scopes", intent.required_scopes)
        .int("lifetime_seconds", 3600)
        .ok()
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
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("fs", "FilesystemHandle", AccessMode::Read),
        ],
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

    // base_ref is an entrypoint only in diff mode.
    if matches!(mode, GistMode::Diff { .. }) {
        sig = sig.with_input("base_ref", "OptionalString", Cardinality::ZERO_OR_ONE);
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
    // Environment: filesystem + clock + cloud credential context
    // ========================================================================

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port("fs:write", "FilesystemHandle")],
        GistGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
    ))?;

    let clock_env = builder.add_root_node(Node::opaque(
        "clock_env",
        vec![],
        vec![port("clock", "Timestamp")],
        GistGraphOp::ClockEnv(ClockEnv),
    ))?;

    let cloud_env = builder.add_root_node(Node::opaque(
        "cloud_env",
        vec![],
        vec![
            port("config", "CloudSecretConfig"),
            optional("request_url", "OptionalString"),
            optional("request_token", "OptionalString"),
        ],
        GistGraphOp::CloudEnv(CloudEnv::new()),
    ))?;

    let resolve_auth = builder.add_root_node(Node::opaque(
        "resolve_auth",
        vec![],
        vec![
            port("service", "String"),
            port("scheme", "String"),
            port("header_name", "String"),
            list("required_scopes", "String"),
            optional("lifetime_seconds", "OptionalInt"),
        ],
        GistGraphOp::ResolveAuth,
    ))?;

    let bind_secret = builder.add_node_after_all(
        Node::opaque(
            "bind_secret",
            vec![port("config", "CloudSecretConfig"), port("service", "String")],
            vec![port("config", "CloudSecretConfig")],
            GistGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::BindSecretName)),
        ),
        &[&cloud_env, &resolve_auth],
    )?;

    let cloud_subdag = lift_cloud_dag(build_cloud_secret_manager_credential_graph_gcp_github());
    let cloud_credential = builder
        .add_node_after(Node::subdag("cloud_credential", cloud_subdag), &bind_secret)?;

    // ========================================================================
    // Content acquisition (mode-dependent)
    // ========================================================================
    // Both modes produce a render_markdown node handle that outputs "markdown".

    let is_recent_mode = matches!(mode, GistMode::Recent);
    let (render_markdown, recent_parse_rev_list) = match &mode {
        GistMode::Snapshot => (build_snapshot_acquire(&mut builder, &fs_env, extensions)?, None),
        GistMode::Diff { base_ref } => {
            (build_diff_acquire(&mut builder, &fs_env, base_ref, extensions)?, None)
        }
        GistMode::Recent => {
            let (md, rev) = build_recent_acquire(&mut builder, &fs_env, extensions)?;
            (md, Some(rev))
        }
    };

    // ========================================================================
    // Branch name acquisition (parallel to content acquisition)
    // ========================================================================

    let current_branch = add_transport_triplet(
        &mut builder,
        "current_branch",
        vec![port("repo_path", "String")],
        vec![resource("fs", "FilesystemHandle", AccessMode::Read)],
        vec![optional("branch", "OptionalString")],
        GistGraphOp::Git(GitOps::PrepareCurrentBranch),
        GistGraphOp::Git(GitOps::ParseCurrentBranch),
        GistGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    // ========================================================================
    // Remote branch resolution (parallel — for detached HEAD)
    // ========================================================================

    let remote_branches = add_transport_triplet(
        &mut builder,
        "remote_branches",
        vec![port("repo_path", "String")],
        vec![resource("fs", "FilesystemHandle", AccessMode::Read)],
        vec![optional("remote_branch", "OptionalString")],
        GistGraphOp::Git(GitOps::PrepareRemoteBranchesAtHead),
        GistGraphOp::Git(GitOps::ParseRemoteBranchesAtHead),
        GistGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    // ========================================================================
    // Shared gist creation tail
    // ========================================================================

    // Node: PrepareGistRequest (PURE)
    let mut gist_prepare_inputs = vec![
        scalar("markdown", "String"),
        optional("branch", "OptionalString"),
        optional("remote_branch", "OptionalString"),
        resource("fs", "FilesystemHandle", AccessMode::Read),
        resource("clock", "Timestamp", AccessMode::Read),
        optional("credential_expires_in", "OptionalInt"),
        list("required_scopes", "String"),
    ];
    if is_recent_mode {
        gist_prepare_inputs.push(optional("base_ref", "OptionalString"));
    }

    let prepare_gist_request = builder.add_node_after(
        Node::opaque(
            "prepare_gist_request",
            gist_prepare_inputs,
            vec![scalar("request", "TransportRequest"), scalar("skip", "Bool")],
            GistGraphOp::Gist(GistOps::PrepareRequest { public }),
        ),
        &render_markdown,
    )?;

    // Node: ExecuteGist (BOUNDARY - actual I/O)
    let execute_gist = builder.add_node_after(
        Node::opaque(
            "execute_gist",
            vec![
                scalar("request", "TransportRequest"),
                scalar("skip", "Bool"),
                resource("credential", "Credential", AccessMode::Read),
            ],
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
        current_branch.parse.out("branch"),
        prepare_gist_request.in_port("branch"),
    )?;
    builder.add_edge(
        remote_branches.parse.out("remote_branch"),
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
    builder.add_edge(cloud_env.out("config"), bind_secret.in_port("config"))?;
    builder.add_edge(resolve_auth.out("service"), bind_secret.in_port("service"))?;
    builder.add_edge(
        bind_secret.out("config"),
        cloud_credential.in_port("config"),
    )?;
    builder.add_edge(
        resolve_auth.out("service"),
        cloud_credential.in_port("source_id"),
    )?;
    builder.add_edge(
        resolve_auth.out("scheme"),
        cloud_credential.in_port("scheme"),
    )?;
    builder.add_edge(
        resolve_auth.out("header_name"),
        cloud_credential.in_port("header_name"),
    )?;
    builder.add_edge(
        resolve_auth.out("lifetime_seconds"),
        cloud_credential.in_port("lifetime_seconds"),
    )?;
    builder.add_edge(
        cloud_env.out("request_url"),
        cloud_credential.in_port("request_url"),
    )?;
    builder.add_edge(
        cloud_credential.out("expires_in"),
        prepare_gist_request.in_port("credential_expires_in"),
    )?;
    builder.add_edge(
        resolve_auth.out("required_scopes"),
        prepare_gist_request.in_port("required_scopes"),
    )?;
    builder.add_edge(
        cloud_env.out("request_token"),
        cloud_credential.in_port("request_token"),
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
        cloud_credential.out("credential"),
        execute_gist.in_port("res:credential"),
    )?;
    builder.add_edge(
        execute_gist.out("response"),
        parse_gist_response.in_port("response"),
    )?;

    // Resource wiring
    builder.add_edge(fs_env.out("fs:write"), current_branch.execute.in_port("res:fs"))?;
    builder.add_edge(fs_env.out("fs:write"), remote_branches.execute.in_port("res:fs"))?;

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
    fs_env: &gunbc_ir::builder::NodeRef<GistGraphOp>,
    extensions: Vec<String>,
) -> Result<gunbc_ir::builder::NodeRef<GistGraphOp>, BuilderError> {
    let list_files = add_transport_triplet(
        builder,
        "list_files",
        vec![port("repo_path", "String")],
        vec![resource("fs", "FilesystemHandle", AccessMode::Read)],
        vec![list("files", "String")],
        GistGraphOp::Git(GitOps::PrepareLsFiles { extensions }),
        GistGraphOp::Git(GitOps::ParseLsFiles),
        GistGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    builder.add_edge(
        fs_env.out("fs:write"),
        list_files.execute.in_port("res:fs"),
    )?;

    // Node: LoopBuilder for per-file reading
    use gunbc_ir::patterns::{LoopBuilder, ResourceInput};

    let body = build_read_file_body_dag();
    let loop_node: Node<GistGraphOp> = LoopBuilder::new("read_files_loop")
        .with_input("files", "String", Cardinality::ZERO_OR_MORE)
        .with_element("filename", "String")
        .with_resource_input(ResourceInput::new("res:fs", "FilesystemHandle"))
        .with_body(body)
        .with_output("contents", "String")
        .build();

    let read_files_loop = builder.add_node_after(loop_node, &list_files.parse)?;
    builder.add_edge(fs_env.out("fs:write"), read_files_loop.in_port("res:fs"))?;

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

    // Wire snapshot pipeline (internal triplet edges handled by helper)
    builder.add_edge(
        list_files.parse.out("files"),
        read_files_loop.in_port("files"),
    )?;
    builder.add_edge(
        list_files.parse.out("files"),
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
    fs_env: &gunbc_ir::builder::NodeRef<GistGraphOp>,
    base_ref: &str,
    extensions: Vec<String>,
) -> Result<gunbc_ir::builder::NodeRef<GistGraphOp>, BuilderError> {
    let diff = add_transport_triplet(
        builder,
        "diff",
        vec![
            port("repo_path", "String"),
            optional("base_ref", "OptionalString"),
        ],
        vec![resource("fs", "FilesystemHandle", AccessMode::Read)],
        vec![port("diff_files", "Map"), scalar("stats", "String")],
        GistGraphOp::Git(GitOps::PrepareDiff {
            base_ref: base_ref.to_string(),
            extensions,
        }),
        GistGraphOp::Git(GitOps::ParseDiff),
        GistGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    builder.add_edge(fs_env.out("fs:write"), diff.execute.in_port("res:fs"))?;

    // Node: RenderDiffSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![port("diff_files", "Map"), optional("stats", "OptionalString")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderDiffSnapshot),
        ),
        &diff.parse,
    )?;

    // Wire diff → render (internal triplet edges handled by helper)
    builder.add_edge(
        diff.parse.out("diff_files"),
        render_markdown.in_port("diff_files"),
    )?;
    builder.add_edge(diff.parse.out("stats"), render_markdown.in_port("stats"))?;

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
    fs_env: &gunbc_ir::builder::NodeRef<GistGraphOp>,
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

    let rev_list = add_transport_triplet(
        builder,
        "rev_list",
        vec![port("repo_path", "String")],
        vec![resource("fs", "FilesystemHandle", AccessMode::Read)],
        vec![optional("base_ref", "OptionalString")],
        GistGraphOp::Git(GitOps::PrepareRevListBefore {
            before: "3 days ago".to_string(),
        }),
        GistGraphOp::Git(GitOps::ParseRevListBefore),
        GistGraphOp::Transport(TransportOps::Execute),
        None,
    )?;

    builder.add_edge(fs_env.out("fs:write"), rev_list.execute.in_port("res:fs"))?;

    // ========================================================================
    // Diff chain: diff against the resolved base_ref
    // ========================================================================

    let diff = add_transport_triplet(
        builder,
        "diff",
        vec![
            port("repo_path", "String"),
            optional("base_ref", "OptionalString"),
        ],
        vec![resource("fs", "FilesystemHandle", AccessMode::Read)],
        vec![port("diff_files", "Map"), scalar("stats", "String")],
        GistGraphOp::Git(GitOps::PrepareDiff {
            base_ref: "HEAD".to_string(),
            extensions,
        }),
        GistGraphOp::Git(GitOps::ParseDiff),
        GistGraphOp::Transport(TransportOps::Execute),
        Some(&rev_list.parse),
    )?;

    builder.add_edge(fs_env.out("fs:write"), diff.execute.in_port("res:fs"))?;

    // Node: RenderDiffSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![port("diff_files", "Map"), optional("stats", "OptionalString")],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderDiffSnapshot),
        ),
        &diff.parse,
    )?;

    // Wire cross-triplet edges (internal triplet edges handled by helpers)
    builder.add_edge(
        rev_list.parse.out("base_ref"),
        diff.prepare.in_port("base_ref"),
    )?;
    builder.add_edge(
        diff.parse.out("diff_files"),
        render_markdown.in_port("diff_files"),
    )?;
    builder.add_edge(diff.parse.out("stats"), render_markdown.in_port("stats"))?;

    Ok((render_markdown, rev_list.parse))
}

fn lift_cloud_dag(
    dag: Dag<CloudSecretManagerGraphOp>,
) -> Dag<GistGraphOp> {
    let mut lift = |op| GistGraphOp::Cloud(op);
    map_dag_ops(dag, &mut lift)
}

fn map_dag_ops<T, U, F>(dag: Dag<T>, f: &mut F) -> Dag<U>
where
    T: Clone,
    U: Clone,
    F: FnMut(T) -> U,
{
    let mut out = Dag::new();
    out.edges = dag.edges.clone();
    out.nodes = dag
        .nodes
        .into_iter()
        .map(|node| map_node_ops(node, f))
        .collect();
    out
}

fn map_node_ops<T, U, F>(node: Node<T>, f: &mut F) -> Node<U>
where
    T: Clone,
    U: Clone,
    F: FnMut(T) -> U,
{
    let Node {
        id,
        inputs,
        outputs,
        body,
        examples,
    } = node;
    let body = match body {
        NodeBody::Opaque(op) => NodeBody::Opaque(f(op)),
        NodeBody::SubDag(subdag) => NodeBody::SubDag(map_dag_ops(subdag, f)),
    };
    Node {
        id,
        inputs,
        outputs,
        body,
        examples,
    }
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

            // Environment ops
            GistGraphOp::FsEnv(op) => op.mock_outputs(),
            GistGraphOp::ClockEnv(op) => op.mock_outputs(),
            GistGraphOp::CloudEnv(op) => op.mock_outputs(),
            GistGraphOp::Cloud(_) => OutputMap::new()
                .value(
                    "credential",
                    Value::Map(std::collections::BTreeMap::from([
                        (
                            "token".to_string(),
                            Value::Secret(gunbc_ir::SecretString::new("<MOCK_GITHUB_TOKEN>")),
                        ),
                        ("source_type".to_string(), Value::Str("static".to_string())),
                        ("scheme".to_string(), Value::Str("bearer".to_string())),
                        (
                            "cap".to_string(),
                            Value::Secret(gunbc_ir::SecretString::new("capability")),
                        ),
                    ])),
                )
                .int("expires_in", 3600)
                .build(),
            GistGraphOp::ResolveAuth => OutputMap::new()
                .str("service", "github")
                .str("scheme", "bearer")
                .str("header_name", "")
                .str_list("required_scopes", vec!["gist:write".to_string()])
                .int("lifetime_seconds", 3600)
                .build(),

            // Pure ops
            GistGraphOp::Markdown(_) => OutputMap::new()
                .str("markdown", "# Code Snapshot\n```rust\nfn main() {}\n```")
                .build(),
            GistGraphOp::Gist(op) => match op {
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
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    // ========================================================================
    // Snapshot mode tests
    // ========================================================================

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

    // Signature validation tests are generated by testgen (via graph_mock).

    // ========================================================================
    // Diff mode tests
    // ========================================================================

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
    fn test_gist_uses_cloud_credential_chain() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("snapshot should build");

        assert!(dag.get_node(&"cloud_env".into()).is_some());
        assert!(dag.get_node(&"resolve_auth".into()).is_some());
        assert!(dag.get_node(&"bind_secret".into()).is_some());
        assert!(dag.get_node(&"cloud_credential".into()).is_some());
        assert!(
            dag.get_node(&"credential_env".into()).is_none(),
            "legacy credential_env node should be removed"
        );
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

    // Signature validation tests are generated by testgen (via graph_mock).

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

        assert!(dag.get_node(&"prepare".into()).is_some());
        assert!(dag.get_node(&"execute".into()).is_some());
        assert!(dag.get_node(&"parse".into()).is_some());
    }
}
