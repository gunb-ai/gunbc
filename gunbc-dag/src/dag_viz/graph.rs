//! Graph builder for the dag-viz tool.
//!
//! Uses DagBuilder for compile-time cycle prevention and edge validation.
//!
//! This tool visualizes DAG topology as interactive HTML or static markdown.
//! The mode determines the graph structure:
//!
//! **Snapshot** (`make dag-viz`):
//! ```text
//! BuildTopology ─┬─→ RenderSnapshot → PrepareGist → Execute → ParseGist
//!                │
//! CurrentBranch ─┘
//! ```
//!
//! **Diff** (`make dag-viz-diff`):
//! ```text
//! BuildTopology ──┬──────────────────→ DiffTopologies → RenderDiff → PrepareGist → Execute → ParseGist
//!                 │                          ↑
//! GitShow(base) → ParseBase ────────────────┘
//!                 │
//! CurrentBranch ──┘
//! ```
//!
//! **Recent** (`make dag-viz-recent`):
//! ```text
//! BuildTopology ──┬──────────────────→ DiffTopologies → RenderDiff → PrepareGist → Execute → ParseGist
//!                 │                          ↑
//! RevList → GitShow → ParseBase ────────────┘
//!                 │
//! CurrentBranch ──┘
//! ```
//!
//! **SaveSnapshot** (`make dag-snapshot`):
//! ```text
//! BuildTopology → PrepareWrite → Execute → ParseWrite
//! ```
//!
//! All I/O happens through `TransportOps::Execute` boundary nodes.

use crate::workspace::build_workspace_dag;
use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::dag_topology::DagTopology;
use gunbc_ir::transport::{
    FileOp, FileRequest, FileResponse, ShellRequest, ShellResponse, TransportRequest,
    TransportResponse,
};
use gunbc_ir::{
    add_transport_triplet, build::*, BuilderError, Cardinality, Dag, DagBuilder, Node, Value,
    WorkflowSignature,
};
use gunbc_lib_cloud_ops::graph_cloud_config;
use gunbc_lib_gist_ops::{build_gist_upload_subdag, GistUploadOp};
use gunbc_lib_git_ops::{build_branch_resolution_subdag, BranchResolutionOp};
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};
use gunbc_test::Mockable;
use std::collections::HashMap;

// ============================================================================
// Mode Selection
// ============================================================================

/// Mode for the dag-viz tool.
///
/// Each variant produces a different graph structure, mirroring `GistMode`.
#[derive(Debug, Clone)]
pub enum DagVizMode {
    /// Render current DAG topology, upload as gist.
    Snapshot,

    /// Diff current DAG vs a base branch, render annotated diagram.
    Diff {
        /// The branch to diff against (e.g., "main").
        base_ref: String,
    },

    /// Diff current DAG vs 3 days ago.
    Recent,

    /// Write topology JSON to `.dag-snapshots/workspace.json`.
    SaveSnapshot,
}

// ============================================================================
// Graph Operation Enum
// ============================================================================

/// The operation type for dag-viz graphs — a union of reusable ops and tool-specific ops.
///
/// Following the CI/gist pattern: all I/O happens through `Transport(TransportOps::Execute)`.
#[derive(Debug, Clone)]
pub enum DagVizGraphOp {
    // ========================================================================
    // Reusable library ops
    // ========================================================================
    /// Git operations (PURE - builds requests, parses responses)
    Git(gunbc_lib_git_ops::GitOps),

    /// Filesystem environment (resource acquisition)
    FsEnv(FsEnv),

    /// Transport operations (boundary - actual I/O)
    Transport(TransportOps),

    // ========================================================================
    // Tool-specific pure ops
    // ========================================================================
    /// Build workspace DAG and extract topology (PURE - introspects compiled DAGs).
    ///
    /// Outputs:
    /// - `topology_json`: JSON string of the current `DagTopology`
    /// - `node_count`: Number of top-level nodes
    /// - `total_node_count`: Total nodes including SubDag internals
    BuildTopology,

    /// Parse a base topology from a JSON string (PURE).
    ///
    /// Inputs: `topology_json` (String)
    /// Outputs: `topology_json` (String, validated)
    ParseBaseTopology,

    /// Diff two topologies and render the result (PURE).
    ///
    /// Combines diffing and rendering to avoid serializing `DagDiffResult`
    /// (which does not implement Serialize).
    ///
    /// Inputs: `current_json` (String), `base_json` (String),
    ///         `branch` (String), `base_ref` (String)
    /// Outputs: `content` (String), `is_empty` (Bool)
    DiffAndRender,

    /// Render topology as HTML or markdown (PURE).
    ///
    /// Inputs: `topology_json` (String), `branch` (String), `format` (String)
    /// Outputs: `content` (String), `ext` (String)
    RenderSnapshot,

    /// Parse git show response for DagTopology (PURE).
    ///
    /// Validates the response as DagTopology JSON. On failure, returns
    /// an empty topology. This is dag-viz-specific because it validates
    /// and defaults to empty DagTopology.
    ///
    /// Inputs: `response` (TransportResponse)
    /// Outputs: `topology_json` (String)
    ParseGitShowTopology,

