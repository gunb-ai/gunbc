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

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::{
    add_transport_triplet_named_with_passthrough, build::*, Dag, DagBuilder, Node, NodeRef, Value,
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
use std::collections::HashMap;

use crate::{ReviewOps, ReviewPipelineConfig};

// ============================================================================
// Unified Operation Type
// ============================================================================

/// Operation type for review phase graphs.
///
/// Union of all ops needed for a complete review workflow.
#[derive(Debug, Clone)]
pub enum ReviewGraphOp {
    /// Blob acquisition operations (PURE)
    Blob(BlobOps),
    /// Git operations (PURE)
    Git(GitOps),
    /// Review-specific operations (PURE)
    Review(ReviewOps),
    /// LLM chat operations (PURE)
    Llm(LlmOps),
    /// Filesystem environment (resource acquisition)
    FsEnv(FsEnv),
    /// Cloud credential flow (GCP/AWS/Azure graph)
    Cloud(CloudSecretManagerGraphOp),
    /// Transport execution (BOUNDARY - actual I/O)
    Transport(TransportOps),
}

impl Executable for ReviewGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            ReviewGraphOp::Blob(op) => op.execute(inputs),
            ReviewGraphOp::Git(op) => op.execute(inputs),
            ReviewGraphOp::Review(op) => op.execute(inputs),
            ReviewGraphOp::Llm(op) => op.execute(inputs),
            ReviewGraphOp::FsEnv(op) => op.execute(inputs),
            ReviewGraphOp::Cloud(op) => op.execute(inputs),
            ReviewGraphOp::Transport(op) => op.execute(inputs),
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
) -> NodeRef<ReviewGraphOp> {
    let bind_secret = builder
        .add_node_after_all(
            Node::opaque(
                "bind_secret",
                vec![
                    port("config", "CloudSecretConfig"),
                    port("service", "String"),
                    optional("secret_name", "OptionalString"),
                ],
                vec![port("config", "CloudSecretConfig")],
                ReviewGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::BindSecretName)),
            ),
            &[cloud_env, resolve_auth],
        )
        .expect("bind_secret node");

    builder
        .add_edge(cloud_env.out("config"), bind_secret.in_port("config"))
        .expect("cloud_env.config -> bind_secret.config");
    builder
        .add_edge(resolve_auth.out("service"), bind_secret.in_port("service"))
        .expect("resolve_auth.service -> bind_secret.service");

    let cloud_subdag = lift_cloud_dag(build_cloud_secret_manager_credential_graph_from_config(
        cloud_config,
    ));
    let cloud_credential = builder
        .add_node_after(Node::subdag("cloud_credential", cloud_subdag), &bind_secret)
        .expect("cloud_credential node");

    builder
        .add_edge(
            bind_secret.out("config"),
            cloud_credential.in_port("config"),
        )
        .expect("bind_secret.config -> cloud_credential.config");
    builder
        .add_edge(
            resolve_auth.out("service"),
            cloud_credential.in_port("source_id"),
        )
        .expect("resolve_auth.service -> cloud_credential.source_id");
    builder
        .add_edge(
            resolve_auth.out("scheme"),
            cloud_credential.in_port("scheme"),
        )
        .expect("resolve_auth.scheme -> cloud_credential.scheme");
    builder
        .add_edge(
            resolve_auth.out("header_name"),
            cloud_credential.in_port("header_name"),
        )
        .expect("resolve_auth.header_name -> cloud_credential.header_name");
    builder
        .add_edge(
            resolve_auth.out("interactive_allowed"),
            cloud_credential.in_port("interactive_allowed"),
        )
        .expect("resolve_auth.interactive_allowed -> cloud_credential.interactive_allowed");
    builder
        .add_edge(
            resolve_auth.out("required_scopes"),
            cloud_credential.in_port("required_scopes"),
        )
        .expect("resolve_auth.required_scopes -> cloud_credential.required_scopes");
    builder
        .add_edge(
            cloud_env.out("request_url"),
            cloud_credential.in_port("request_url"),
        )
        .expect("cloud_env.request_url -> cloud_credential.request_url");
    builder
        .add_edge(
            cloud_env.out("request_token"),
            cloud_credential.in_port("request_token"),
        )
        .expect("cloud_env.request_token -> cloud_credential.request_token");

    cloud_credential
}

