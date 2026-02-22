//! DAG builders for review workflows.
//!
//! Provides composable DAGs for the review phase pattern:
//!
//! ```text
//! BlobOps → ReviewOps::PrepareReviewPrompt → LlmOps → ReviewOps::ParseReviewResponse
//! ```
//!
//! All internal operations are PURE. I/O happens at two TransportOps::Execute nodes:
//! 1. Blob fetch (for non-inline sources)
//! 2. LLM call

use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::{
    add_transport_triplet_named_with_passthrough, build::*, BuilderError, Dag, DagBuilder, Node,
    NodeRef,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_cloud_ops::{
    build_cloud_secret_manager_credential_graph_from_config, graph_cloud_config, CloudOps,
    CloudSecretManagerGraphOp,
};
use gunbc_lib_git_ops::GitOps;
use gunbc_lib_llm_ops::LlmOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};

use crate::dimension::{DimensionOps, FermiDepth, ReviewDimension};
use crate::profile::{coding_review_profile, ReviewProfile};
use crate::{Criteria, ReviewOps, ReviewPipelineConfig};
use std::collections::HashMap;

pub type ReviewGraphOp = DynOp;

#[derive(Debug, Clone)]
enum DimensionGraphConfigOps {
    LoadLlmConfig {
        provider: String,
        model: String,
    },
    LoadDimensionConfig {
        dimension: ReviewDimension,
        criteria: Criteria,
        depth: FermiDepth,
    },
}

impl Executable for DimensionGraphConfigOps {
    fn execute(
        &self,
        _inputs: HashMap<String, gunbc_ir::Value>,
    ) -> Result<HashMap<String, gunbc_ir::Value>, ExecError> {
        match self {
            Self::LoadLlmConfig { provider, model } => OutputMap::new()
                .str("provider", provider.clone())
                .str("model", model.clone())
                .ok(),
            Self::LoadDimensionConfig {
                dimension,
                criteria,
                depth,
            } => {
                let criteria_json = serde_json::to_value(criteria).map_err(|err| {
                    ExecError::new(format!("failed to serialize dimension criteria: {err}"))
                })?;
                OutputMap::new()
                    .str("dimension", dimension.as_str())
                    .str("depth", depth.as_str())
                    .json("criteria", criteria_json)
                    .ok()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cloud credential wiring helpers
// ---------------------------------------------------------------------------

fn add_cloud_credential_chain(
    builder: &mut DagBuilder<ReviewGraphOp>,
    cloud_env: &NodeRef<ReviewGraphOp>,
    resolve_auth: &NodeRef<ReviewGraphOp>,
    cloud_config: &CloudSecretConfig,
) -> Result<NodeRef<ReviewGraphOp>, BuilderError> {
    let bind_secret = builder.add_node_after_all(
        Node::opaque(
            "bind_secret",
            vec![
                port("config", "CloudSecretConfig"),
                port("service", "String"),
                port("secret_name", "String"),
            ],
            vec![port("config", "CloudSecretConfig")],
            DynOp::new(CloudOps::BindSecretName),
        ),
        &[cloud_env, resolve_auth],
    )?;

    builder.add_edge(cloud_env.out("config"), bind_secret.in_port("config"))?;
    builder.add_edge(resolve_auth.out("service"), bind_secret.in_port("service"))?;
    builder.add_edge(
        resolve_auth.out("secret_name"),
        bind_secret.in_port("secret_name"),
    )?;

    let cloud_subdag = lift_cloud_dag(build_cloud_secret_manager_credential_graph_from_config(
        cloud_config,
    )?);
    let cloud_credential =
        builder.add_node_after(Node::subdag("cloud_credential", cloud_subdag), &bind_secret)?;

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
        resolve_auth.out("interactive_allowed"),
        cloud_credential.in_port("interactive_allowed"),
    )?;
    builder.add_edge(
        resolve_auth.out("required_scopes"),
        cloud_credential.in_port("required_scopes"),
    )?;
    builder.add_edge(
        cloud_env.out("request_url"),
        cloud_credential.in_port("request_url"),
    )?;
    builder.add_edge(
        cloud_env.out("request_token"),
        cloud_credential.in_port("request_token"),
    )?;

    Ok(cloud_credential)
}

fn add_scope_preflight_chain(
    builder: &mut DagBuilder<ReviewGraphOp>,
    resolve_auth: &NodeRef<ReviewGraphOp>,
) -> Result<NodeRef<ReviewGraphOp>, BuilderError> {
    let scope_preflight = builder.add_node_after(
        Node::opaque(
            "scope_preflight",
            vec![list("required_scopes", "String")],
            vec![port("scope_verified", "Bool")],
            DynOp::new(CloudOps::ScopePreflight),
        ),
        resolve_auth,
    )?;

    builder.add_edge(
        resolve_auth.out("required_scopes"),
        scope_preflight.in_port("required_scopes"),
    )?;

    Ok(scope_preflight)
}

fn lift_cloud_dag(dag: Dag<CloudSecretManagerGraphOp>) -> Dag<ReviewGraphOp> {
    dag
}

/// Create the `cloud_env` root node using `ConstCloudConfig`.
fn add_cloud_env_node(
    builder: &mut DagBuilder<ReviewGraphOp>,
    cloud_config: &CloudSecretConfig,
) -> Result<NodeRef<ReviewGraphOp>, BuilderError> {
    builder.add_root_node(Node::opaque(
        "cloud_env",
        vec![],
        vec![
            port("config", "CloudSecretConfig"),
            optional("request_url", "OptionalString"),
            optional("request_token", "OptionalString"),
        ],
        DynOp::new(CloudOps::ConstCloudConfig {
            config: cloud_config.clone(),
        }),
    ))
}

// ============================================================================
// ReviewPhase DAG Builder
// ============================================================================

/// Build a ReviewPhase DAG.
///
/// This DAG performs a complete review of a blob source using an LLM.
///
/// ## Entrypoints (unconnected inputs):
/// - `prepare_blob.source`: Json — BlobSource definition
/// - `prepare_prompt.criteria`: Json — Criteria definition
/// - `prepare_llm.provider`: String — LLM provider ID
/// - `prepare_llm.model`: String — LLM model identifier
///
/// ## Boundaries (unconnected outputs):
/// - `parse_response.output`: Json — ReviewOutput
/// - `parse_response.errors`: Json — Parse errors array
/// - `parse_blob.meta`: Json — BlobMeta (for caching)
///
/// ## Internal Flow:
/// ```text
/// prepare_blob → [execute_blob] → parse_blob
///                                     ↓
///                              prepare_prompt
///                                     ↓
///                              prepare_llm → resolve_auth → cloud_credential → [execute_llm] → parse_llm
///                                                                      ↓
///                                                               parse_response
/// ```
///
/// Note: For inline blob sources, execute_blob is skipped (handled by prepare_blob).
/// The graph handles this with conditional execution.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "review-phase",
    builder = "build_review_phase_graph()",
    returns_result
)]
pub fn build_review_phase_graph() -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_review_phase_graph_with_config(graph_cloud_config())
}

