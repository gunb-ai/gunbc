//! Graph builder for the gist tool.
//!
//! This graph is composed from primitives and library ops.
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! # Transport Pattern
//!
//! All I/O happens through explicit `TransportOps::Execute` nodes:
//! - ListFiles: PrepareListFiles -> Execute -> ParseListFiles
//! - ReadFiles: per-file loop (`PrepareReadFile -> Execute -> ParseReadFile`)
//! - Gist creation: PrepareRequest -> Execute

use gunbc_exec::{
    optional_str_list_strict, optional_str_strict, propagate_skipped, require_response,
    require_str, ExecError, Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::patterns::PatternOp;
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::transport::{FileOp, FileRequest, ShellRequest, TransportRequest, TransportResponse};
use gunbc_ir::{
    add_transport_triplet, build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value,
    WorkflowSignature,
};
use gunbc_lib_cloud_ops::graph_cloud_config;
use gunbc_lib_gist_ops::{build_gist_upload_subdag, GistOps, GistUploadOp};
use gunbc_lib_git_ops::{build_branch_resolution_subdag, BranchResolutionOp, GitOps};
use gunbc_lib_markdown::MarkdownOp;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};
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

    // ========================================================================
    // Single-file operations (used by LoopBuilder in snapshot mode)
    // ========================================================================
    /// Prepare single file read request (PURE - no I/O)
    /// Takes filename and repo_path, outputs file read request for one file
    PrepareReadFile,
    /// Parse single file read response (PURE - no I/O)
    /// Takes file response, outputs filename and content
    ParseReadFile,
    /// Collect file results into a map (PURE - no I/O)
    /// Takes list of (filename, content) pairs, outputs Map
    CollectFileContents,

    // ========================================================================
    // Library ops
    // ========================================================================
    /// Markdown operations
    Markdown(MarkdownOp),

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

    // ========================================================================
    // SubDag wrappers (shared library SubDags)
    // ========================================================================
    /// Gist upload pipeline (credential chain + request + response).
    ///
    /// Self-contained SubDag — includes its own `fs_env`, `clock_env`,
    /// cloud credential chain, and gist upload pipeline.
    GistUpload(GistUploadOp),

    /// Branch resolution (current_branch + remote_branches).
    ///
    /// Wraps the two-triplet branch resolution SubDag from git-ops.
    BranchResolution(BranchResolutionOp),
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

            // Single-file operations (pure)
            GistGraphOp::PrepareReadFile => execute_prepare_read_file(inputs),
            GistGraphOp::ParseReadFile => execute_parse_read_file(inputs),
            GistGraphOp::CollectFileContents => execute_collect_file_contents(inputs),

            // Pattern ops (loop unpack/pack, etc.)
            GistGraphOp::Pattern(op) => op.execute(inputs),

            // Library ops
            GistGraphOp::Markdown(op) => op.execute(inputs),

            // Transport boundary
            GistGraphOp::Transport(op) => op.execute(inputs),

            // SubDag wrappers (delegated to shared libraries)
            GistGraphOp::GistUpload(op) => op.execute(inputs),
            GistGraphOp::BranchResolution(op) => op.execute(inputs),
        }
    }
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
/// - request: TransportRequest (file read request for one file)
/// - filename: pass through for correlation
fn execute_prepare_read_file(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let filename = require_str(&inputs, "filename")?;
    let repo_path = optional_str_strict(&inputs, "repo_path")?.unwrap_or(".");

    let path = if repo_path == "." {
        filename.to_string()
    } else {
        format!("{repo_path}/{filename}")
    };
    let request = TransportRequest::File(FileRequest::read(path));

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
/// - response: TransportResponse from file read
/// - filename: the filename (for correlation)
///
/// Outputs:
/// - filename: the original filename
/// - result: the file content
fn execute_parse_read_file(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    if let Some(result) = propagate_skipped(&inputs, "response", &["filename", "result"]) {
        return result;
    }
    let response = require_response(&inputs, "response")?;
    let filename = require_str(&inputs, "filename")?;
    let file = response.require_file()?;

    if file.operation != FileOp::Read {
        return Err(ExecError::new(format!(
            "expected file read response for '{}', got {:?}",
            filename, file.operation
        )));
    }
    if !file.success {
        let err = file.error.as_deref().unwrap_or("unknown file read error");
        // Directories (e.g. git submodules) appear in git ls-files but can't
        // be read as files. Return empty content so the loop continues; the
        // downstream collect node filters these out.
        if err.contains("Is a directory") || err.contains("is a directory") {
            return OutputMap::new()
                .str("filename", filename)
                .str("result", "")
                .ok();
        }
        return Err(ExecError::new(format!(
            "failed to read '{}': {}",
            filename, err
        )));
    }
    let content = file.content.clone().ok_or_else(|| {
        ExecError::new(format!(
            "missing file content in read response for '{}'",
            filename
        ))
    })?;

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
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("file", "FilesystemHandle", AccessMode::Read),
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
    dag.add_edge(gunbc_ir::Edge::new("prepare", "skip", "execute", "skip"));
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
        .with_output("url", "String", Cardinality::ONE)
        // cloud_credential subdag exposes ok from IAM ensure chain (LocalDev only)
        .with_output("ok", "Bool", Cardinality::ONE);

    // base_ref is an entrypoint from gist_upload SubDag in snapshot and diff modes
    // (in recent mode it's wired from rev_list, so it's not an entrypoint)
    if !matches!(mode, GistMode::Recent) {
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
    build_gist_graph_with_config(mode, extensions, public, graph_cloud_config())
}

/// Build the gist graph with an explicit `CloudSecretConfig`.
///
/// The graph uses shared SubDags for branch resolution and gist upload:
///
/// ```text
///   fs_env ──────> content_acquisition ──> render_markdown ──┐
///                                                            │
///   branch_resolution ───────────────────────────────────┐   │
///                                                        │   │
///   gist_upload <────── markdown + branch + remote + base_ref
/// ```
///
/// The `gist_upload` SubDag is self-contained: it includes its own
/// credential chain, filesystem/clock environments, and gist request
/// pipeline. Consumers just wire `markdown` in and get `url` out.
pub fn build_gist_graph_with_config(
    mode: GistMode,
    extensions: Vec<String>,
    public: bool,
    cloud_config: CloudSecretConfig,
) -> Result<Dag<GistGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Environment: filesystem (for content acquisition transport boundaries)
    // ========================================================================

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        GistGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
    ))?;

    // ========================================================================
    // Content acquisition (mode-dependent)
    // ========================================================================
    // Both modes produce a render_markdown node handle that outputs "markdown".

    let (render_markdown, recent_parse_rev_list) = match &mode {
        GistMode::Snapshot => (
            build_snapshot_acquire(&mut builder, &fs_env, extensions)?,
            None,
        ),
        GistMode::Diff { base_ref } => (
            build_diff_acquire(&mut builder, &fs_env, base_ref, extensions)?,
            None,
        ),
        GistMode::Recent => {
            let (md, rev) = build_recent_acquire(&mut builder, &fs_env, extensions)?;
            (md, Some(rev))
        }
    };

    // ========================================================================
    // Branch resolution SubDag (parallel to content acquisition)
    // ========================================================================

    let branch_dag = lift_branch_dag(build_branch_resolution_subdag());
    let branch_resolution =
        builder.add_node_after(Node::subdag("branch_resolution", branch_dag), &fs_env)?;

    // Wire filesystem handle to branch resolution
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        branch_resolution.in_port("res:file"),
    )?;

    // ========================================================================
    // Gist upload SubDag (self-contained credential chain + upload)
    // ========================================================================

    let gist_dag = lift_gist_upload_dag(build_gist_upload_subdag(cloud_config, public)?);
    let gist_upload =
        builder.add_node_after(Node::subdag("gist_upload", gist_dag), &render_markdown)?;

    // Wire: content → gist_upload.markdown
    builder.add_edge(
        render_markdown.out("markdown"),
        gist_upload.in_port("markdown"),
    )?;

    // Wire: branch_resolution → gist_upload
    builder.add_edge(
        branch_resolution.out("branch"),
        gist_upload.in_port("branch"),
    )?;
    builder.add_edge(
        branch_resolution.out("remote_branch"),
        gist_upload.in_port("remote_branch"),
    )?;

    // Wire commit range (recent mode only) so filename reflects the diff range
    if let Some(ref parse_rev_list) = recent_parse_rev_list {
        builder.add_edge(
            parse_rev_list.out("base_ref"),
            gist_upload.in_port("base_ref"),
        )?;
    }

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
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![list("files", "String")],
        GistGraphOp::Git(GitOps::PrepareLsFiles { extensions }),
        GistGraphOp::Git(GitOps::ParseLsFiles),
        GistGraphOp::Transport(TransportOps::Execute),
        Some(fs_env),
    )?;

    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        list_files.in_port("res:file"),
    )?;

    // Node: LoopBuilder for per-file reading
    use gunbc_ir::patterns::{LoopBuilder, ResourceInput};

    let body = build_read_file_body_dag();
    let loop_node: Node<GistGraphOp> = LoopBuilder::new("read_files_loop")
        .with_input("files", "String", Cardinality::ZERO_OR_MORE)
        .with_element("filename", "String")
        .with_resource_input(ResourceInput::new("res:file", "FilesystemHandle"))
        .with_body(body)
        .with_output("contents", "String")
        .build();

    let read_files_loop = builder.add_node_after(loop_node, &list_files)?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        read_files_loop.in_port("res:file"),
    )?;

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
    builder.add_edge(list_files.out("files"), read_files_loop.in_port("files"))?;
    builder.add_edge(
        list_files.out("files"),
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
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![port("diff_files", "Map"), scalar("stats", "String")],
        GistGraphOp::Git(GitOps::PrepareDiff {
            base_ref: base_ref.to_string(),
            extensions,
        }),
        GistGraphOp::Git(GitOps::ParseDiff),
        GistGraphOp::Transport(TransportOps::Execute),
        Some(fs_env),
    )?;

    builder.add_edge(fs_env.out(FsEnv::WRITE_PORT), diff.in_port("res:file"))?;

    // Node: RenderDiffSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![
                port("diff_files", "Map"),
                optional("stats", "OptionalString"),
            ],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderDiffSnapshot),
        ),
        &diff,
    )?;

    // Wire diff → render (internal triplet edges handled by helper)
    builder.add_edge(
        diff.out("diff_files"),
        render_markdown.in_port("diff_files"),
    )?;
    builder.add_edge(diff.out("stats"), render_markdown.in_port("stats"))?;

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
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![optional("base_ref", "OptionalString")],
        GistGraphOp::Git(GitOps::PrepareRevListBefore {
            before: "3 days ago".to_string(),
        }),
        GistGraphOp::Git(GitOps::ParseRevListBefore),
        GistGraphOp::Transport(TransportOps::Execute),
        Some(fs_env),
    )?;

    builder.add_edge(fs_env.out(FsEnv::WRITE_PORT), rev_list.in_port("res:file"))?;

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
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![port("diff_files", "Map"), scalar("stats", "String")],
        GistGraphOp::Git(GitOps::PrepareDiff {
            base_ref: "HEAD".to_string(),
            extensions,
        }),
        GistGraphOp::Git(GitOps::ParseDiff),
        GistGraphOp::Transport(TransportOps::Execute),
        Some(&rev_list),
    )?;

    builder.add_edge(fs_env.out(FsEnv::WRITE_PORT), diff.in_port("res:file"))?;

    // Node: RenderDiffSnapshot (PURE)
    let render_markdown = builder.add_node_after(
        Node::opaque(
            "render_markdown",
            vec![
                port("diff_files", "Map"),
                optional("stats", "OptionalString"),
            ],
            vec![scalar("markdown", "String")],
            GistGraphOp::Markdown(MarkdownOp::RenderDiffSnapshot),
        ),
        &diff,
    )?;

    // Wire cross-triplet edges (internal triplet edges handled by helpers)
    builder.add_edge(rev_list.out("base_ref"), diff.in_port("base_ref"))?;
    builder.add_edge(
        diff.out("diff_files"),
        render_markdown.in_port("diff_files"),
    )?;
    builder.add_edge(diff.out("stats"), render_markdown.in_port("stats"))?;

    Ok((render_markdown, rev_list))
}