fn add_scope_preflight_chain(
    builder: &mut DagBuilder<ReviewGraphOp>,
    resolve_auth: &NodeRef<ReviewGraphOp>,
) -> NodeRef<ReviewGraphOp> {
    let scope_preflight = builder
        .add_node_after(
            Node::opaque(
                "scope_preflight",
                vec![list("required_scopes", "String")],
                vec![port("scope_verified", "Bool")],
                ReviewGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::ScopePreflight)),
            ),
            resolve_auth,
        )
        .expect("scope_preflight node");

    builder
        .add_edge(
            resolve_auth.out("required_scopes"),
            scope_preflight.in_port("required_scopes"),
        )
        .expect("resolve_auth.required_scopes -> scope_preflight.required_scopes");

    scope_preflight
}

fn lift_cloud_dag(dag: Dag<CloudSecretManagerGraphOp>) -> Dag<ReviewGraphOp> {
    dag.map_ops(&mut ReviewGraphOp::Cloud)
}

/// Create the `cloud_env` root node using `ConstCloudConfig`.
fn add_cloud_env_node(
    builder: &mut DagBuilder<ReviewGraphOp>,
    cloud_config: &CloudSecretConfig,
) -> NodeRef<ReviewGraphOp> {
    builder
        .add_root_node(Node::opaque(
            "cloud_env",
            vec![],
            vec![
                port("config", "CloudSecretConfig"),
                optional("request_url", "OptionalString"),
                optional("request_token", "OptionalString"),
            ],
            ReviewGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(
                CloudOps::ConstCloudConfig {
                    config: cloud_config.clone(),
                },
            )),
        ))
        .expect("cloud_env node")
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
    builder = "build_review_phase_graph()"
)]
pub fn build_review_phase_graph() -> Dag<ReviewGraphOp> {
    build_review_phase_graph_with_config(graph_cloud_config())
}