/// Build a ReviewPhase DAG with an explicit cloud config.
pub fn build_review_phase_graph_with_config(
    cloud_config: CloudSecretConfig,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        DynOp::new(FsEnv::new(filename::Scope::Write)),
    ))?;

    // Node 0: Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config)?;

    // ========================================================================
    // Blob Acquisition
    // ========================================================================

    // Node 1: PrepareFetch - builds request or returns inline data
    let prepare_blob = builder.add_root_node(Node::opaque(
        "prepare_blob",
        vec![port("source", "Json")],
        vec![
            port("request", "TransportRequest"),
            port("skip_fetch", "Bool"),
            port("skip", "Bool"),
            port("handle", "Json"), // Present if inline
            port("source", "Json"), // Echo for parse
        ],
        DynOp::new(BlobOps::PrepareFetch),
    ))?;

    // Node 2: Execute blob fetch (I/O boundary)
    // Note: This is skipped for inline sources (skip_fetch=true)
    let execute_blob = builder.add_node_after(
        Node::opaque(
            "execute_blob",
            vec![
                port("request", "TransportRequest"),
                port("skip", "Bool"),
                resource("file", "FilesystemHandle", AccessMode::Read),
            ],
            vec![port("response", "TransportResponse")],
            DynOp::new(TransportOps::Execute),
        ),
        &prepare_blob,
    )?;

    // Node 3: ParseFetch - converts response to BlobHandle
    let parse_blob = builder.add_node_after(
        Node::opaque(
            "parse_blob",
            vec![
                port("source", "Json"),
                port("response", "TransportResponse"),
                optional("handle", "OptionalJson"),
                port("skip", "Bool"),
            ],
            vec![port("handle", "Json"), port("meta", "Json")],
            DynOp::new(BlobOps::ParseFetch),
        ),
        &execute_blob,
    )?;

    // ========================================================================
    // Review Prompt Building
    // ========================================================================

    // Node 4: PrepareReviewPrompt - builds question from blob + criteria
    let prepare_prompt = builder.add_root_node(Node::opaque(
        "prepare_prompt",
        vec![
            port("artifact", "String"),
            port("criteria", "Json"),
            optional("context", "OptionalString"),
        ],
        vec![port("question", "String"), port("system_prompt", "String")],
        DynOp::new(ReviewOps::PrepareReviewPrompt),
    ))?;

    // ========================================================================
    // LLM Interaction
    // ========================================================================

    // Resolve auth requirements (pure)
    let resolve_auth = builder.add_node_after(
        Node::opaque(
            "resolve_auth",
            vec![port("provider", "String")],
            vec![
                port("service", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
                list("required_scopes", "String"),
                port("interactive_allowed", "Bool"),
                port("secret_name", "String"),
            ],
            DynOp::new(ReviewOps::ResolveAuthContract),
        ),
        &prepare_prompt,
    )?;

    // Cloud credential acquisition (resolves provider credentials)
    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config)?;
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth)?;

    // LLM triplet SubDag: prepare_llm → execute_llm → parse_llm
    let llm_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "llm",
        "prepare_llm",
        "execute_llm",
        "parse_llm",
        vec![
            port("content", "String"),
            port("question", "String"),
            port("provider", "String"),
            port("model", "String"),
            optional("system_prompt", "OptionalString"),
        ],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![port("provider", "String")],
        vec![port("answer", "String")],
        DynOp::new(LlmOps::PrepareSimpleRequest),
        DynOp::new(LlmOps::ParseSimpleResponse),
        DynOp::new(TransportOps::Execute),
        Some(&cloud_credential),
    )?;
    builder.add_edge(
        scope_preflight.out("scope_verified"),
        llm_triplet.in_port("scope_verified"),
    )?;

    // ========================================================================
    // Review Response Parsing
    // ========================================================================

    // ParseReviewResponse - converts answer to ReviewOutput
    let parse_response = builder.add_node_after(
        Node::opaque(
            "parse_response",
            vec![port("answer", "String"), port("criteria", "Json")],
            vec![port("output", "Json"), port("errors", "Json")],
            DynOp::new(ReviewOps::ParseReviewResponse),
        ),
        &llm_triplet,
    )?;

    // ========================================================================
    // Edges
    // ========================================================================

    // Blob acquisition flow
    builder.add_edge(prepare_blob.out("request"), execute_blob.in_port("request"))?;
    builder.add_edge(prepare_blob.out("skip"), execute_blob.in_port("skip"))?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        execute_blob.in_port("res:file"),
    )?;
    builder.add_edge(execute_blob.out("response"), parse_blob.in_port("response"))?;
    builder.add_edge(prepare_blob.out("source"), parse_blob.in_port("source"))?;
    builder.add_edge(prepare_blob.out("handle"), parse_blob.in_port("handle"))?;
    builder.add_edge(prepare_blob.out("skip"), parse_blob.in_port("skip"))?;

    // LLM flow
    builder.add_edge(
        prepare_prompt.out("question"),
        llm_triplet.in_port("question"),
    )?;
    builder.add_edge(
        prepare_prompt.out("system_prompt"),
        llm_triplet.in_port("system_prompt"),
    )?;
    builder.add_edge(
        cloud_credential.out("credential"),
        llm_triplet.in_port("res:credential"),
    )?;

    // Response parsing
    builder.add_edge(llm_triplet.out("answer"), parse_response.in_port("answer"))?;
    // criteria is an entrypoint, flows to both prepare_prompt and parse_response

    Ok(builder.build())
}