    /// Prepare write-snapshot file request (PURE).
    ///
    /// Inputs: `topology_json` (String)
    /// Outputs: `request` (TransportRequest), `skip` (Bool)
    PrepareWriteSnapshot,

    /// Parse write-snapshot response (PURE).
    ///
    /// Inputs: `response` (TransportResponse), `node_count` (Int), `total_node_count` (Int)
    /// Outputs: `summary` (String)
    ParseWriteResult,

    /// Prepare local file save (PURE).
    ///
    /// Writes rendered content to a local file for browser viewing.
    ///
    /// Inputs: `content` (String), `ext` (String)
    /// Outputs: `request` (TransportRequest), `skip` (Bool)
    PrepareLocalSave {
        /// Output directory (e.g., "target/dag-viz").
        output_dir: String,
    },

    /// Parse local file save response (PURE).
    ///
    /// Inputs: `response` (TransportResponse)
    /// Outputs: `file_path` (String)
    ParseLocalSave,

    /// Prepare browser open command (PURE).
    ///
    /// Inputs: `file_path` (String)
    /// Outputs: `request` (TransportRequest), `skip` (Bool)
    OpenBrowser,

    /// Parse browser open result (PURE — no-op).
    ///
    /// Inputs: `response` (TransportResponse)
    /// Outputs: `opened` (Bool)
    ParseBrowserOpen,

    // ========================================================================
    // SubDag wrappers (shared library SubDags)
    // ========================================================================
    /// Gist upload pipeline (credential chain + request + response).
    ///
    /// Self-contained SubDag — replaces the inline `gh gist create` approach
    /// with the proper REST API + cloud credential chain.
    GistUpload(GistUploadOp),

    /// Branch resolution (current_branch + remote_branches).
    ///
    /// Wraps the two-triplet branch resolution SubDag from git-ops.
    BranchResolution(BranchResolutionOp),
}

// ============================================================================
// Executable Implementation
// ============================================================================

impl Executable for DagVizGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DagVizGraphOp::Git(op) => op.execute(inputs),
            DagVizGraphOp::FsEnv(op) => op.execute(inputs),
            DagVizGraphOp::Transport(op) => op.execute(inputs),

            DagVizGraphOp::BuildTopology => execute_build_topology(inputs),
            DagVizGraphOp::ParseBaseTopology => execute_parse_base_topology(inputs),
            DagVizGraphOp::DiffAndRender => execute_diff_and_render(inputs),
            DagVizGraphOp::RenderSnapshot => execute_render_snapshot(inputs),
            DagVizGraphOp::ParseGitShowTopology => execute_parse_git_show_topology(inputs),
            DagVizGraphOp::PrepareWriteSnapshot => execute_prepare_write_snapshot(inputs),
            DagVizGraphOp::ParseWriteResult => execute_parse_write_result(inputs),
            DagVizGraphOp::PrepareLocalSave { output_dir } => {
                execute_prepare_local_save(inputs, output_dir)
            }
            DagVizGraphOp::ParseLocalSave => execute_parse_local_save(inputs),
            DagVizGraphOp::OpenBrowser => execute_open_browser(inputs),
            DagVizGraphOp::ParseBrowserOpen => execute_parse_browser_open(inputs),

            // SubDag wrappers (delegated to shared libraries)
            DagVizGraphOp::GistUpload(op) => op.execute(inputs),
            DagVizGraphOp::BranchResolution(op) => op.execute(inputs),
        }
    }
}

// ============================================================================
// Operation Implementations
// ============================================================================

/// Build workspace DAG and extract topology.
fn execute_build_topology(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let dag = build_workspace_dag()
        .map_err(|e| ExecError::new(format!("Failed to build workspace DAG: {}", e)))?;
    let topo = dag.topology();

    let node_count = topo.node_count();
    let total_node_count = topo.total_node_count();

    let json = serde_json::to_string(&topo)
        .map_err(|e| ExecError::new(format!("Failed to serialize DagTopology: {}", e)))?;

    OutputMap::new()
        .str("topology_json", json)
        .int("node_count", node_count as i64)
        .int("total_node_count", total_node_count as i64)
        .ok()
}

/// Parse base topology from JSON string.
fn execute_parse_base_topology(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let json = gunbc_exec::require_str(&inputs, "topology_json")?;

    // Validate it parses, but pass through as-is
    let _topo: DagTopology = serde_json::from_str(json)
        .map_err(|e| ExecError::new(format!("Failed to parse base topology: {}", e)))?;

    OutputMap::new().str("topology_json", json).ok()
}