/// Build a ReviewPhase DAG with an explicit cloud config.
pub fn build_review_phase_graph_with_config(cloud_config: CloudSecretConfig) -> Dag<ReviewGraphOp> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    let fs_env = builder
        .add_root_node(Node::opaque(
            "fs_env",
            vec![],
            vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
            ReviewGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
        ))
        .expect("fs_env node");

    // Node 0: Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config);

    // ========================================================================
    // Blob Acquisition
    // ========================================================================

    // Node 1: PrepareFetch - builds request or returns inline data
    let prepare_blob = builder
        .add_root_node(Node::opaque(
            "prepare_blob",
            vec![port("source", "Json")],
            vec![
                port("request", "TransportRequest"),
                port("skip_fetch", "Bool"),
                port("skip", "Bool"),
                port("handle", "Json"), // Present if inline
                port("source", "Json"), // Echo for parse
            ],
            ReviewGraphOp::Blob(BlobOps::PrepareFetch),
        ))
        .expect("prepare_blob node");

    // Node 2: Execute blob fetch (I/O boundary)
    // Note: This is skipped for inline sources (skip_fetch=true)
    let execute_blob = builder
        .add_node_after(
            Node::opaque(
                "execute_blob",
                vec![
                    port("request", "TransportRequest"),
                    port("skip", "Bool"),
                    resource("file", "FilesystemHandle", AccessMode::Read),
                ],
                vec![port("response", "TransportResponse")],
                ReviewGraphOp::Transport(TransportOps::Execute),
            ),
            &prepare_blob,
        )
        .expect("execute_blob node");

    // Node 3: ParseFetch - converts response to BlobHandle
    let parse_blob = builder
        .add_node_after(
            Node::opaque(
                "parse_blob",
                vec![
                    port("source", "Json"),
                    port("response", "TransportResponse"),
                    optional("handle", "OptionalJson"),
                    port("skip", "Bool"),
                ],
                vec![port("handle", "Json"), port("meta", "Json")],
                ReviewGraphOp::Blob(BlobOps::ParseFetch),
            ),
            &execute_blob,
        )
        .expect("parse_blob node");

    // ========================================================================
    // Review Prompt Building
    // ========================================================================

    // Node 4: PrepareReviewPrompt - builds question from blob + criteria
    let prepare_prompt = builder
        .add_root_node(Node::opaque(
            "prepare_prompt",
            vec![
                port("artifact", "String"),
                port("criteria", "Json"),
                optional("context", "OptionalString"),
            ],
            vec![port("question", "String"), port("system_prompt", "String")],
            ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
        ))
        .expect("prepare_prompt node");

    // ========================================================================
    // LLM Interaction
    // ========================================================================

    // Resolve auth requirements (pure)
    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                    list("required_scopes", "String"),
                    port("interactive_allowed", "Bool"),
                ],
                ReviewGraphOp::Review(ReviewOps::ResolveAuthContract),
            ),
            &prepare_prompt,
        )
        .expect("resolve_auth node");

    // Cloud credential acquisition (resolves provider credentials)
    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config);
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth);

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
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&cloud_credential),
    )
    .expect("llm triplet");
    builder
        .add_edge(
            scope_preflight.out("scope_verified"),
            llm_triplet.in_port("scope_verified"),
        )
        .expect("scope_preflight.scope_verified -> execute_llm.scope_verified");

    // ========================================================================
    // Review Response Parsing
    // ========================================================================

    // ParseReviewResponse - converts answer to ReviewOutput
    let parse_response = builder
        .add_node_after(
            Node::opaque(
                "parse_response",
                vec![port("answer", "String"), port("criteria", "Json")],
                vec![port("output", "Json"), port("errors", "Json")],
                ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
            ),
            &llm_triplet,
        )
        .expect("parse_response node");

    // ========================================================================
    // Edges
    // ========================================================================

    // Blob acquisition flow
    builder
        .add_edge(prepare_blob.out("request"), execute_blob.in_port("request"))
        .expect("prepare_blob.request -> execute_blob.request");
    builder
        .add_edge(prepare_blob.out("skip"), execute_blob.in_port("skip"))
        .expect("prepare_blob.skip -> execute_blob.skip");
    builder
        .add_edge(fs_env.out(FsEnv::WRITE_PORT), execute_blob.in_port("res:file"))
        .expect("fs_env -> execute_blob.res:file");
    builder
        .add_edge(execute_blob.out("response"), parse_blob.in_port("response"))
        .expect("execute_blob.response -> parse_blob.response");
    builder
        .add_edge(prepare_blob.out("source"), parse_blob.in_port("source"))
        .expect("prepare_blob.source -> parse_blob.source");
    builder
        .add_edge(prepare_blob.out("handle"), parse_blob.in_port("handle"))
        .expect("prepare_blob.handle -> parse_blob.handle");
    builder
        .add_edge(prepare_blob.out("skip"), parse_blob.in_port("skip"))
        .expect("prepare_blob.skip -> parse_blob.skip");

    // LLM flow
    builder
        .add_edge(
            prepare_prompt.out("question"),
            llm_triplet.in_port("question"),
        )
        .expect("prepare_prompt.question -> llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            llm_triplet.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> llm.system_prompt");
    builder
        .add_edge(
            cloud_credential.out("credential"),
            llm_triplet.in_port("res:credential"),
        )
        .expect("cloud_credential -> llm.res:credential");

    // Response parsing
    builder
        .add_edge(
            llm_triplet.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("llm.answer -> parse_response.answer");
    // criteria is an entrypoint, flows to both prepare_prompt and parse_response

    builder.build()
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
pub fn build_inline_review_graph() -> Dag<ReviewGraphOp> {
    build_inline_review_graph_with_config(graph_cloud_config())
}

/// Build an inline review graph with explicit cloud config.
pub fn build_inline_review_graph_with_config(
    cloud_config: CloudSecretConfig,
) -> Dag<ReviewGraphOp> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    // Node 0: Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config);

    // Node 1: PrepareReviewPrompt
    let prepare_prompt = builder
        .add_root_node(Node::opaque(
            "prepare_prompt",
            vec![
                port("artifact", "String"),
                port("criteria", "Json"),
                optional("context", "OptionalString"),
            ],
            vec![port("question", "String"), port("system_prompt", "String")],
            ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
        ))
        .expect("prepare_prompt node");

    // Resolve auth requirements (pure)
    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                    list("required_scopes", "String"),
                    port("interactive_allowed", "Bool"),
                ],
                ReviewGraphOp::Review(ReviewOps::ResolveAuthContract),
            ),
            &prepare_prompt,
        )
        .expect("resolve_auth node");

    // Cloud credential acquisition (resolves provider credentials)
    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config);
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth);

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
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&cloud_credential),
    )
    .expect("llm triplet");
    builder
        .add_edge(
            scope_preflight.out("scope_verified"),
            llm_triplet.in_port("scope_verified"),
        )
        .expect("scope_preflight.scope_verified -> execute_llm.scope_verified");

    // ParseReviewResponse
    let parse_response = builder
        .add_node_after(
            Node::opaque(
                "parse_response",
                vec![port("answer", "String"), port("criteria", "Json")],
                vec![port("output", "Json"), port("errors", "Json")],
                ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
            ),
            &llm_triplet,
        )
        .expect("parse_response node");

    // Edges
    builder
        .add_edge(
            prepare_prompt.out("question"),
            llm_triplet.in_port("question"),
        )
        .expect("prepare_prompt.question -> llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            llm_triplet.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> llm.system_prompt");
    builder
        .add_edge(
            cloud_credential.out("credential"),
            llm_triplet.in_port("res:credential"),
        )
        .expect("cloud_credential -> llm.res:credential");
    builder
        .add_edge(
            llm_triplet.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("llm.answer -> parse_response.answer");

    builder.build()
}