/// Build a simplified ReviewPhase DAG for inline content.
///
/// This version skips blob acquisition - content is provided directly.
///
/// ## Entrypoints:
/// - `prepare_prompt.artifact`: String — content to review
/// - `prepare_prompt.criteria`: Json — Criteria definition
/// - `prepare_llm.provider`: String — LLM provider ID
/// - `prepare_llm.model`: String — LLM model identifier
///
/// ## Boundaries:
/// - `parse_response.output`: Json — ReviewOutput
/// - `parse_response.errors`: Json — Parse errors array
pub fn build_inline_review_graph() -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_inline_review_graph_with_config(graph_cloud_config())
}

/// Build an inline review graph with explicit cloud config.
pub fn build_inline_review_graph_with_config(
    cloud_config: CloudSecretConfig,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    // Node 0: Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config)?;

    // Node 1: PrepareReviewPrompt
    let prepare_prompt = builder.add_root_node(Node::opaque(
        "prepare_prompt",
        vec![
            port("artifact", "String"),
            port("criteria", "Json"),
            optional("context", "OptionalString"),
        ],
        vec![port("question", "String"), port("system_prompt", "String")],
        DynOp::new(ReviewOps::PrepareReviewPrompt),
    ))?;

    // Resolve auth requirements (pure)
    let resolve_auth = builder.add_node_after(
        Node::opaque(
            "resolve_auth",
            vec![port("provider", "String")],
            vec![
                port("service", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
                list("required_scopes", "String"),
                port("interactive_allowed", "Bool"),
                port("secret_name", "String"),
            ],
            DynOp::new(ReviewOps::ResolveAuthContract),
        ),
        &prepare_prompt,
    )?;

    // Cloud credential acquisition (resolves provider credentials)
    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config)?;
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth)?;

    // LLM triplet SubDag: prepare_llm → execute_llm → parse_llm
    let llm_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "llm",
        "prepare_llm",
        "execute_llm",
        "parse_llm",
        vec![
            port("content", "String"),
            port("question", "String"),
            port("provider", "String"),
            port("model", "String"),
            optional("system_prompt", "OptionalString"),
        ],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![port("provider", "String")],
        vec![port("answer", "String")],
        DynOp::new(LlmOps::PrepareSimpleRequest),
        DynOp::new(LlmOps::ParseSimpleResponse),
        DynOp::new(TransportOps::Execute),
        Some(&cloud_credential),
    )?;
    builder.add_edge(
        scope_preflight.out("scope_verified"),
        llm_triplet.in_port("scope_verified"),
    )?;

    // ParseReviewResponse
    let parse_response = builder.add_node_after(
        Node::opaque(
            "parse_response",
            vec![port("answer", "String"), port("criteria", "Json")],
            vec![port("output", "Json"), port("errors", "Json")],
            DynOp::new(ReviewOps::ParseReviewResponse),
        ),
        &llm_triplet,
    )?;

    // Edges
    builder.add_edge(
        prepare_prompt.out("question"),
        llm_triplet.in_port("question"),
    )?;
    builder.add_edge(
        prepare_prompt.out("system_prompt"),
        llm_triplet.in_port("system_prompt"),
    )?;
    builder.add_edge(
        cloud_credential.out("credential"),
        llm_triplet.in_port("res:credential"),
    )?;
    builder.add_edge(llm_triplet.out("answer"), parse_response.in_port("answer"))?;

    Ok(builder.build())
}

// ============================================================================
// DiffReviewPhase DAG Builder (Track 5.2)
// ============================================================================

/// Build a DiffReviewPhase DAG with default pipeline config.
///
/// Uses `ReviewPipelineConfig::gunbc_default()` for provider, model, and criteria.
/// See [`build_diff_review_graph_with`] for full documentation.
pub fn build_diff_review_graph() -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_diff_review_graph_with(ReviewPipelineConfig::gunbc_default())
}

/// Build a DiffReviewPhase DAG.
///
/// Composes GitOps::PrepareDiff → Execute → ParseDiff with an inline
/// review phase. Reviews the unified diff of the current branch against
/// a base ref.
///
/// Provider, model, and criteria are baked into the DAG via a
/// `LoadPipelineConfig` node — they are pipeline decisions, not CLI flags.
///
/// ## Entrypoints (unconnected inputs):
/// - `prepare_diff.base_ref` (optional): String — base ref override
/// - `prepare_diff.repo_path` (required): String — repo path
///
/// ## Boundaries (unconnected outputs):
/// - `parse_response.output`: Json — ReviewOutput
/// - `parse_response.errors`: Json — Parse errors array
/// - `parse_diff.stats`: String — Diff statistics
///
/// ## Internal Flow:
/// ```text
/// [config] ──provider,model,criteria──┐
///                                     ↓
/// prepare_diff → [execute_diff] → parse_diff → format_artifact
///                                                    ↓
///                                             prepare_prompt
///                                                    ↓
///                                             prepare_llm → resolve_auth → cloud_credential → [execute_llm] → parse_llm
///                                                                                     ↓
///                                                                              parse_response
/// ```
///
/// I/O Classification:
/// - Two TransportOps::Execute calls: git diff (read), LLM (read)
/// - Phase overall: Read-only
pub fn build_diff_review_graph_with(
    config: ReviewPipelineConfig,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_diff_review_graph_with_cloud_config(config, graph_cloud_config())
}