fn lift_gist_upload_dag(dag: Dag<GistUploadOp>) -> Dag<GistGraphOp> {
    dag.map_ops(&mut GistGraphOp::GistUpload)
}

fn lift_branch_dag(dag: Dag<BranchResolutionOp>) -> Dag<GistGraphOp> {
    dag.map_ops(&mut GistGraphOp::BranchResolution)
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
                    | GitOps::PrepareRevListBefore { .. }
                    | GitOps::PrepareGitShow { .. } => OutputMap::new()
                        .request(
                            "request",
                            ShellRequest::new("git")
                                .arg("mock")
                                .into_transport_request(),
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
                    GitOps::ParseGitShow => OutputMap::new().str("content", "{}").build(),
                }
            }

            // Single-file operations
            GistGraphOp::PrepareReadFile => OutputMap::new()
                .request(
                    "request",
                    TransportRequest::File(FileRequest::read("src/main.rs")),
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
                PatternOp::BranchMerge { output_port } => {
                    OutputMap::new().str(output_port, "mock_merge").build()
                }
                _ => HashMap::new(),
            },

            // Environment ops
            GistGraphOp::FsEnv(op) => op.mock_outputs(),

            // Pure ops
            GistGraphOp::Markdown(_) => OutputMap::new()
                .str("markdown", "# Code Snapshot\n```rust\nfn main() {}\n```")
                .build(),

            // Transport boundary
            GistGraphOp::Transport(_) => OutputMap::new()
                .response(
                    "response",
                    TransportResponse::Shell(gunbc_ir::transport::ShellResponse::ok(
                        "src/main.rs\nREADME.md\n",
                    )),
                )
                .build(),

            // SubDag wrappers — delegate to inner op's mock outputs
            GistGraphOp::GistUpload(op) => mock_gist_upload_op(op),
            GistGraphOp::BranchResolution(op) => mock_branch_resolution_op(op),
        }
    }
}