/// Diff two topologies and render the result as markdown.
///
/// Combines diffing and rendering into a single node to avoid serializing
/// `DagDiffResult` (which does not implement Serialize/Deserialize).
fn execute_diff_and_render(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let current_json = gunbc_exec::require_str(&inputs, "current_json")?;
    let base_json = gunbc_exec::require_str(&inputs, "base_json")?;
    let branch = gunbc_exec::optional_str_strict(&inputs, "branch")?.unwrap_or("unknown");
    let base_ref = gunbc_exec::optional_str_strict(&inputs, "base_ref")?.unwrap_or("HEAD~1");

    let current: DagTopology = serde_json::from_str(current_json)
        .map_err(|e| ExecError::new(format!("Failed to parse current topology: {}", e)))?;
    let base: DagTopology = serde_json::from_str(base_json)
        .map_err(|e| ExecError::new(format!("Failed to parse base topology: {}", e)))?;

    let diff = gunbc_ir::dag_diff::diff_topologies(&base, &current);
    let is_empty = diff.is_empty();

    let title = format!("DAG Diff: {}...{}", base_ref, branch);
    let content = gunbc_lib_markdown::render_dag_diff_snapshot(&title, &current, &diff, &base);

    OutputMap::new()
        .str("content", content)
        .bool("is_empty", is_empty)
        .ok()
}

/// Render snapshot as HTML or markdown.
fn execute_render_snapshot(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let topology_json = gunbc_exec::require_str(&inputs, "topology_json")?;
    let branch = gunbc_exec::optional_str_strict(&inputs, "branch")?.unwrap_or("unknown");
    let format = gunbc_exec::require_str(&inputs, "format")?;

    let topo: DagTopology = serde_json::from_str(topology_json)
        .map_err(|e| ExecError::new(format!("Failed to parse topology: {}", e)))?;

    let title = format!("DAG Snapshot ({})", branch);

    let (content, ext) = match format {
        "md" => (gunbc_lib_markdown::render_dag_snapshot(&title, &topo), "md"),
        _ => (crate::viewer::render_viewer(&title, &topo), "html"),
    };

    OutputMap::new()
        .str("content", content)
        .str("ext", ext)
        .ok()
}

/// Parse git show response into topology JSON (dag-viz specific).
///
/// Validates the response as DagTopology. On failure or empty content,
/// returns an empty DagTopology.
fn execute_parse_git_show_topology(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let content = gunbc_exec::require_str(&inputs, "content")?;

    let json = content.trim().to_string();
    if json.is_empty() {
        // No snapshot at base ref — return empty topology
        let empty = DagTopology {
            nodes: vec![],
            edges: vec![],
        };
        let empty_json = serde_json::to_string(&empty).unwrap();
        OutputMap::new().str("topology_json", empty_json).ok()
    } else {
        // Validate JSON parses as DagTopology
        let _: DagTopology = serde_json::from_str(&json)
            .map_err(|e| ExecError::new(format!("Invalid base topology JSON: {}", e)))?;
        OutputMap::new().str("topology_json", json).ok()
    }
}