// ============================================================================
// DiffReviewPhase DAG Builder (Track 5.2)
// ============================================================================

/// Build a DiffReviewPhase DAG with default pipeline config.
///
/// Uses `ReviewPipelineConfig::gunbc_default()` for provider, model, and criteria.
/// See [`build_diff_review_graph_with`] for full documentation.
pub fn build_diff_review_graph() -> Dag<ReviewGraphOp> {
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
pub fn build_diff_review_graph_with(config: ReviewPipelineConfig) -> Dag<ReviewGraphOp> {
    build_diff_review_graph_with_cloud_config(config, graph_cloud_config())
}

/// Build a DiffReviewPhase DAG with explicit cloud config.
pub fn build_diff_review_graph_with_cloud_config(
    config: ReviewPipelineConfig,
    cloud_config: CloudSecretConfig,
) -> Dag<ReviewGraphOp> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    let fs_env = builder
        .add_root_node(Node::opaque(
            "fs_env",
            vec![],
            vec![port(FsEnv::WRITE_PORT, "FilesystemHandle")],
            ReviewGraphOp::FsEnv(FsEnv::new(filename::Scope::Write)),
        ))
        .expect("fs_env node");

    // Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config);

    let default_branch = config.default_branch.clone();

    // ========================================================================
    // Pipeline Config (zero-input node, emits constants)
    // ========================================================================

    let config_node = builder
        .add_root_node(Node::opaque(
            "config",
            vec![],
            vec![
                port("provider", "String"),
                port("model", "String"),
                port("criteria", "Json"),
            ],
            ReviewGraphOp::Review(ReviewOps::LoadPipelineConfig(config)),
        ))
        .expect("config node");

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
            port("repo_path", "String"),
        ],
        vec![resource("file", "FilesystemHandle", AccessMode::Read)],
        vec![],
        vec![port("diff_files", "Map"), port("stats", "String")],
        ReviewGraphOp::Git(GitOps::PrepareDiff {
            base_ref: default_branch,
            extensions: vec![],
        }),
        ReviewGraphOp::Git(GitOps::ParseDiff),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&fs_env),
    )
    .expect("diff triplet");

    // ========================================================================
    // Diff → Artifact Formatting
    // ========================================================================

    let format_artifact = builder
        .add_node_after(
            Node::opaque(
                "format_artifact",
                vec![port("diff_files", "Map")],
                vec![port("artifact", "String")],
                ReviewGraphOp::Review(ReviewOps::FormatDiffArtifact),
            ),
            &diff_triplet,
        )
        .expect("format_artifact node");

    // ========================================================================
    // Review Prompt Building
    // ========================================================================

    let prepare_prompt = builder
        .add_node_after(
            Node::opaque(
                "prepare_prompt",
                vec![
                    port("artifact", "String"),
                    port("criteria", "Json"),
                    optional("context", "OptionalString"),
                ],
                vec![port("question", "String"), port("system_prompt", "String")],
                ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
            ),
            &format_artifact,
        )
        .expect("prepare_prompt node");

    // ========================================================================
    // LLM Interaction
    // ========================================================================

    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                    list("required_scopes", "String"),
                    port("interactive_allowed", "Bool"),
                ],
                ReviewGraphOp::Review(ReviewOps::ResolveAuthContract),
            ),
            &prepare_prompt,
        )
        .expect("resolve_auth node");

    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config);
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth);

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
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&cloud_credential),
    )
    .expect("llm triplet");
    builder
        .add_edge(
            scope_preflight.out("scope_verified"),
            llm_triplet.in_port("scope_verified"),
        )
        .expect("scope_preflight.scope_verified -> execute_llm.scope_verified");

    // ========================================================================
    // Review Response Parsing
    // ========================================================================

    let parse_response = builder
        .add_node_after(
            Node::opaque(
                "parse_response",
                vec![port("answer", "String"), port("criteria", "Json")],
                vec![port("output", "Json"), port("errors", "Json")],
                ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
            ),
            &llm_triplet,
        )
        .expect("parse_response node");

    // ========================================================================
    // Edges
    // ========================================================================

    // Config → downstream consumers
    builder
        .add_edge(config_node.out("provider"), llm_triplet.in_port("provider"))
        .expect("config.provider -> llm.provider");
    builder
        .add_edge(config_node.out("model"), llm_triplet.in_port("model"))
        .expect("config.model -> llm.model");
    builder
        .add_edge(
            config_node.out("criteria"),
            prepare_prompt.in_port("criteria"),
        )
        .expect("config.criteria -> prepare_prompt.criteria");
    builder
        .add_edge(
            config_node.out("criteria"),
            parse_response.in_port("criteria"),
        )
        .expect("config.criteria -> parse_response.criteria");
    builder
        .add_edge(
            config_node.out("provider"),
            resolve_auth.in_port("provider"),
        )
        .expect("config.provider -> resolve_auth.provider");

    // Diff → artifact formatting
    builder
        .add_edge(
            diff_triplet.out("diff_files"),
            format_artifact.in_port("diff_files"),
        )
        .expect("diff.diff_files -> format_artifact.diff_files");
    builder
        .add_edge(
            fs_env.out(FsEnv::WRITE_PORT),
            diff_triplet.in_port("res:file"),
        )
        .expect("fs_env -> diff.res:file");

    // Artifact → review prompt + LLM content
    builder
        .add_edge(
            format_artifact.out("artifact"),
            prepare_prompt.in_port("artifact"),
        )
        .expect("format_artifact.artifact -> prepare_prompt.artifact");
    builder
        .add_edge(
            format_artifact.out("artifact"),
            llm_triplet.in_port("content"),
        )
        .expect("format_artifact.artifact -> llm.content");

    // LLM flow
    builder
        .add_edge(
            prepare_prompt.out("question"),
            llm_triplet.in_port("question"),
        )
        .expect("prepare_prompt.question -> llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            llm_triplet.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> llm.system_prompt");
    builder
        .add_edge(
            cloud_credential.out("credential"),
            llm_triplet.in_port("res:credential"),
        )
        .expect("cloud_credential -> llm.res:credential");

    // Response parsing
    builder
        .add_edge(
            llm_triplet.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("llm.answer -> parse_response.answer");

    builder.build()
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
    builder = "build_multi_source_review_graph()"
)]
pub fn build_multi_source_review_graph() -> Dag<ReviewGraphOp> {
    build_multi_source_review_graph_with(ReviewPipelineConfig::gunbc_default())
}

