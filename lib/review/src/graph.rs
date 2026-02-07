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
use gunbc_ir::{
    add_transport_execute_parse_named_with_passthrough,
    add_transport_triplet_named_with_passthrough, build::*, Dag, DagBuilder, Node, Value,
};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_git_ops::GitOps;
use gunbc_lib_llm_ops::LlmOps;
use gunbc_lib_transport::{CredentialOp, TransportOps};
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
    /// Credential environment (BOUNDARY - resolves provider credentials)
    Cred(CredentialOp),
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
            ReviewGraphOp::Cred(op) => op.execute(inputs),
            ReviewGraphOp::Transport(op) => op.execute(inputs),
        }
    }
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
///                              prepare_llm → resolve_auth → credential_env → [execute_llm] → parse_llm
///                                                                      ↓
///                                                               parse_response
/// ```
///
/// Note: For inline blob sources, execute_blob is skipped (handled by prepare_blob).
/// The graph handles this with conditional execution.
pub fn build_review_phase_graph() -> Dag<ReviewGraphOp> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

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
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
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
                    optional("handle", "Json"),
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
                optional("context", "String"),
            ],
            vec![port("question", "String"), port("system_prompt", "String")],
            ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
        ))
        .expect("prepare_prompt node");

    // ========================================================================
    // LLM Interaction
    // ========================================================================

    // Node 5: PrepareSimpleRequest - builds LLM request
    let prepare_llm = builder
        .add_node_after(
            Node::opaque(
                "prepare_llm",
                vec![
                    port("content", "String"),
                    port("question", "String"),
                    port("provider", "String"),
                    port("model", "String"),
                    optional("system_prompt", "String"),
                ],
                vec![
                    port("request", "TransportRequest"),
                    port("provider", "String"),
                    port("skip", "Bool"),
                ],
                ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
            ),
            &prepare_prompt,
        )
        .expect("prepare_llm node");

    // Node 6: Resolve auth requirements (pure)
    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                ReviewGraphOp::Llm(LlmOps::ResolveAuth),
            ),
            &prepare_llm,
        )
        .expect("resolve_auth node");

    // Node 7: Credential environment (resolves provider credentials)
    let cred_port = "credential:llm";
    let credential_env = builder
        .add_node_after(
            Node::opaque(
                "credential_env",
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                vec![port(cred_port, "Credential")],
                ReviewGraphOp::Cred(CredentialOp::from_inputs(cred_port)),
            ),
            &resolve_auth,
        )
        .expect("credential_env node");

    // Nodes 8-9: Execute LLM + ParseSimpleResponse
    let llm_triplet = add_transport_execute_parse_named_with_passthrough(
        &mut builder,
        &prepare_llm,
        "execute_llm",
        "parse_llm",
        vec![port("provider", "String")],
        vec![resource("credential", "Credential", AccessMode::Read)],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&credential_env),
    )
    .expect("llm triplet");

    // ========================================================================
    // Review Response Parsing
    // ========================================================================

    // Node 10: ParseReviewResponse - converts answer to ReviewOutput
    let parse_response = builder
        .add_node_after(
            Node::opaque(
                "parse_response",
                vec![port("answer", "String"), port("criteria", "Json")],
                vec![port("output", "Json"), port("errors", "Json")],
                ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
            ),
            &llm_triplet.parse,
        )
        .expect("parse_response node");

    // ========================================================================
    // Edges
    // ========================================================================

    // Blob acquisition flow
    builder
        .add_edge(
            prepare_blob.out("request"),
            execute_blob.in_port("request"),
        )
        .expect("prepare_blob.request -> execute_blob.request");
    builder
        .add_edge(prepare_blob.out("skip"), execute_blob.in_port("skip"))
        .expect("prepare_blob.skip -> execute_blob.skip");
    builder
        .add_edge(
            execute_blob.out("response"),
            parse_blob.in_port("response"),
        )
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
            prepare_llm.in_port("question"),
        )
        .expect("prepare_prompt.question -> prepare_llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            prepare_llm.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> prepare_llm.system_prompt");
    builder
        .add_edge(
            prepare_llm.out("provider"),
            resolve_auth.in_port("provider"),
        )
        .expect("prepare_llm.provider -> resolve_auth.provider");
    builder
        .add_edge(
            resolve_auth.out("service"),
            credential_env.in_port("service"),
        )
        .expect("resolve_auth.service -> credential_env.service");
    builder
        .add_edge(
            resolve_auth.out("env_var"),
            credential_env.in_port("env_var"),
        )
        .expect("resolve_auth.env_var -> credential_env.env_var");
    builder
        .add_edge(
            resolve_auth.out("scheme"),
            credential_env.in_port("scheme"),
        )
        .expect("resolve_auth.scheme -> credential_env.scheme");
    builder
        .add_edge(
            resolve_auth.out("header_name"),
            credential_env.in_port("header_name"),
        )
        .expect("resolve_auth.header_name -> credential_env.header_name");
    builder
        .add_edge(
            credential_env.out(cred_port),
            llm_triplet.execute.in_port("res:credential"),
        )
        .expect("credential_env -> execute_llm.res:credential");

    // Response parsing
    builder
        .add_edge(
            llm_triplet.parse.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("parse_llm.answer -> parse_response.answer");
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
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

    // Node 1: PrepareReviewPrompt
    let prepare_prompt = builder
        .add_root_node(Node::opaque(
            "prepare_prompt",
            vec![
                port("artifact", "String"),
                port("criteria", "Json"),
                optional("context", "String"),
            ],
            vec![port("question", "String"), port("system_prompt", "String")],
            ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
        ))
        .expect("prepare_prompt node");

    // Node 2: PrepareSimpleRequest
    let prepare_llm = builder
        .add_node_after(
            Node::opaque(
                "prepare_llm",
                vec![
                    port("content", "String"),
                    port("question", "String"),
                    port("provider", "String"),
                    port("model", "String"),
                    optional("system_prompt", "String"),
                ],
                vec![
                    port("request", "TransportRequest"),
                    port("provider", "String"),
                    port("skip", "Bool"),
                ],
                ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
            ),
            &prepare_prompt,
        )
        .expect("prepare_llm node");

    // Node 3: Resolve auth requirements (pure)
    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                ReviewGraphOp::Llm(LlmOps::ResolveAuth),
            ),
            &prepare_llm,
        )
        .expect("resolve_auth node");

    // Node 4: Credential environment (resolves provider credentials)
    let cred_port = "credential:llm";
    let credential_env = builder
        .add_node_after(
            Node::opaque(
                "credential_env",
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                vec![port(cred_port, "Credential")],
                ReviewGraphOp::Cred(CredentialOp::from_inputs(cred_port)),
            ),
            &resolve_auth,
        )
        .expect("credential_env node");

    // Nodes 5-6: Execute LLM + ParseSimpleResponse
    let llm_triplet = add_transport_execute_parse_named_with_passthrough(
        &mut builder,
        &prepare_llm,
        "execute_llm",
        "parse_llm",
        vec![port("provider", "String")],
        vec![resource("credential", "Credential", AccessMode::Read)],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&credential_env),
    )
    .expect("llm triplet");

    // Node 7: ParseReviewResponse
    let parse_response = builder
        .add_node_after(
            Node::opaque(
                "parse_response",
                vec![port("answer", "String"), port("criteria", "Json")],
                vec![port("output", "Json"), port("errors", "Json")],
                ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
            ),
            &llm_triplet.parse,
        )
        .expect("parse_response node");

    // Edges
    builder
        .add_edge(
            prepare_prompt.out("question"),
            prepare_llm.in_port("question"),
        )
        .expect("prepare_prompt.question -> prepare_llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            prepare_llm.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> prepare_llm.system_prompt");
    builder
        .add_edge(
            prepare_llm.out("provider"),
            resolve_auth.in_port("provider"),
        )
        .expect("prepare_llm.provider -> resolve_auth.provider");
    builder
        .add_edge(
            resolve_auth.out("service"),
            credential_env.in_port("service"),
        )
        .expect("resolve_auth.service -> credential_env.service");
    builder
        .add_edge(
            resolve_auth.out("env_var"),
            credential_env.in_port("env_var"),
        )
        .expect("resolve_auth.env_var -> credential_env.env_var");
    builder
        .add_edge(
            resolve_auth.out("scheme"),
            credential_env.in_port("scheme"),
        )
        .expect("resolve_auth.scheme -> credential_env.scheme");
    builder
        .add_edge(
            resolve_auth.out("header_name"),
            credential_env.in_port("header_name"),
        )
        .expect("resolve_auth.header_name -> credential_env.header_name");
    builder
        .add_edge(
            credential_env.out(cred_port),
            llm_triplet.execute.in_port("res:credential"),
        )
        .expect("credential_env -> execute_llm.res:credential");
    builder
        .add_edge(
            llm_triplet.parse.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("parse_llm.answer -> parse_response.answer");

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
///                                             prepare_llm → resolve_auth → credential_env → [execute_llm] → parse_llm
///                                                                                     ↓
///                                                                              parse_response
/// ```
///
/// I/O Classification:
/// - Two TransportOps::Execute calls: git diff (read), LLM (read)
/// - Phase overall: Read-only
pub fn build_diff_review_graph_with(config: ReviewPipelineConfig) -> Dag<ReviewGraphOp> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

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
        "prepare_diff",
        "execute_diff",
        "parse_diff",
        vec![
            optional("base_ref", "String"),
            port("repo_path", "String"),
        ],
        vec![],
        vec![port("diff_files", "Map"), port("stats", "String")],
        ReviewGraphOp::Git(GitOps::PrepareDiff {
            base_ref: default_branch,
            extensions: vec![],
        }),
        ReviewGraphOp::Git(GitOps::ParseDiff),
        ReviewGraphOp::Transport(TransportOps::Execute),
        None,
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
            &diff_triplet.parse,
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
                    optional("context", "String"),
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

    let prepare_llm = builder
        .add_node_after(
            Node::opaque(
                "prepare_llm",
                vec![
                    port("content", "String"),
                    port("question", "String"),
                    port("provider", "String"),
                    port("model", "String"),
                    optional("system_prompt", "String"),
                ],
                vec![
                    port("request", "TransportRequest"),
                    port("provider", "String"),
                    port("skip", "Bool"),
                ],
                ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
            ),
            &prepare_prompt,
        )
        .expect("prepare_llm node");

    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                ReviewGraphOp::Llm(LlmOps::ResolveAuth),
            ),
            &prepare_llm,
        )
        .expect("resolve_auth node");

    let cred_port = "credential:llm";
    let credential_env = builder
        .add_node_after(
            Node::opaque(
                "credential_env",
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                vec![port(cred_port, "Credential")],
                ReviewGraphOp::Cred(CredentialOp::from_inputs(cred_port)),
            ),
            &resolve_auth,
        )
        .expect("credential_env node");

    let llm_triplet = add_transport_execute_parse_named_with_passthrough(
        &mut builder,
        &prepare_llm,
        "execute_llm",
        "parse_llm",
        vec![port("provider", "String")],
        vec![resource("credential", "Credential", AccessMode::Read)],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&credential_env),
    )
    .expect("llm triplet");

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
            &llm_triplet.parse,
        )
        .expect("parse_response node");

    // ========================================================================
    // Edges
    // ========================================================================

    // Config → downstream consumers
    builder
        .add_edge(config_node.out("provider"), prepare_llm.in_port("provider"))
        .expect("config.provider -> prepare_llm.provider");
    builder
        .add_edge(config_node.out("model"), prepare_llm.in_port("model"))
        .expect("config.model -> prepare_llm.model");
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
        .add_edge(config_node.out("provider"), resolve_auth.in_port("provider"))
        .expect("config.provider -> resolve_auth.provider");

    // Diff → artifact formatting
    builder
        .add_edge(
            diff_triplet.parse.out("diff_files"),
            format_artifact.in_port("diff_files"),
        )
        .expect("parse_diff.diff_files -> format_artifact.diff_files");

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
            prepare_llm.in_port("content"),
        )
        .expect("format_artifact.artifact -> prepare_llm.content");

    // LLM flow
    builder
        .add_edge(
            prepare_prompt.out("question"),
            prepare_llm.in_port("question"),
        )
        .expect("prepare_prompt.question -> prepare_llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            prepare_llm.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> prepare_llm.system_prompt");
    builder
        .add_edge(
            resolve_auth.out("service"),
            credential_env.in_port("service"),
        )
        .expect("resolve_auth.service -> credential_env.service");
    builder
        .add_edge(
            resolve_auth.out("env_var"),
            credential_env.in_port("env_var"),
        )
        .expect("resolve_auth.env_var -> credential_env.env_var");
    builder
        .add_edge(
            resolve_auth.out("scheme"),
            credential_env.in_port("scheme"),
        )
        .expect("resolve_auth.scheme -> credential_env.scheme");
    builder
        .add_edge(
            resolve_auth.out("header_name"),
            credential_env.in_port("header_name"),
        )
        .expect("resolve_auth.header_name -> credential_env.header_name");
    builder
        .add_edge(
            credential_env.out(cred_port),
            llm_triplet.execute.in_port("res:credential"),
        )
        .expect("credential_env -> execute_llm.res:credential");

    // Response parsing
    builder
        .add_edge(
            llm_triplet.parse.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("parse_llm.answer -> parse_response.answer");

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
pub fn build_multi_source_review_graph() -> Dag<ReviewGraphOp> {
    build_multi_source_review_graph_with(ReviewPipelineConfig::gunbc_default())
}

/// Build a MultiSourceReviewPhase DAG with explicit pipeline config.
pub fn build_multi_source_review_graph_with(config: ReviewPipelineConfig) -> Dag<ReviewGraphOp> {
    let mut builder: DagBuilder<ReviewGraphOp> = DagBuilder::new();

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
                    optional("context", "String"),
                ],
                vec![port("question", "String"), port("system_prompt", "String")],
                ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
            ),
            &config_node,
        )
        .expect("prepare_prompt node");

    let prepare_llm = builder
        .add_node_after(
            Node::opaque(
                "prepare_llm",
                vec![
                    port("content", "String"),
                    port("question", "String"),
                    port("provider", "String"),
                    port("model", "String"),
                    optional("system_prompt", "String"),
                ],
                vec![
                    port("request", "TransportRequest"),
                    port("provider", "String"),
                    port("skip", "Bool"),
                ],
                ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
            ),
            &prepare_prompt,
        )
        .expect("prepare_llm node");

    let resolve_auth = builder
        .add_node_after(
            Node::opaque(
                "resolve_auth",
                vec![port("provider", "String")],
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                ReviewGraphOp::Llm(LlmOps::ResolveAuth),
            ),
            &prepare_llm,
        )
        .expect("resolve_auth node");

    let cred_port = "credential:llm";
    let credential_env = builder
        .add_node_after(
            Node::opaque(
                "credential_env",
                vec![
                    port("service", "String"),
                    port("env_var", "String"),
                    port("scheme", "String"),
                    port("header_name", "String"),
                ],
                vec![port(cred_port, "Credential")],
                ReviewGraphOp::Cred(CredentialOp::from_inputs(cred_port)),
            ),
            &resolve_auth,
        )
        .expect("credential_env node");

    let llm_triplet = add_transport_execute_parse_named_with_passthrough(
        &mut builder,
        &prepare_llm,
        "execute_llm",
        "parse_llm",
        vec![port("provider", "String")],
        vec![resource("credential", "Credential", AccessMode::Read)],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
        ReviewGraphOp::Transport(TransportOps::Execute),
        Some(&credential_env),
    )
    .expect("llm triplet");

    let parse_response = builder
        .add_node_after(
            Node::opaque(
                "parse_response",
                vec![port("answer", "String"), port("criteria", "Json")],
                vec![port("output", "Json"), port("errors", "Json")],
                ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
            ),
            &llm_triplet.parse,
        )
        .expect("parse_response node");

    // ========================================================================
    // Merge (combines sources)
    // ========================================================================

    let merge = builder
        .add_node_after(
            Node::opaque(
                "merge",
                vec![list("outputs", "Json")],
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
        .add_edge(config_node.out("provider"), prepare_llm.in_port("provider"))
        .expect("config.provider -> prepare_llm.provider");
    builder
        .add_edge(config_node.out("model"), prepare_llm.in_port("model"))
        .expect("config.model -> prepare_llm.model");
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
        .add_edge(config_node.out("provider"), resolve_auth.in_port("provider"))
        .expect("config.provider -> resolve_auth.provider");

    // LLM review flow
    builder
        .add_edge(
            prepare_prompt.out("question"),
            prepare_llm.in_port("question"),
        )
        .expect("prepare_prompt.question -> prepare_llm.question");
    builder
        .add_edge(
            prepare_prompt.out("system_prompt"),
            prepare_llm.in_port("system_prompt"),
        )
        .expect("prepare_prompt.system_prompt -> prepare_llm.system_prompt");
    builder
        .add_edge(
            resolve_auth.out("service"),
            credential_env.in_port("service"),
        )
        .expect("resolve_auth.service -> credential_env.service");
    builder
        .add_edge(
            resolve_auth.out("env_var"),
            credential_env.in_port("env_var"),
        )
        .expect("resolve_auth.env_var -> credential_env.env_var");
    builder
        .add_edge(
            resolve_auth.out("scheme"),
            credential_env.in_port("scheme"),
        )
        .expect("resolve_auth.scheme -> credential_env.scheme");
    builder
        .add_edge(
            resolve_auth.out("header_name"),
            credential_env.in_port("header_name"),
        )
        .expect("resolve_auth.header_name -> credential_env.header_name");
    builder
        .add_edge(
            credential_env.out(cred_port),
            llm_triplet.execute.in_port("res:credential"),
        )
        .expect("credential_env -> execute_llm.res:credential");
    builder
        .add_edge(
            llm_triplet.parse.out("answer"),
            parse_response.in_port("answer"),
        )
        .expect("parse_llm.answer -> parse_response.answer");

    // Review output → merge (list port collects fan-in automatically)
    builder
        .add_edge(
            parse_response.out("output"),
            merge.in_port("outputs"),
        )
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

        // prepare_llm has provider/model as entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_llm".into()),
            "prepare_llm should have entrypoints"
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

        // prepare_llm has provider/model/content as entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_llm".into()),
            "prepare_llm should have entrypoints"
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
            ReviewGraphOp::Cred(CredentialOp::from_inputs("credential:llm")),
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

        // stats from parse_diff is also an unconnected output
        assert!(
            boundaries.is_boundary_node(&"parse_diff".into()),
            "parse_diff.stats should be a boundary"
        );
    }

    #[test]
    fn test_diff_review_graph_entrypoints() {
        let dag = build_diff_review_graph();
        let entrypoints = detect_entrypoints(&dag);

        // prepare_diff has base_ref (optional) and repo_path (required) entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_diff".into()),
            "prepare_diff should have entrypoints"
        );

        // provider, model are NOT entrypoints — config node provides them
        assert!(
            !entrypoints.is_entrypoint_node(&"prepare_llm".into()),
            "prepare_llm should NOT have entrypoints (config provides provider/model)"
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
    fn test_diff_review_graph_has_two_transport_boundaries() {
        let dag = build_diff_review_graph();
        let transport_nodes: Vec<_> = dag
            .nodes
            .iter()
            .filter(|n| {
                matches!(
                    n.body,
                    gunbc_ir::NodeBody::Opaque(ReviewGraphOp::Transport(_))
                )
            })
            .collect();
        assert_eq!(
            transport_nodes.len(),
            2,
            "should have execute_diff and execute_llm"
        );
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

        // provider/model/content on prepare_llm — only content is an entrypoint
        assert!(
            entrypoints.is_entrypoint_node(&"prepare_llm".into()),
            "prepare_llm should have content as entrypoint"
        );
    }
}