/// Prepare write-snapshot file request.
fn execute_prepare_write_snapshot(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let topology_json = gunbc_exec::require_str(&inputs, "topology_json")?;

    // Pretty-print the JSON for readability
    let topo: DagTopology = serde_json::from_str(topology_json)
        .map_err(|e| ExecError::new(format!("Failed to parse topology: {}", e)))?;
    let pretty_json = serde_json::to_string_pretty(&topo).unwrap();

    let request = TransportRequest::File(FileRequest {
        path: ".dag-snapshots/workspace.json".to_string(),
        operation: FileOp::Write,
        content: Some(pretty_json),
        create_parents: true,
    });

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse write-snapshot response.
fn execute_parse_write_result(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = gunbc_exec::require_response(&inputs, "response")?;
    let node_count = gunbc_exec::require_int(&inputs, "node_count")?;
    let total_node_count = gunbc_exec::require_int(&inputs, "total_node_count")?;

    match response {
        TransportResponse::File(FileResponse { success: true, .. }) => {
            let summary = format!(
                "Saved DAG topology snapshot to .dag-snapshots/workspace.json ({} workflows, {} total nodes)",
                node_count, total_node_count
            );
            OutputMap::new().str("summary", summary).ok()
        }
        TransportResponse::File(ref f) => Err(ExecError::new(format!(
            "Failed to write snapshot: {}",
            f.error.as_deref().unwrap_or("unknown error")
        ))),
        _ => Err(ExecError::new("Unexpected response type for file write")),
    }
}

/// Prepare local file save request.
fn execute_prepare_local_save(
    inputs: HashMap<String, Value>,
    output_dir: &str,
) -> Result<HashMap<String, Value>, ExecError> {
    let content = gunbc_exec::require_str(&inputs, "content")?;
    let ext = gunbc_exec::require_str(&inputs, "ext")?;

    let file_path = format!("{}/dag-visualization.{}", output_dir, ext);
    let request = TransportRequest::File(FileRequest {
        path: file_path,
        operation: FileOp::Write,
        content: Some(content.to_string()),
        create_parents: true,
    });

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse local file save response.
fn execute_parse_local_save(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = gunbc_exec::require_response(&inputs, "response")?;

    match response {
        TransportResponse::File(FileResponse {
            success: true,
            ref path,
            ..
        }) => OutputMap::new().str("file_path", path).ok(),
        TransportResponse::File(ref f) => Err(ExecError::new(format!(
            "Failed to save local file: {}",
            f.error.as_deref().unwrap_or("unknown error")
        ))),
        _ => Err(ExecError::new(
            "Unexpected response type for local file save",
        )),
    }
}

/// Prepare browser open command.
///
/// Uses platform-appropriate open command:
/// - WSL: `wslview` (opens in Windows default browser)
/// - macOS: `open`
/// - Linux: `xdg-open`
fn execute_open_browser(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let file_path = gunbc_exec::require_str(&inputs, "file_path")?;

    // Detect WSL via the WSL_DISTRO_NAME env var (always set on WSL2).
    let is_wsl = std::env::var("WSL_DISTRO_NAME").is_ok();

    let request = if is_wsl {
        // On WSL, convert to absolute path and use wslview to open in Windows browser
        let abs_path = std::path::Path::new(file_path);
        let abs_path = if abs_path.is_relative() {
            std::env::current_dir()
                .map(|cwd| cwd.join(abs_path))
                .unwrap_or_else(|_| abs_path.to_path_buf())
        } else {
            abs_path.to_path_buf()
        };
        ShellRequest::new("wslview")
            .arg(abs_path.to_string_lossy().into_owned())
            .into_transport_request()
    } else if cfg!(target_os = "macos") {
        ShellRequest::new("open")
            .arg(file_path)
            .into_transport_request()
    } else {
        ShellRequest::new("xdg-open")
            .arg(file_path)
            .into_transport_request()
    };

    OutputMap::new()
        .request("request", request)
        .bool("skip", false)
        .ok()
}

/// Parse browser open result (best-effort — browser open may fail silently).
fn execute_parse_browser_open(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let response = gunbc_exec::require_response(&inputs, "response")?;

    let opened = matches!(
        response,
        TransportResponse::Shell(ref s) if s.success()
    );

    OutputMap::new().bool("opened", opened).ok()
}

// ============================================================================
// SubDag lifters
// ============================================================================

fn lift_gist_upload_dag(dag: Dag<GistUploadOp>) -> Dag<DagVizGraphOp> {
    dag.map_ops(&mut DagVizGraphOp::GistUpload)
}

fn lift_branch_dag(dag: Dag<BranchResolutionOp>) -> Dag<DagVizGraphOp> {
    dag.map_ops(&mut DagVizGraphOp::BranchResolution)
}

// ============================================================================
// Workflow Signature
// ============================================================================

/// Declared workflow signature for dag-viz.
pub fn dag_viz_signature(mode: &DagVizMode) -> WorkflowSignature {
    let mut sig = WorkflowSignature::new();

    match mode {
        DagVizMode::Snapshot => {
            sig = sig
                .with_input("repo_path", "String", Cardinality::ONE)
                .with_input("format", "String", Cardinality::ONE)
                .with_input("base_ref", "OptionalString", Cardinality::ZERO_OR_ONE)
                .with_output("url", "String", Cardinality::ONE)
                .with_output("node_count", "Int", Cardinality::ONE)
                .with_output("total_node_count", "Int", Cardinality::ONE)
                .with_output("opened", "Bool", Cardinality::ONE)
                .with_output("ok", "Bool", Cardinality::ONE);
        }
        DagVizMode::Diff { .. } => {
            sig = sig
                .with_input("repo_path", "String", Cardinality::ONE)
                .with_input("base_ref", "OptionalString", Cardinality::ZERO_OR_ONE)
                .with_output("url", "String", Cardinality::ONE)
                .with_output("node_count", "Int", Cardinality::ONE)
                .with_output("total_node_count", "Int", Cardinality::ONE)
                .with_output("is_empty", "Bool", Cardinality::ONE)
                .with_output("ok", "Bool", Cardinality::ONE);
        }
        DagVizMode::Recent => {
            sig = sig
                .with_input("repo_path", "String", Cardinality::ONE)
                .with_output("url", "String", Cardinality::ONE)
                .with_output("node_count", "Int", Cardinality::ONE)
                .with_output("total_node_count", "Int", Cardinality::ONE)
                .with_output("is_empty", "Bool", Cardinality::ONE)
                .with_output("ok", "Bool", Cardinality::ONE);
        }
        DagVizMode::SaveSnapshot => {
            sig = sig.with_output("summary", "String", Cardinality::ONE);
        }
    }

    sig
}

// ============================================================================
// Graph Builder
// ============================================================================

/// Build the dag-viz graph using DagBuilder.
///
/// The mode determines which subgraph is constructed. All modes share a
/// `BuildTopology` root node; snapshot/diff/recent add git + gist transport,
/// save-snapshot adds a file write transport.
pub fn build_dag_viz_graph(mode: DagVizMode) -> Result<Dag<DagVizGraphOp>, BuilderError> {
    let mut builder = DagBuilder::new();

    // ========================================================================
    // Environment: filesystem (for transport boundaries)
    // ========================================================================

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        DagVizGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
    ))?;

    // ========================================================================
    // Shared root: BuildTopology
    // ========================================================================

    let build_topology = builder.add_root_node(Node::opaque(
        "build_topology",
        vec![],
        vec![
            scalar("topology_json", "String"),
            scalar("node_count", "Int"),
            scalar("total_node_count", "Int"),
        ],
        DagVizGraphOp::BuildTopology,
    ))?;

    match mode {
        DagVizMode::Snapshot => {
            build_snapshot_graph(&mut builder, &fs_env, &build_topology)?;
        }
        DagVizMode::Diff { ref base_ref } => {
            build_diff_graph(&mut builder, &fs_env, &build_topology, base_ref)?;
        }
        DagVizMode::Recent => {
            build_recent_graph(&mut builder, &fs_env, &build_topology)?;
        }
        DagVizMode::SaveSnapshot => {
            build_save_snapshot_graph(&mut builder, &fs_env, &build_topology)?;
        }
    }

    Ok(builder.build())
}