/// Build a DiffReviewPhase DAG with explicit cloud config.
pub fn build_diff_review_graph_with_cloud_config(
    config: ReviewPipelineConfig,
    cloud_config: CloudSecretConfig,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        DynOp::new(FsEnv::new(filename::Scope::Write)),
    ))?;

    // Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config)?;

    let default_branch = config.default_branch.clone();

    // ========================================================================
    // Pipeline Config (zero-input node, emits constants)
    // ========================================================================

    let config_node = builder.add_root_node(Node::opaque(
        "config",
        vec![],
        vec![
            port("provider", "String"),
            port("model", "String"),
            port("criteria", "Json"),
        ],
        DynOp::new(ReviewOps::LoadPipelineConfig(config)),
    ))?;

    // ========================================================================
    // Git Diff Acquisition
    // ========================================================================

    // PrepareDiff - builds git diff request.
    // base_ref default comes from pipeline config (GitConfig::default_branch);
    // the optional port allows CLI override.
    let diff_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "diff",
        "prepare_diff",
        "execute_diff",
        "parse_diff",
        vec![
            optional("base_ref", "OptionalString"),
            port("repo_path", "FilePath"),
        ],
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![],
        vec![port("diff_files", "Map"), port("stats", "String")],
        DynOp::new(GitOps::PrepareDiff {
            base_ref: default_branch,
            extensions: vec![],
        }),
        DynOp::new(GitOps::ParseDiff),
        DynOp::new(TransportOps::Execute),
        Some(&fs_env),
    )?;

    // ========================================================================
    // Diff → Artifact Formatting
    // ========================================================================

    let format_artifact = builder.add_node_after(
        Node::opaque(
            "format_artifact",
            vec![port("diff_files", "Map")],
            vec![port("artifact", "String")],
            DynOp::new(ReviewOps::FormatDiffArtifact),
        ),
        &diff_triplet,
    )?;

    // ========================================================================
    // Review Prompt Building
    // ========================================================================

    let prepare_prompt = builder.add_node_after(
        Node::opaque(
            "prepare_prompt",
            vec![
                port("artifact", "String"),
                port("criteria", "Json"),
                optional("context", "OptionalString"),
            ],
            vec![port("question", "String"), port("system_prompt", "String")],
            DynOp::new(ReviewOps::PrepareReviewPrompt),
        ),
        &format_artifact,
    )?;

    // ========================================================================
    // LLM Interaction
    // ========================================================================

    let resolve_auth = builder.add_node_after(
        Node::opaque(
            "resolve_auth",
            vec![port("provider", "String")],
            vec![
                port("service", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
                list("required_scopes", "String"),
                port("interactive_allowed", "Bool"),
                port("secret_name", "String"),
            ],
            DynOp::new(ReviewOps::ResolveAuthContract),
        ),
        &prepare_prompt,
    )?;

    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config)?;
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth)?;

    // LLM triplet SubDag: prepare_llm → execute_llm → parse_llm
    let llm_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "llm",
        "prepare_llm",
        "execute_llm",
        "parse_llm",
        vec![
            port("content", "String"),
            port("question", "String"),
            port("provider", "String"),
            port("model", "String"),
            optional("system_prompt", "OptionalString"),
        ],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![port("provider", "String")],
        vec![port("answer", "String")],
        DynOp::new(LlmOps::PrepareSimpleRequest),
        DynOp::new(LlmOps::ParseSimpleResponse),
        DynOp::new(TransportOps::Execute),
        Some(&cloud_credential),
    )?;
    builder.add_edge(
        scope_preflight.out("scope_verified"),
        llm_triplet.in_port("scope_verified"),
    )?;

    // ========================================================================
    // Review Response Parsing
    // ========================================================================

    let parse_response = builder.add_node_after(
        Node::opaque(
            "parse_response",
            vec![port("answer", "String"), port("criteria", "Json")],
            vec![port("output", "Json"), port("errors", "Json")],
            DynOp::new(ReviewOps::ParseReviewResponse),
        ),
        &llm_triplet,
    )?;

    // ========================================================================
    // Edges
    // ========================================================================

    // Config → downstream consumers
    builder.add_edge(config_node.out("provider"), llm_triplet.in_port("provider"))?;
    builder.add_edge(config_node.out("model"), llm_triplet.in_port("model"))?;
    builder.add_edge(
        config_node.out("criteria"),
        prepare_prompt.in_port("criteria"),
    )?;
    builder.add_edge(
        config_node.out("criteria"),
        parse_response.in_port("criteria"),
    )?;
    builder.add_edge(
        config_node.out("provider"),
        resolve_auth.in_port("provider"),
    )?;

    // Diff → artifact formatting
    builder.add_edge(
        diff_triplet.out("diff_files"),
        format_artifact.in_port("diff_files"),
    )?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        diff_triplet.in_port("res:file"),
    )?;

    // Artifact → review prompt + LLM content
    builder.add_edge(
        format_artifact.out("artifact"),
        prepare_prompt.in_port("artifact"),
    )?;
    builder.add_edge(
        format_artifact.out("artifact"),
        llm_triplet.in_port("content"),
    )?;

    // LLM flow
    builder.add_edge(
        prepare_prompt.out("question"),
        llm_triplet.in_port("question"),
    )?;
    builder.add_edge(
        prepare_prompt.out("system_prompt"),
        llm_triplet.in_port("system_prompt"),
    )?;
    builder.add_edge(
        cloud_credential.out("credential"),
        llm_triplet.in_port("res:credential"),
    )?;

    // Response parsing
    builder.add_edge(llm_triplet.out("answer"), parse_response.in_port("answer"))?;

    Ok(builder.build())
}

// ============================================================================
// MultiSourceReviewPhase DAG Builder (Track 5.3)
// ============================================================================

/// Build a MultiSourceReviewPhase DAG.
///
/// Performs a review using LLM analysis, then merges outputs.
/// This is designed to be extended with additional review sources
/// (cargo check, clippy) as separate sub-DAGs in the future.
///
/// Provider, model, and criteria are baked in via `LoadPipelineConfig`.
///
/// ## Entrypoints (unconnected inputs):
/// - `prepare_prompt.artifact`: String — content to review
///
/// ## Boundaries (unconnected outputs):
/// - `merge.bundle`: Json — ReviewBundle with merged findings
/// - `merge.conflicts`: Json — Finding ID conflicts
///
/// ## Internal Flow:
/// ```text
/// [config] ──provider,model,criteria──┐
///                                     ↓
///                  ┌──▶ [LLM Review] ──────┐
///  artifact ───────┤                        ▼
///                  └──(future sources)──▶ [MergeOutputs]
/// ```
///
/// MergeOutputs declares a list port for `outputs` — the engine collects
/// fan-in edges into `Value::List` automatically. Each source wires
/// directly to the merge node without wrapper nodes.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "review-multi-source",
    builder = "build_multi_source_review_graph()",
    returns_result
)]
pub fn build_multi_source_review_graph() -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_multi_source_review_graph_with(ReviewPipelineConfig::gunbc_default())
}

/// Build a MultiSourceReviewPhase DAG with explicit pipeline config.
pub fn build_multi_source_review_graph_with(
    config: ReviewPipelineConfig,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_multi_source_review_graph_with_cloud_config(config, graph_cloud_config())
}