/// Build a MultiSourceReviewPhase DAG with explicit pipeline config.
pub fn build_multi_source_review_graph_with(config: ReviewPipelineConfig) -> Dag<ReviewGraphOp> {
    build_multi_source_review_graph_with_cloud_config(config, graph_cloud_config())
}

/// Build a MultiSourceReviewPhase DAG with explicit pipeline and cloud configs.
pub fn build_multi_source_review_graph_with_cloud_config(
    config: ReviewPipelineConfig,
    cloud_config: CloudSecretConfig,
) -> Dag<ReviewGraphOp> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    // Cloud environment (config + OIDC request inputs)
    let cloud_env = add_cloud_env_node(&mut builder, &cloud_config);

    // ========================================================================
    // Pipeline Config
    // ========================================================================

    let config_node = builder
        .add_root_node(Node::opaque(
            "config",
            vec![],
            vec![
                port("provider", "String"),
                port("model", "String"),
                port("criteria", "Json"),
            ],
            ReviewGraphOp::Review(ReviewOps::LoadPipelineConfig(config)),
        ))
        .expect("config node");

    // ========================================================================
    // LLM Review Source (source 1)
    // ========================================================================

    let prepare_prompt = builder
        .add_node_after(
            Node::opaque(
                "prepare_prompt",
                vec![
                    port("artifact", "String"),
                    port("criteria", "Json"),
                    optional("context", "OptionalString"),
                ],
                vec![port("question", "String"), port("system_prompt", "String")],
                ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
            ),
            &config_node,
        )
        .expect("prepare_prompt node");

    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                    list("required_scopes", "String"),
                    port("interactive_allowed", "Bool"),
                ],
                ReviewGraphOp::Review(ReviewOps::ResolveAuthContract),
            ),
            &prepare_prompt,
        )
        .expect("resolve_auth node");

    let cloud_credential =
        add_cloud_credential_chain(&mut builder, &cloud_env, &resolve_auth, &cloud_config);
    let scope_preflight = add_scope_preflight_chain(&mut builder, &resolve_auth);

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
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&cloud_credential),
    )
    .expect("llm triplet");
    builder
        .add_edge(
            scope_preflight.out("scope_verified"),
            llm_triplet.in_port("scope_verified"),
        )
        .expect("scope_preflight.scope_verified -> execute_llm.scope_verified");

    let parse_response = builder
        .add_node_after(
            Node::opaque(
                "parse_response",
                vec![port("answer", "String"), port("criteria", "Json")],
                vec![port("output", "Json"), port("errors", "Json")],
                ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
            ),
            &llm_triplet,
        )
        .expect("parse_response node");

    // ========================================================================
    // Merge (combines sources)
    // ========================================================================

    let merge = builder
        .add_node_after(
            Node::opaque(
                "merge",
                vec![list("outputs", "JsonList")],
                vec![port("bundle", "Json"), port("conflicts", "Json")],
                ReviewGraphOp::Review(ReviewOps::MergeOutputs),
            ),
            &parse_response,
        )
        .expect("merge node");

    // ========================================================================
    // Edges
    // ========================================================================

    // Config → downstream consumers
    builder
        .add_edge(config_node.out("provider"), llm_triplet.in_port("provider"))
        .expect("config.provider -> llm.provider");
    builder
        .add_edge(config_node.out("model"), llm_triplet.in_port("model"))
        .expect("config.model -> llm.model");
    builder
        .add_edge(
            config_node.out("criteria"),
            prepare_prompt.in_port("criteria"),
        )
        .expect("config.criteria -> prepare_prompt.criteria");
    builder
        .add_edge(
            config_node.out("criteria"),
            parse_response.in_port("criteria"),
        )
        .expect("config.criteria -> parse_response.criteria");
    builder
        .add_edge(
            config_node.out("provider"),
            resolve_auth.in_port("provider"),
        )
        .expect("config.provider -> resolve_auth.provider");

    // LLM review flow
    builder
        .add_edge(
            prepare_prompt.out("question"),
            llm_triplet.in_port("question"),
        )
        .expect("prepare_prompt.question -> llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            llm_triplet.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> llm.system_prompt");
    builder
        .add_edge(
            cloud_credential.out("credential"),
            llm_triplet.in_port("res:credential"),
        )
        .expect("cloud_credential -> llm.res:credential");
    builder
        .add_edge(
            llm_triplet.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("llm.answer -> parse_response.answer");

    // Review output → merge (list port collects fan-in automatically)
    builder
        .add_edge(parse_response.out("output"), merge.in_port("outputs"))
        .expect("parse_response.output -> merge.outputs");

    builder.build()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_review_phase_graph_boundaries() {
        let dag = build_review_phase_graph();
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
        let dag = build_review_phase_graph();
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
        let dag = build_inline_review_graph();
        let boundaries = detect_boundaries(&dag);

        assert!(
            boundaries.is_boundary_node(&"parse_response".into()),
            "parse_response should be a boundary"
        );
    }

    #[test]
    fn test_inline_review_graph_entrypoints() {
        let dag = build_inline_review_graph();
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
            ReviewGraphOp::Blob(BlobOps::PrepareFetch),
            ReviewGraphOp::Git(GitOps::ParseDiff),
            ReviewGraphOp::Review(ReviewOps::HashFinding),
            ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
            ReviewGraphOp::Llm(LlmOps::ResolveAuth),
            ReviewGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::ResolveConfig)),
            ReviewGraphOp::Transport(TransportOps::Execute),
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
        let dag = build_diff_review_graph();
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
        let dag = build_diff_review_graph();
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
        let dag = build_diff_review_graph();
        for node_id in ["diff", "llm"] {
            let node = dag
                .get_node(&node_id.into())
                .unwrap_or_else(|| panic!("missing subdag node: {}", node_id));
            assert!(
                node.is_subdag(),
                "{} should be a subdag node",
                node_id
            );
        }
    }

    // ========================================================================
    // MultiSourceReviewPhase tests
    // ========================================================================

    #[test]
    fn test_multi_source_review_graph_boundaries() {
        let dag = build_multi_source_review_graph();
        let boundaries = detect_boundaries(&dag);

        assert!(
            boundaries.is_boundary_node(&"merge".into()),
            "merge should be a boundary (bundle + conflicts outputs)"
        );
    }

    #[test]
    fn test_multi_source_review_graph_entrypoints() {
        let dag = build_multi_source_review_graph();
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
}