// ============================================================================
// Mode-Specific Graph Builders
// ============================================================================

/// Snapshot mode: BuildTopology + BranchResolution → RenderSnapshot → GistUpload + LocalSave + BrowserOpen.
fn build_snapshot_graph(
    builder: &mut DagBuilder<DagVizGraphOp>,
    fs_env: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
    build_topology: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
) -> Result<(), BuilderError> {
    // Branch resolution SubDag
    let branch_dag = lift_branch_dag(build_branch_resolution_subdag());
    let branch_resolution =
        builder.add_node_after(Node::subdag("branch_resolution", branch_dag), fs_env)?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        branch_resolution.in_port("res:file"),
    )?;

    // Render snapshot
    let render_snapshot = builder.add_node_after_all(
        Node::opaque(
            "render_snapshot",
            vec![
                scalar("topology_json", "String"),
                optional("branch", "OptionalString"),
                scalar("format", "String"),
            ],
            vec![scalar("content", "String"), scalar("ext", "String")],
            DagVizGraphOp::RenderSnapshot,
        ),
        &[build_topology, &branch_resolution],
    )?;

    builder.add_edge(
        build_topology.out("topology_json"),
        render_snapshot.in_port("topology_json"),
    )?;
    builder.add_edge(
        branch_resolution.out("branch"),
        render_snapshot.in_port("branch"),
    )?;

    // Gist upload SubDag (replaces gh gist create shell approach)
    let gist_dag = lift_gist_upload_dag(build_gist_upload_subdag(graph_cloud_config(), false));
    let gist_upload =
        builder.add_node_after(Node::subdag("gist_upload", gist_dag), &render_snapshot)?;

    builder.add_edge(
        render_snapshot.out("content"),
        gist_upload.in_port("markdown"),
    )?;
    builder.add_edge(
        branch_resolution.out("branch"),
        gist_upload.in_port("branch"),
    )?;
    builder.add_edge(
        branch_resolution.out("remote_branch"),
        gist_upload.in_port("remote_branch"),
    )?;

    // Local save + browser open (parallel to gist upload)
    let local_save = add_transport_triplet(
        builder,
        "local_save",
        vec![scalar("content", "String"), scalar("ext", "String")],
        vec![resource("file", "FilesystemHandle", AccessMode::Write)],
        vec![scalar("file_path", "String")],
        DagVizGraphOp::PrepareLocalSave {
            output_dir: "target/dag-viz".to_string(),
        },
        DagVizGraphOp::ParseLocalSave,
        DagVizGraphOp::Transport(TransportOps::Execute),
        Some(&render_snapshot),
    )?;

    builder.add_edge(
        render_snapshot.out("content"),
        local_save.in_port("content"),
    )?;
    builder.add_edge(render_snapshot.out("ext"), local_save.in_port("ext"))?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        local_save.in_port("res:file"),
    )?;

    // Open in browser after local save
    let browser_open = add_transport_triplet(
        builder,
        "browser_open",
        vec![scalar("file_path", "String")],
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![scalar("opened", "Bool")],
        DagVizGraphOp::OpenBrowser,
        DagVizGraphOp::ParseBrowserOpen,
        DagVizGraphOp::Transport(TransportOps::Execute),
        Some(&local_save),
    )?;

    builder.add_edge(
        local_save.out("file_path"),
        browser_open.in_port("file_path"),
    )?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        browser_open.in_port("res:file"),
    )?;

    Ok(())
}