/// Build a MultiSourceReviewPhase DAG with explicit pipeline and cloud configs.
pub fn build_multi_source_review_graph_with_cloud_config(
    config: ReviewPipelineConfig,
    cloud_config: CloudSecretConfig,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    // Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config)?;

    // ========================================================================
    // Pipeline Config
    // ========================================================================

    let config_node = builder.add_root_node(Node::opaque(
        "config",
        vec![],
        vec![
            port("provider", "String"),
            port("model", "String"),
            port("criteria", "Json"),
        ],
        DynOp::new(ReviewOps::LoadPipelineConfig(config)),
    ))?;

    // ========================================================================
    // LLM Review Source (source 1)
    // ========================================================================

    let prepare_prompt = builder.add_node_after(
        Node::opaque(
            "prepare_prompt",
            vec![
                port("artifact", "String"),
                port("criteria", "Json"),
                optional("context", "OptionalString"),
            ],
            vec![port("question", "String"), port("system_prompt", "String")],
            DynOp::new(ReviewOps::PrepareReviewPrompt),
        ),
        &config_node,
    )?;

    let resolve_auth = builder.add_node_after(
        Node::opaque(
            "resolve_auth",
            vec![port("provider", "String")],
            vec![
                port("service", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
                list("required_scopes", "String"),
                port("interactive_allowed", "Bool"),
                port("secret_name", "String"),
            ],
            DynOp::new(ReviewOps::ResolveAuthContract),
        ),
        &prepare_prompt,
    )?;

    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config)?;
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth)?;

    // LLM triplet SubDag: prepare_llm → execute_llm → parse_llm
    let llm_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "llm",
        "prepare_llm",
        "execute_llm",
        "parse_llm",
        vec![
            port("content", "String"),
            port("question", "String"),
            port("provider", "String"),
            port("model", "String"),
            optional("system_prompt", "OptionalString"),
        ],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![port("provider", "String")],
        vec![port("answer", "String")],
        DynOp::new(LlmOps::PrepareSimpleRequest),
        DynOp::new(LlmOps::ParseSimpleResponse),
        DynOp::new(TransportOps::Execute),
        Some(&cloud_credential),
    )?;
    builder.add_edge(
        scope_preflight.out("scope_verified"),
        llm_triplet.in_port("scope_verified"),
    )?;

    let parse_response = builder.add_node_after(
        Node::opaque(
            "parse_response",
            vec![port("answer", "String"), port("criteria", "Json")],
            vec![port("output", "Json"), port("errors", "Json")],
            DynOp::new(ReviewOps::ParseReviewResponse),
        ),
        &llm_triplet,
    )?;

    // ========================================================================
    // Merge (combines sources)
    // ========================================================================

    let merge = builder.add_node_after(
        Node::opaque(
            "merge",
            vec![list("outputs", "JsonList")],
            vec![port("bundle", "Json"), port("conflicts", "Json")],
            DynOp::new(ReviewOps::MergeOutputs),
        ),
        &parse_response,
    )?;

    // ========================================================================
    // Edges
    // ========================================================================

    // Config → downstream consumers
    builder.add_edge(config_node.out("provider"), llm_triplet.in_port("provider"))?;
    builder.add_edge(config_node.out("model"), llm_triplet.in_port("model"))?;
    builder.add_edge(
        config_node.out("criteria"),
        prepare_prompt.in_port("criteria"),
    )?;
    builder.add_edge(
        config_node.out("criteria"),
        parse_response.in_port("criteria"),
    )?;
    builder.add_edge(
        config_node.out("provider"),
        resolve_auth.in_port("provider"),
    )?;

    // LLM review flow
    builder.add_edge(
        prepare_prompt.out("question"),
        llm_triplet.in_port("question"),
    )?;
    builder.add_edge(
        prepare_prompt.out("system_prompt"),
        llm_triplet.in_port("system_prompt"),
    )?;
    builder.add_edge(
        cloud_credential.out("credential"),
        llm_triplet.in_port("res:credential"),
    )?;
    builder.add_edge(llm_triplet.out("answer"), parse_response.in_port("answer"))?;

    // Review output → merge (list port collects fan-in automatically)
    builder.add_edge(parse_response.out("output"), merge.in_port("outputs"))?;

    Ok(builder.build())
}

fn merge_input_port_for_dimension(dimension: ReviewDimension) -> &'static str {
    match dimension {
        ReviewDimension::Coherence => "coherence_output",
        ReviewDimension::Quality => "quality_output",
        ReviewDimension::Requirements => "requirements_output",
        ReviewDimension::Aspirational => "aspirational_output",
    }
}

// ============================================================================
// Dimension Diff Review Graph (W4)
// ============================================================================

/// Build a 4-dimension diff review graph with the default coding profile.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "review-diff-dimensions",
    builder = "build_dimension_diff_review_graph()",
    returns_result
)]
pub fn build_dimension_diff_review_graph() -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_dimension_diff_review_graph_with(
        ReviewPipelineConfig::gunbc_default(),
        coding_review_profile(FermiDepth::M),
    )
}

/// Build a 4-dimension diff review graph with explicit pipeline config + profile.
pub fn build_dimension_diff_review_graph_with(
    config: ReviewPipelineConfig,
    profile: ReviewProfile,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    build_dimension_diff_review_graph_with_cloud_config(config, profile, graph_cloud_config())
}