/// Mock outputs for gist upload SubDag operations.
fn mock_gist_upload_op(op: &GistUploadOp) -> HashMap<String, Value> {
    match op {
        GistUploadOp::Gist(gist_op) => match gist_op {
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
        GistUploadOp::Cloud(_) => OutputMap::new()
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
        GistUploadOp::Transport(_) => OutputMap::new()
            .response(
                "response",
                TransportResponse::Shell(gunbc_ir::transport::ShellResponse::ok(
                    "https://gist.github.com/mock/123\n",
                )),
            )
            .build(),
        GistUploadOp::FsEnv(op) => op.mock_outputs(),
        GistUploadOp::ClockEnv(op) => op.mock_outputs(),
        GistUploadOp::ResolveAuth => OutputMap::new()
            .str("service", "github")
            .str("scheme", "bearer")
            .str("header_name", "")
            .str_list("required_scopes", vec!["gist:write".to_string()])
            .bool("interactive_allowed", true)
            .int("lifetime_seconds", 3600)
            .build(),
    }
}

/// Mock outputs for branch resolution SubDag operations.
fn mock_branch_resolution_op(op: &BranchResolutionOp) -> HashMap<String, Value> {
    match op {
        BranchResolutionOp::Git(git_op) => match git_op {
            GitOps::PrepareCurrentBranch | GitOps::PrepareRemoteBranchesAtHead => OutputMap::new()
                .request(
                    "request",
                    ShellRequest::new("git")
                        .arg("mock")
                        .into_transport_request(),
                )
                .bool("skip", false)
                .build(),
            GitOps::ParseCurrentBranch => OutputMap::new().str("branch", "main").build(),
            GitOps::ParseRemoteBranchesAtHead => {
                OutputMap::new().str("remote_branch", "main").build()
            }
            _ => HashMap::new(),
        },
        BranchResolutionOp::Transport(_) => OutputMap::new()
            .response(
                "response",
                TransportResponse::Shell(gunbc_ir::transport::ShellResponse::ok("main\n")),
            )
            .build(),
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

        // Content acquisition SubDag wrappers
        assert!(dag.get_node(&"list_files".into()).is_some());
        assert!(dag.get_node(&"read_files_loop".into()).is_some());
        // Branch resolution and gist upload are now SubDags
        assert!(dag.get_node(&"branch_resolution".into()).is_some());
        assert!(dag.get_node(&"gist_upload".into()).is_some());
    }

    #[test]
    fn test_snapshot_graph_has_entrypoints() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");
        let entrypoints = detect_entrypoints(&dag);

        // Content acquisition entrypoints
        assert!(entrypoints.is_entrypoint_port(&"list_files".into(), &"repo_path".into()));
        // Branch resolution exposes repo_path
        assert!(entrypoints.is_entrypoint_port(&"branch_resolution".into(), &"repo_path".into()));
    }

    #[test]
    fn test_snapshot_pure_nodes_not_boundaries() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");
        let boundaries = detect_boundaries(&dag);

        // Pure/intermediate nodes should not be boundaries
        assert!(!boundaries.is_boundary_node(&"collect_file_contents".into()));
        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        // gist_upload is terminal → boundary (outputs url, ok)
        assert!(boundaries.is_boundary_node(&"gist_upload".into()));
    }

    #[test]
    fn test_snapshot_transport_nodes_have_correct_ports() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");

        // list_files is a SubDag — check its interface ports
        let list_files = dag.get_node(&"list_files".into()).unwrap();
        assert!(list_files.inputs.iter().any(|p| p.name.0 == "repo_path"));
        assert!(list_files.outputs.iter().any(|p| p.name.0 == "files"));

        // branch_resolution is a SubDag — check its interface ports
        let branch = dag.get_node(&"branch_resolution".into()).unwrap();
        assert!(branch.inputs.iter().any(|p| p.name.0 == "repo_path"));
        assert!(branch.outputs.iter().any(|p| p.name.0 == "branch"));
        assert!(branch.outputs.iter().any(|p| p.name.0 == "remote_branch"));

        // gist_upload is a SubDag — check its interface ports
        let gist = dag.get_node(&"gist_upload".into()).unwrap();
        assert!(gist.inputs.iter().any(|p| p.name.0 == "markdown"));
        assert!(gist.outputs.iter().any(|p| p.name.0 == "url"));
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

        assert!(dag.get_node(&"diff".into()).is_some());
        assert!(dag.get_node(&"branch_resolution".into()).is_some());
        assert!(dag.get_node(&"gist_upload".into()).is_some());
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

        assert!(entrypoints.is_entrypoint_port(&"diff".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"diff".into(), &"base_ref".into()));
        assert!(entrypoints.is_entrypoint_port(&"branch_resolution".into(), &"repo_path".into()));
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

        // Top-level nodes: content acquisition + branch_resolution + gist_upload
        let expected_nodes = vec![
            "diff",
            "render_markdown",
            "branch_resolution",
            "gist_upload",
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

        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        // gist_upload is terminal → boundary
        assert!(boundaries.is_boundary_node(&"gist_upload".into()));
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

        // All must have the shared SubDags
        for node_id in &["branch_resolution", "gist_upload"] {
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
        let dag =
            build_gist_graph(GistMode::Snapshot, vec![], false).expect("snapshot should build");

        // Credential chain is inside the gist_upload SubDag
        assert!(dag.get_node(&"gist_upload".into()).is_some());
        // These are now inside gist_upload, not top-level
        assert!(dag.get_node(&"cloud_env".into()).is_none());
        assert!(dag.get_node(&"resolve_auth".into()).is_none());
    }

    #[test]
    fn test_snapshot_has_no_diff_nodes() {
        let dag = build_gist_graph(GistMode::Snapshot, vec![], false).expect("graph should build");

        assert!(dag.get_node(&"diff".into()).is_none());
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

        assert!(dag.get_node(&"list_files".into()).is_none());
        assert!(dag.get_node(&"read_files_loop".into()).is_none());
    }

    // ========================================================================
    // Recent mode tests
    // ========================================================================

    #[test]
    fn test_recent_graph_has_transport_boundaries() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        assert!(dag.get_node(&"rev_list".into()).is_some());
        assert!(dag.get_node(&"diff".into()).is_some());
        assert!(dag.get_node(&"branch_resolution".into()).is_some());
        assert!(dag.get_node(&"gist_upload".into()).is_some());
    }

    #[test]
    fn test_recent_graph_has_entrypoints() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");
        let entrypoints = detect_entrypoints(&dag);

        assert!(entrypoints.is_entrypoint_port(&"rev_list".into(), &"repo_path".into()));
        assert!(entrypoints.is_entrypoint_port(&"branch_resolution".into(), &"repo_path".into()));
    }

    #[test]
    fn test_recent_graph_node_ids() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        let expected_nodes = vec![
            "rev_list",
            "diff",
            "render_markdown",
            "branch_resolution",
            "gist_upload",
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

        assert!(!boundaries.is_boundary_node(&"render_markdown".into()));
        // gist_upload is terminal → boundary
        assert!(boundaries.is_boundary_node(&"gist_upload".into()));
    }

    // Signature validation tests are generated by testgen (via graph_mock).

    #[test]
    fn test_recent_has_no_snapshot_nodes() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        assert!(dag.get_node(&"list_files".into()).is_none());
        assert!(dag.get_node(&"read_files_loop".into()).is_none());
    }

    #[test]
    fn test_recent_has_rev_list_nodes() {
        let dag =
            build_gist_graph(GistMode::Recent, vec![], false).expect("recent graph should build");

        // rev_list is a SubDag wrapper
        assert!(dag.get_node(&"rev_list".into()).is_some());
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