/// Diff mode: BuildTopology + BranchResolution + GitShow(base) → DiffAndRender → GistUpload.
fn build_diff_graph(
    builder: &mut DagBuilder<DagVizGraphOp>,
    fs_env: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
    build_topology: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
    base_ref: &str,
) -> Result<(), BuilderError> {
    // Branch resolution SubDag
    let branch_dag = lift_branch_dag(build_branch_resolution_subdag());
    let branch_resolution =
        builder.add_node_after(Node::subdag("branch_resolution", branch_dag), fs_env)?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        branch_resolution.in_port("res:file"),
    )?;

    // Git show: load base topology (generic git show triplet + dag-viz-specific parsing)
    let git_show = add_transport_triplet(
        builder,
        "git_show_base",
        vec![optional("base_ref", "OptionalString")],
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![scalar("content", "String")],
        DagVizGraphOp::Git(gunbc_lib_git_ops::GitOps::PrepareGitShow {
            default_ref: base_ref.to_string(),
            path: ".dag-snapshots/workspace.json".to_string(),
        }),
        DagVizGraphOp::Git(gunbc_lib_git_ops::GitOps::ParseGitShow),
        DagVizGraphOp::Transport(TransportOps::Execute),
        Some(fs_env),
    )?;

    builder.add_edge(fs_env.out(FsEnv::WRITE_PORT), git_show.in_port("res:file"))?;

    // Parse git show content as DagTopology (dag-viz-specific validation)
    let parse_base_topology = builder.add_node_after(
        Node::opaque(
            "parse_base_topology",
            vec![scalar("content", "String")],
            vec![scalar("topology_json", "String")],
            DagVizGraphOp::ParseGitShowTopology,
        ),
        &git_show,
    )?;
    builder.add_edge(
        git_show.out("content"),
        parse_base_topology.in_port("content"),
    )?;

    // Diff + render (combined to avoid serializing DagDiffResult)
    let diff_and_render = builder.add_node_after_all(
        Node::opaque(
            "diff_and_render",
            vec![
                scalar("current_json", "String"),
                scalar("base_json", "String"),
                optional("branch", "OptionalString"),
                optional("base_ref", "OptionalString"),
            ],
            vec![scalar("content", "String"), scalar("is_empty", "Bool")],
            DagVizGraphOp::DiffAndRender,
        ),
        &[build_topology, &parse_base_topology, &branch_resolution],
    )?;

    builder.add_edge(
        build_topology.out("topology_json"),
        diff_and_render.in_port("current_json"),
    )?;
    builder.add_edge(
        parse_base_topology.out("topology_json"),
        diff_and_render.in_port("base_json"),
    )?;
    builder.add_edge(
        branch_resolution.out("branch"),
        diff_and_render.in_port("branch"),
    )?;

    // Gist upload SubDag
    let gist_dag = lift_gist_upload_dag(build_gist_upload_subdag(graph_cloud_config(), false));
    let gist_upload =
        builder.add_node_after(Node::subdag("gist_upload", gist_dag), &diff_and_render)?;

    builder.add_edge(
        diff_and_render.out("content"),
        gist_upload.in_port("markdown"),
    )?;
    builder.add_edge(
        branch_resolution.out("branch"),
        gist_upload.in_port("branch"),
    )?;
    builder.add_edge(
        branch_resolution.out("remote_branch"),
        gist_upload.in_port("remote_branch"),
    )?;

    Ok(())
}

/// Recent mode: BranchResolution + RevList → GitShow → ParseBaseTopology → DiffAndRender → GistUpload.
fn build_recent_graph(
    builder: &mut DagBuilder<DagVizGraphOp>,
    fs_env: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
    build_topology: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
) -> Result<(), BuilderError> {
    // Branch resolution SubDag
    let branch_dag = lift_branch_dag(build_branch_resolution_subdag());
    let branch_resolution =
        builder.add_node_after(Node::subdag("branch_resolution", branch_dag), fs_env)?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        branch_resolution.in_port("res:file"),
    )?;

    // Rev-list: find commit from 3 days ago
    let rev_list = add_transport_triplet(
        builder,
        "rev_list",
        vec![port("repo_path", "String")],
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![optional("base_ref", "OptionalString")],
        DagVizGraphOp::Git(gunbc_lib_git_ops::GitOps::PrepareRevListBefore {
            before: "3 days ago".to_string(),
        }),
        DagVizGraphOp::Git(gunbc_lib_git_ops::GitOps::ParseRevListBefore),
        DagVizGraphOp::Transport(TransportOps::Execute),
        Some(fs_env),
    )?;

    builder.add_edge(fs_env.out(FsEnv::WRITE_PORT), rev_list.in_port("res:file"))?;

    // Git show: load base topology (generic git show triplet + dag-viz-specific parsing)
    let git_show = add_transport_triplet(
        builder,
        "git_show_base",
        vec![optional("base_ref", "OptionalString")],
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![scalar("content", "String")],
        DagVizGraphOp::Git(gunbc_lib_git_ops::GitOps::PrepareGitShow {
            default_ref: "HEAD".to_string(),
            path: ".dag-snapshots/workspace.json".to_string(),
        }),
        DagVizGraphOp::Git(gunbc_lib_git_ops::GitOps::ParseGitShow),
        DagVizGraphOp::Transport(TransportOps::Execute),
        Some(&rev_list),
    )?;

    builder.add_edge(fs_env.out(FsEnv::WRITE_PORT), git_show.in_port("res:file"))?;

    // Wire rev-list base_ref → git_show
    builder.add_edge(rev_list.out("base_ref"), git_show.in_port("base_ref"))?;

    // Parse git show content as DagTopology (dag-viz-specific validation)
    let parse_base_topology = builder.add_node_after(
        Node::opaque(
            "parse_base_topology",
            vec![scalar("content", "String")],
            vec![scalar("topology_json", "String")],
            DagVizGraphOp::ParseGitShowTopology,
        ),
        &git_show,
    )?;
    builder.add_edge(
        git_show.out("content"),
        parse_base_topology.in_port("content"),
    )?;

    // Diff + render (combined)
    let diff_and_render = builder.add_node_after_all(
        Node::opaque(
            "diff_and_render",
            vec![
                scalar("current_json", "String"),
                scalar("base_json", "String"),
                optional("branch", "OptionalString"),
                optional("base_ref", "OptionalString"),
            ],
            vec![scalar("content", "String"), scalar("is_empty", "Bool")],
            DagVizGraphOp::DiffAndRender,
        ),
        &[
            build_topology,
            &parse_base_topology,
            &branch_resolution,
            &rev_list,
        ],
    )?;

    builder.add_edge(
        build_topology.out("topology_json"),
        diff_and_render.in_port("current_json"),
    )?;
    builder.add_edge(
        parse_base_topology.out("topology_json"),
        diff_and_render.in_port("base_json"),
    )?;
    builder.add_edge(
        branch_resolution.out("branch"),
        diff_and_render.in_port("branch"),
    )?;
    builder.add_edge(
        rev_list.out("base_ref"),
        diff_and_render.in_port("base_ref"),
    )?;

    // Gist upload SubDag
    let gist_dag = lift_gist_upload_dag(build_gist_upload_subdag(graph_cloud_config(), false));
    let gist_upload =
        builder.add_node_after(Node::subdag("gist_upload", gist_dag), &diff_and_render)?;

    builder.add_edge(
        diff_and_render.out("content"),
        gist_upload.in_port("markdown"),
    )?;
    builder.add_edge(
        branch_resolution.out("branch"),
        gist_upload.in_port("branch"),
    )?;
    builder.add_edge(
        branch_resolution.out("remote_branch"),
        gist_upload.in_port("remote_branch"),
    )?;
    // Wire rev-list base_ref → gist_upload for Recent mode
    builder.add_edge(rev_list.out("base_ref"), gist_upload.in_port("base_ref"))?;

    Ok(())
}