/// Build a 4-dimension diff review graph with explicit cloud config.
///
/// Parallel first-wave dimensions: coherence, quality, requirements.
/// Aspirational runs last and receives prior findings context.
pub fn build_dimension_diff_review_graph_with_cloud_config(
    config: ReviewPipelineConfig,
    profile: ReviewProfile,
    cloud_config: CloudSecretConfig,
) -> Result<Dag<ReviewGraphOp>, BuilderError> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    let fs_env = builder.add_root_node(Node::opaque(
        "fs_env",
        vec![],
        vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
        DynOp::new(FsEnv::new(filename::Scope::Write)),
    ))?;

    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config)?;
    let default_branch = config.default_branch.clone();

    let diff_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "diff",
        "prepare_diff",
        "execute_diff",
        "parse_diff",
        vec![
            optional("base_ref", "OptionalString"),
            port("repo_path", "FilePath"),
        ],
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![],
        vec![port("diff_files", "Map"), port("stats", "String")],
        DynOp::new(GitOps::PrepareDiff {
            base_ref: default_branch,
            extensions: vec![],
        }),
        DynOp::new(GitOps::ParseDiff),
        DynOp::new(TransportOps::Execute),
        Some(&fs_env),
    )?;

    let format_artifact = builder.add_node_after(
        Node::opaque(
            "format_artifact",
            vec![port("diff_files", "Map")],
            vec![port("artifact", "String")],
            DynOp::new(ReviewOps::FormatDiffArtifact),
        ),
        &diff_triplet,
    )?;

    let llm_config = builder.add_root_node(Node::opaque(
        "llm_config",
        vec![],
        vec![port("provider", "String"), port("model", "String")],
        DynOp::new(DimensionGraphConfigOps::LoadLlmConfig {
            provider: config.provider,
            model: config.model,
        }),
    ))?;

    let resolve_auth = builder.add_node_after(
        Node::opaque(
            "resolve_auth",
            vec![port("provider", "String")],
            vec![
                port("service", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
                list("required_scopes", "String"),
                port("interactive_allowed", "Bool"),
                port("secret_name", "String"),
            ],
            DynOp::new(ReviewOps::ResolveAuthContract),
        ),
        &llm_config,
    )?;
    builder.add_edge(llm_config.out("provider"), resolve_auth.in_port("provider"))?;

    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config)?;
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth)?;

    let mut merge_inputs: Vec<(NodeRef<ReviewGraphOp>, &'static str)> = Vec::new();
    let mut parallel_dimension_outputs: Vec<(ReviewDimension, NodeRef<ReviewGraphOp>)> = Vec::new();

    for &dimension in ReviewDimension::parallel_dimensions() {
        let Some(criteria) = profile.criteria_for(dimension).cloned() else {
            continue;
        };
        let depth = profile.depth_for(dimension);
        let prefix = dimension.as_str();

        let dimension_config = builder.add_root_node(Node::opaque(
            format!("{prefix}_config"),
            vec![],
            vec![
                port("criteria", "Json"),
                port("dimension", "String"),
                port("depth", "String"),
            ],
            DynOp::new(DimensionGraphConfigOps::LoadDimensionConfig {
                dimension,
                criteria,
                depth,
            }),
        ))?;

        let prepare_prompt = builder.add_node_after(
            Node::opaque(
                format!("{prefix}_prepare_prompt"),
                vec![
                    port("artifact", "String"),
                    port("criteria", "Json"),
                    port("dimension", "String"),
                    port("depth", "String"),
                    optional("context", "OptionalString"),
                    optional("prior_findings", "OptionalString"),
                ],
                vec![port("question", "String"), port("system_prompt", "String")],
                DynOp::new(DimensionOps::PrepareDimensionPrompt),
            ),
            &format_artifact,
        )?;

        let llm_triplet = add_transport_triplet_named_with_passthrough(
            &mut builder,
            format!("{prefix}_llm").as_str(),
            format!("prepare_{prefix}_llm").as_str(),
            format!("execute_{prefix}_llm").as_str(),
            format!("parse_{prefix}_llm").as_str(),
            vec![
                port("content", "String"),
                port("question", "String"),
                port("provider", "String"),
                port("model", "String"),
                optional("system_prompt", "OptionalString"),
            ],
            vec![
                optional("scope_verified", "OptionalBool"),
                resource("credential", "Credential", AccessMode::Read),
            ],
            vec![port("provider", "String")],
            vec![port("answer", "String")],
            DynOp::new(LlmOps::PrepareSimpleRequest),
            DynOp::new(LlmOps::ParseSimpleResponse),
            DynOp::new(TransportOps::Execute),
            Some(&cloud_credential),
        )?;

        let parse_response = builder.add_node_after(
            Node::opaque(
                format!("{prefix}_parse_response"),
                vec![
                    port("answer", "String"),
                    port("criteria", "Json"),
                    port("dimension", "String"),
                ],
                vec![port("output", "Json"), port("errors", "Json")],
                DynOp::new(DimensionOps::ParseDimensionResponse),
            ),
            &llm_triplet,
        )?;

        builder.add_edge(
            dimension_config.out("criteria"),
            prepare_prompt.in_port("criteria"),
        )?;
        builder.add_edge(
            dimension_config.out("criteria"),
            parse_response.in_port("criteria"),
        )?;
        builder.add_edge(
            dimension_config.out("dimension"),
            prepare_prompt.in_port("dimension"),
        )?;
        builder.add_edge(
            dimension_config.out("dimension"),
            parse_response.in_port("dimension"),
        )?;
        builder.add_edge(
            dimension_config.out("depth"),
            prepare_prompt.in_port("depth"),
        )?;
        builder.add_edge(
            format_artifact.out("artifact"),
            prepare_prompt.in_port("artifact"),
        )?;
        builder.add_edge(
            format_artifact.out("artifact"),
            llm_triplet.in_port("content"),
        )?;
        builder.add_edge(
            prepare_prompt.out("question"),
            llm_triplet.in_port("question"),
        )?;
        builder.add_edge(
            prepare_prompt.out("system_prompt"),
            llm_triplet.in_port("system_prompt"),
        )?;
        builder.add_edge(llm_config.out("provider"), llm_triplet.in_port("provider"))?;
        builder.add_edge(llm_config.out("model"), llm_triplet.in_port("model"))?;
        builder.add_edge(
            cloud_credential.out("credential"),
            llm_triplet.in_port("res:credential"),
        )?;
        builder.add_edge(
            scope_preflight.out("scope_verified"),
            llm_triplet.in_port("scope_verified"),
        )?;
        builder.add_edge(llm_triplet.out("answer"), parse_response.in_port("answer"))?;
        merge_inputs.push((
            parse_response.clone(),
            merge_input_port_for_dimension(dimension),
        ));

        parallel_dimension_outputs.push((dimension, parse_response));
    }

    if let Some(criteria) = profile.criteria_for(ReviewDimension::Aspirational).cloned() {
        let depth = profile.depth_for(ReviewDimension::Aspirational);

        let prior_node_def = Node::opaque(
            "format_prior_findings",
            vec![
                optional("coherence_output", "OptionalJson"),
                optional("quality_output", "OptionalJson"),
                optional("requirements_output", "OptionalJson"),
            ],
            vec![port("prior_findings", "String")],
            DynOp::new(DimensionOps::FormatPriorFindings),
        );

        let format_prior = if parallel_dimension_outputs.is_empty() {
            builder.add_root_node(prior_node_def)?
        } else {
            let parent_refs: Vec<&NodeRef<ReviewGraphOp>> = parallel_dimension_outputs
                .iter()
                .map(|(_, node)| node)
                .collect();
            builder.add_node_after_all(prior_node_def, &parent_refs)?
        };

        for (dimension, node) in &parallel_dimension_outputs {
            builder.add_edge(
                node.out("output"),
                format_prior.in_port(merge_input_port_for_dimension(*dimension)),
            )?;
        }

        let aspirational_config = builder.add_root_node(Node::opaque(
            "aspirational_config",
            vec![],
            vec![
                port("criteria", "Json"),
                port("dimension", "String"),
                port("depth", "String"),
            ],
            DynOp::new(DimensionGraphConfigOps::LoadDimensionConfig {
                dimension: ReviewDimension::Aspirational,
                criteria,
                depth,
            }),
        ))?;

        let aspirational_prepare = builder.add_node_after_all(
            Node::opaque(
                "aspirational_prepare_prompt",
                vec![
                    port("artifact", "String"),
                    port("criteria", "Json"),
                    port("dimension", "String"),
                    port("depth", "String"),
                    optional("context", "OptionalString"),
                    optional("prior_findings", "OptionalString"),
                ],
                vec![port("question", "String"), port("system_prompt", "String")],
                DynOp::new(DimensionOps::PrepareDimensionPrompt),
            ),
            &[&format_artifact, &format_prior],
        )?;

        let aspirational_llm = add_transport_triplet_named_with_passthrough(
            &mut builder,
            "aspirational_llm",
            "prepare_aspirational_llm",
            "execute_aspirational_llm",
            "parse_aspirational_llm",
            vec![
                port("content", "String"),
                port("question", "String"),
                port("provider", "String"),
                port("model", "String"),
                optional("system_prompt", "OptionalString"),
            ],
            vec![
                optional("scope_verified", "OptionalBool"),
                resource("credential", "Credential", AccessMode::Read),
            ],
            vec![port("provider", "String")],
            vec![port("answer", "String")],
            DynOp::new(LlmOps::PrepareSimpleRequest),
            DynOp::new(LlmOps::ParseSimpleResponse),
            DynOp::new(TransportOps::Execute),
            Some(&aspirational_prepare),
        )?;

        let aspirational_parse = builder.add_node_after(
            Node::opaque(
                "aspirational_parse_response",
                vec![
                    port("answer", "String"),
                    port("criteria", "Json"),
                    port("dimension", "String"),
                ],
                vec![port("output", "Json"), port("errors", "Json")],
                DynOp::new(DimensionOps::ParseDimensionResponse),
            ),
            &aspirational_llm,
        )?;

        builder.add_edge(
            aspirational_config.out("criteria"),
            aspirational_prepare.in_port("criteria"),
        )?;
        builder.add_edge(
            aspirational_config.out("criteria"),
            aspirational_parse.in_port("criteria"),
        )?;
        builder.add_edge(
            aspirational_config.out("dimension"),
            aspirational_prepare.in_port("dimension"),
        )?;
        builder.add_edge(
            aspirational_config.out("dimension"),
            aspirational_parse.in_port("dimension"),
        )?;
        builder.add_edge(
            aspirational_config.out("depth"),
            aspirational_prepare.in_port("depth"),
        )?;
        builder.add_edge(
            format_artifact.out("artifact"),
            aspirational_prepare.in_port("artifact"),
        )?;
        builder.add_edge(
            format_prior.out("prior_findings"),
            aspirational_prepare.in_port("prior_findings"),
        )?;
        builder.add_edge(
            format_artifact.out("artifact"),
            aspirational_llm.in_port("content"),
        )?;
        builder.add_edge(
            aspirational_prepare.out("question"),
            aspirational_llm.in_port("question"),
        )?;
        builder.add_edge(
            aspirational_prepare.out("system_prompt"),
            aspirational_llm.in_port("system_prompt"),
        )?;
        builder.add_edge(
            llm_config.out("provider"),
            aspirational_llm.in_port("provider"),
        )?;
        builder.add_edge(llm_config.out("model"), aspirational_llm.in_port("model"))?;
        builder.add_edge(
            cloud_credential.out("credential"),
            aspirational_llm.in_port("res:credential"),
        )?;
        builder.add_edge(
            scope_preflight.out("scope_verified"),
            aspirational_llm.in_port("scope_verified"),
        )?;
        builder.add_edge(
            aspirational_llm.out("answer"),
            aspirational_parse.in_port("answer"),
        )?;
        merge_inputs.push((aspirational_parse, "aspirational_output"));
    }

    builder.add_edge(
        diff_triplet.out("diff_files"),
        format_artifact.in_port("diff_files"),
    )?;
    builder.add_edge(
        fs_env.out(FsEnv::WRITE_PORT),
        diff_triplet.in_port("res:file"),
    )?;

    let merge_node_def = Node::opaque(
        "merge_dimensions",
        vec![
            optional("coherence_output", "OptionalJson"),
            optional("quality_output", "OptionalJson"),
            optional("requirements_output", "OptionalJson"),
            optional("aspirational_output", "OptionalJson"),
        ],
        vec![port("output", "Json"), port("summary", "String")],
        DynOp::new(DimensionOps::MergeDimensionOutputs),
    );
    let merge = if merge_inputs.is_empty() {
        builder.add_root_node(merge_node_def)?
    } else {
        let parent_refs: Vec<&NodeRef<ReviewGraphOp>> =
            merge_inputs.iter().map(|(node, _)| node).collect();
        builder.add_node_after_all(merge_node_def, &parent_refs)?
    };
    for (source_node, merge_port) in &merge_inputs {
        builder.add_edge(source_node.out("output"), merge.in_port(*merge_port))?;
    }

    Ok(builder.build())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_exec::Executable;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};
    use std::collections::HashMap;

    #[test]
    fn test_review_phase_graph_boundaries() {
        let dag = build_review_phase_graph().unwrap();
        let boundaries = detect_boundaries(&dag);

        // parse_response outputs are boundaries
        assert!(
            boundaries.is_boundary_node(&"parse_response".into()),
            "parse_response should be a boundary"
        );

        // parse_blob.meta is also a boundary
        assert!(
            boundaries.is_boundary_node(&"parse_blob".into()),
            "parse_blob should have boundary outputs"
        );
    }

    #[test]
    fn test_review_phase_graph_entrypoints() {
        let dag = build_review_phase_graph().unwrap();
        let entrypoints = detect_entrypoints(&dag);

        // prepare_blob.source is an entrypoint
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_blob".into()),
            "prepare_blob should have entrypoints"
        );

        // prepare_prompt.criteria is an entrypoint
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_prompt".into()),
            "prepare_prompt should have entrypoints"
        );

        // llm SubDag has provider/model/content as entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"llm".into()),
            "llm should have entrypoints"
        );
    }

    #[test]
    fn test_inline_review_graph_boundaries() {
        let dag = build_inline_review_graph().unwrap();
        let boundaries = detect_boundaries(&dag);

        assert!(
            boundaries.is_boundary_node(&"parse_response".into()),
            "parse_response should be a boundary"
        );
    }

    #[test]
    fn test_inline_review_graph_entrypoints() {
        let dag = build_inline_review_graph().unwrap();
        let entrypoints = detect_entrypoints(&dag);

        // prepare_prompt.artifact and criteria are entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_prompt".into()),
            "prepare_prompt should have entrypoints"
        );

        // llm SubDag has provider/model/content as entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"llm".into()),
            "llm should have entrypoints"
        );
    }

    #[test]
    fn test_review_graph_ops_execute() {
        // Test that all ops can be executed (basic smoke test)
        let ops = vec![
            DynOp::new(BlobOps::PrepareFetch),
            DynOp::new(GitOps::ParseDiff),
            DynOp::new(ReviewOps::HashFinding),
            DynOp::new(LlmOps::PrepareSimpleRequest),
            DynOp::new(LlmOps::ResolveAuth),
            DynOp::new(CloudOps::ResolveConfig),
            DynOp::new(TransportOps::Execute),
        ];

        for op in ops {
            // Just verify the match arms work - actual execution will fail
            // due to missing inputs, which is expected
            let result = op.execute(HashMap::new());
            assert!(result.is_err(), "should fail with empty inputs");
        }
    }

    // ========================================================================
    // DiffReviewPhase tests
    // ========================================================================

    #[test]
    fn test_diff_review_graph_boundaries() {
        let dag = build_diff_review_graph().unwrap();
        let boundaries = detect_boundaries(&dag);

        assert!(
            boundaries.is_boundary_node(&"parse_response".into()),
            "parse_response should be a boundary"
        );

        // stats from diff SubDag is also an unconnected output
        assert!(
            boundaries.is_boundary_node(&"diff".into()),
            "diff.stats should be a boundary"
        );
    }

    #[test]
    fn test_diff_review_graph_entrypoints() {
        let dag = build_diff_review_graph().unwrap();
        let entrypoints = detect_entrypoints(&dag);

        // diff SubDag has base_ref (optional) and repo_path (required) entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"diff".into()),
            "diff should have entrypoints"
        );

        // provider, model are NOT entrypoints — config node provides them
        assert!(
            !entrypoints.is_entrypoint_node(&"llm".into()),
            "llm should NOT have entrypoints (config provides provider/model)"
        );

        // prepare_prompt.context is optional and unconnected → entrypoint.
        // But criteria is NOT an entrypoint (comes from config node).
        assert!(
            !entrypoints.is_entrypoint_port(&"prepare_prompt".into(), &"criteria".into()),
            "criteria should NOT be an entrypoint (config provides it)"
        );
        assert!(
            entrypoints.is_entrypoint_port(&"prepare_prompt".into(), &"context".into()),
            "context should still be an entrypoint (optional, unconnected)"
        );
    }

    #[test]
    fn test_diff_review_graph_has_two_transport_subdags() {
        let dag = build_diff_review_graph().unwrap();
        for node_id in ["diff", "llm"] {
            let node = dag
                .get_node(&node_id.into())
                .unwrap_or_else(|| panic!("missing subdag node: {}", node_id));
            assert!(node.is_subdag(), "{} should be a subdag node", node_id);
        }
    }

    // ========================================================================
    // MultiSourceReviewPhase tests
    // ========================================================================

    #[test]
    fn test_multi_source_review_graph_boundaries() {
        let dag = build_multi_source_review_graph().unwrap();
        let boundaries = detect_boundaries(&dag);

        assert!(
            boundaries.is_boundary_node(&"merge".into()),
            "merge should be a boundary (bundle + conflicts outputs)"
        );
    }

    #[test]
    fn test_multi_source_review_graph_entrypoints() {
        let dag = build_multi_source_review_graph().unwrap();
        let entrypoints = detect_entrypoints(&dag);

        // Only artifact on prepare_prompt is an entrypoint (criteria comes from config)
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_prompt".into()),
            "prepare_prompt should have entrypoints (artifact)"
        );

        // provider/model/content on llm SubDag — only content is an entrypoint
        assert!(
            entrypoints.is_entrypoint_node(&"llm".into()),
            "llm should have content as entrypoint"
        );
    }

    // ========================================================================
    // Dimension Diff Review Graph (W4) tests
    // ========================================================================

    #[test]
    fn test_dimension_diff_review_graph_boundaries() {
        let dag = build_dimension_diff_review_graph().unwrap();
        let boundaries = detect_boundaries(&dag);

        assert!(
            boundaries.is_boundary_node(&"merge_dimensions".into()),
            "merge_dimensions should be a boundary (output + summary)"
        );
        assert!(
            boundaries.is_boundary_node(&"diff".into()),
            "diff stats should remain a boundary"
        );
    }

    #[test]
    fn test_dimension_diff_review_graph_has_dimension_llm_subdags() {
        let dag = build_dimension_diff_review_graph().unwrap();
        for node_id in [
            "coherence_llm",
            "quality_llm",
            "requirements_llm",
            "aspirational_llm",
        ] {
            let node = dag
                .get_node(&node_id.into())
                .unwrap_or_else(|| panic!("missing subdag node: {}", node_id));
            assert!(node.is_subdag(), "{} should be a subdag node", node_id);
        }
    }

    #[test]
    fn test_dimension_diff_review_graph_respects_profile_opt_out() {
        let mut profile = coding_review_profile(FermiDepth::M);
        profile.dimension_criteria.remove("quality");
        profile.dimension_criteria.remove("requirements");

        let dag =
            build_dimension_diff_review_graph_with(ReviewPipelineConfig::gunbc_default(), profile)
                .expect("dimension graph should build with partial profile");

        assert!(dag.get_node(&"coherence_llm".into()).is_some());
        assert!(dag.get_node(&"aspirational_llm".into()).is_some());
        assert!(dag.get_node(&"quality_llm".into()).is_none());
        assert!(dag.get_node(&"requirements_llm".into()).is_none());
    }
}