/// Save-snapshot mode: BuildTopology → WriteFile.
fn build_save_snapshot_graph(
    builder: &mut DagBuilder<DagVizGraphOp>,
    fs_env: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
    build_topology: &gunbc_ir::builder::NodeRef<DagVizGraphOp>,
) -> Result<(), BuilderError> {
    // Prepare write request
    let prepare_write = builder.add_node_after(
        Node::opaque(
            "prepare_write_snapshot",
            vec![scalar("topology_json", "String")],
            vec![
                scalar("request", "TransportRequest"),
                scalar("skip", "Bool"),
            ],
            DagVizGraphOp::PrepareWriteSnapshot,
        ),
        build_topology,
    )?;

    builder.add_edge(
        build_topology.out("topology_json"),
        prepare_write.in_port("topology_json"),
    )?;

    // Execute write (transport boundary)
    let execute_write = builder.add_node_after(
        Node::opaque(
            "execute_write_snapshot",
            vec![
                scalar("request", "TransportRequest"),
                scalar("skip", "Bool"),
                resource("file", "FilesystemHandle", AccessMode::Write),
            ],
            vec![scalar("response", "TransportResponse")],
            DagVizGraphOp::Transport(TransportOps::Execute),
        ),
        &prepare_write,
    )?;

    builder.add_edge(
        prepare_write.out("request"),
        execute_write.in_port("request"),
    )?;
    builder.add_edge(prepare_write.out("skip"), execute_write.in_port("skip"))?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        execute_write.in_port("res:file"),
    )?;

    // Parse write result
    let parse_write = builder.add_node_after(
        Node::opaque(
            "parse_write_result",
            vec![
                scalar("response", "TransportResponse"),
                scalar("node_count", "Int"),
                scalar("total_node_count", "Int"),
            ],
            vec![scalar("summary", "String")],
            DagVizGraphOp::ParseWriteResult,
        ),
        &execute_write,
    )?;

    builder.add_edge(
        execute_write.out("response"),
        parse_write.in_port("response"),
    )?;
    builder.add_edge(
        build_topology.out("node_count"),
        parse_write.in_port("node_count"),
    )?;
    builder.add_edge(
        build_topology.out("total_node_count"),
        parse_write.in_port("total_node_count"),
    )?;

    Ok(())
}

// ============================================================================
// Mockable Implementation
// ============================================================================

impl Mockable for DagVizGraphOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            DagVizGraphOp::Git(op) => match op {
                gunbc_lib_git_ops::GitOps::PrepareCurrentBranch
                | gunbc_lib_git_ops::GitOps::PrepareRevListBefore { .. }
                | gunbc_lib_git_ops::GitOps::PrepareGitShow { .. } => OutputMap::new()
                    .request(
                        "request",
                        ShellRequest::new("git")
                            .arg("mock")
                            .into_transport_request(),
                    )
                    .bool("skip", false)
                    .build(),
                gunbc_lib_git_ops::GitOps::ParseCurrentBranch => {
                    OutputMap::new().str("branch", "main").build()
                }
                gunbc_lib_git_ops::GitOps::ParseRevListBefore => {
                    OutputMap::new().str("base_ref", "abc123").build()
                }
                gunbc_lib_git_ops::GitOps::ParseGitShow => OutputMap::new()
                    .str("content", r#"{"nodes":[],"edges":[]}"#)
                    .build(),
                _ => HashMap::new(),
            },

            DagVizGraphOp::FsEnv(op) => op.mock_outputs(),

            DagVizGraphOp::Transport(_) => OutputMap::new()
                .response(
                    "response",
                    TransportResponse::Shell(ShellResponse::ok("{}")),
                )
                .build(),

            DagVizGraphOp::BuildTopology => OutputMap::new()
                .str("topology_json", r#"{"nodes":[],"edges":[]}"#)
                .int("node_count", 0)
                .int("total_node_count", 0)
                .build(),

            DagVizGraphOp::ParseBaseTopology => OutputMap::new()
                .str("topology_json", r#"{"nodes":[],"edges":[]}"#)
                .build(),

            DagVizGraphOp::DiffAndRender => OutputMap::new()
                .str("content", "# Mock Diff\nNo changes.")
                .bool("is_empty", true)
                .build(),

            DagVizGraphOp::RenderSnapshot => OutputMap::new()
                .str("content", "<html>mock snapshot</html>")
                .str("ext", "html")
                .build(),

            DagVizGraphOp::ParseGitShowTopology => OutputMap::new()
                .str("topology_json", r#"{"nodes":[],"edges":[]}"#)
                .build(),

            DagVizGraphOp::GistUpload(op) => mock_gist_upload_op(op),
            DagVizGraphOp::BranchResolution(op) => mock_branch_resolution_op(op),

            DagVizGraphOp::PrepareWriteSnapshot => OutputMap::new()
                .request(
                    "request",
                    TransportRequest::File(FileRequest {
                        path: ".dag-snapshots/workspace.json".to_string(),
                        operation: FileOp::Write,
                        content: Some("{}".to_string()),
                        create_parents: true,
                    }),
                )
                .bool("skip", false)
                .build(),

            DagVizGraphOp::ParseWriteResult => OutputMap::new()
                .str("summary", "Saved DAG topology snapshot")
                .build(),

            DagVizGraphOp::PrepareLocalSave { .. } => OutputMap::new()
                .request(
                    "request",
                    TransportRequest::File(FileRequest {
                        path: "target/dag-viz/dag-visualization.html".to_string(),
                        operation: FileOp::Write,
                        content: Some("<html>mock</html>".to_string()),
                        create_parents: true,
                    }),
                )
                .bool("skip", false)
                .build(),

            DagVizGraphOp::ParseLocalSave => OutputMap::new()
                .str("file_path", "target/dag-viz/dag-visualization.html")
                .build(),

            DagVizGraphOp::OpenBrowser => OutputMap::new()
                .request(
                    "request",
                    ShellRequest::new("xdg-open")
                        .arg("target/dag-viz/dag-visualization.html")
                        .into_transport_request(),
                )
                .bool("skip", false)
                .build(),

            DagVizGraphOp::ParseBrowserOpen => OutputMap::new().bool("opened", true).build(),
        }
    }
}

/// Mock outputs for gist upload SubDag operations.
fn mock_gist_upload_op(op: &GistUploadOp) -> HashMap<String, Value> {
    use gunbc_lib_gist_ops::GistOps;
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
                TransportResponse::Shell(ShellResponse::ok("https://gist.github.com/mock/123\n")),
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
    use gunbc_lib_git_ops::GitOps;
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
                TransportResponse::Shell(ShellResponse::ok("main\n")),
            )
            .build(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::detect_boundaries;

    #[test]
    fn test_build_snapshot_graph() {
        let dag = build_dag_viz_graph(DagVizMode::Snapshot).expect("should build");
        let boundaries = detect_boundaries(&dag);
        assert!(
            !boundaries.boundary_nodes.is_empty(),
            "snapshot graph should have transport boundaries"
        );
    }

    #[test]
    fn test_build_diff_graph() {
        let dag = build_dag_viz_graph(DagVizMode::Diff {
            base_ref: "main".to_string(),
        })
        .expect("should build");
        let boundaries = detect_boundaries(&dag);
        assert!(
            !boundaries.boundary_nodes.is_empty(),
            "diff graph should have transport boundaries"
        );
    }

    #[test]
    fn test_build_recent_graph() {
        let dag = build_dag_viz_graph(DagVizMode::Recent).expect("should build");
        let boundaries = detect_boundaries(&dag);
        assert!(
            !boundaries.boundary_nodes.is_empty(),
            "recent graph should have transport boundaries"
        );
    }

    #[test]
    fn test_build_save_snapshot_graph() {
        let dag = build_dag_viz_graph(DagVizMode::SaveSnapshot).expect("should build");
        let boundaries = detect_boundaries(&dag);
        assert!(
            !boundaries.boundary_nodes.is_empty(),
            "save-snapshot graph should have transport boundaries"
        );
    }

    #[test]
    fn test_snapshot_signature() {
        let sig = dag_viz_signature(&DagVizMode::Snapshot);
        assert!(sig.inputs.iter().any(|i| i.name == "format".into()));
        assert!(sig.outputs.iter().any(|o| o.name == "url".into()));
    }

    #[test]
    fn test_diff_signature() {
        let sig = dag_viz_signature(&DagVizMode::Diff {
            base_ref: "main".to_string(),
        });
        assert!(sig.inputs.iter().any(|i| i.name == "base_ref".into()));
    }

    #[test]
    fn test_save_snapshot_signature() {
        let sig = dag_viz_signature(&DagVizMode::SaveSnapshot);
        assert!(sig.outputs.iter().any(|o| o.name == "summary".into()));
    }
}
