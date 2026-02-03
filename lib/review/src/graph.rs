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
use gunbc_ir::{build::*, Dag, Node, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_git_ops::GitOps;
use gunbc_lib_llm_ops::LlmOps;
use gunbc_lib_transport::TransportOps;
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
///                              prepare_llm → [execute_llm] → parse_llm
///                                                                ↓
///                                                         parse_response
/// ```
///
/// Note: For inline blob sources, execute_blob is skipped (handled by prepare_blob).
/// The graph handles this with conditional execution.
pub fn build_review_phase_graph() -> Dag<ReviewGraphOp> {
    let mut dag = Dag::new();

    // ========================================================================
    // Blob Acquisition
    // ========================================================================

    // Node 1: PrepareFetch - builds request or returns inline data
    dag.add_node(Node::opaque(
        "prepare_blob",
        vec![port("source", "Json")],
        vec![
            port("request", "TransportRequest"),
            port("skip_fetch", "Bool"),
            port("handle", "Json"),    // Present if inline
            port("source", "Json"),    // Echo for parse
        ],
        ReviewGraphOp::Blob(BlobOps::PrepareFetch),
    ));

    // Node 2: Execute blob fetch (I/O boundary)
    // Note: This is skipped for inline sources (skip_fetch=true)
    dag.add_node(Node::opaque(
        "execute_blob",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        ReviewGraphOp::Transport(TransportOps::Execute),
    ));

    // Node 3: ParseFetch - converts response to BlobHandle
    dag.add_node(Node::opaque(
        "parse_blob",
        vec![
            port("source", "Json"),
            port("response", "TransportResponse"),
        ],
        vec![
            port("handle", "Json"),
            port("meta", "Json"),
        ],
        ReviewGraphOp::Blob(BlobOps::ParseFetch),
    ));

    // ========================================================================
    // Review Prompt Building
    // ========================================================================

    // Node 4: PrepareReviewPrompt - builds question from blob + criteria
    dag.add_node(Node::opaque(
        "prepare_prompt",
        vec![
            port("artifact", "String"),
            port("criteria", "Json"),
            optional("context", "String"),
        ],
        vec![
            port("question", "String"),
            port("system_prompt", "String"),
        ],
        ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
    ));

    // ========================================================================
    // LLM Interaction
    // ========================================================================

    // Node 5: PrepareSimpleRequest - builds LLM request
    dag.add_node(Node::opaque(
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
        ],
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
    ));

    // Node 6: Execute LLM call (I/O boundary)
    dag.add_node(Node::opaque(
        "execute_llm",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        ReviewGraphOp::Transport(TransportOps::Execute),
    ));

    // Node 7: ParseSimpleResponse - extracts answer
    dag.add_node(Node::opaque(
        "parse_llm",
        vec![
            port("provider", "String"),
            port("response", "TransportResponse"),
        ],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
    ));

    // ========================================================================
    // Review Response Parsing
    // ========================================================================

    // Node 8: ParseReviewResponse - converts answer to ReviewOutput
    dag.add_node(Node::opaque(
        "parse_response",
        vec![
            port("answer", "String"),
            port("criteria", "Json"),
        ],
        vec![
            port("output", "Json"),
            port("errors", "Json"),
        ],
        ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
    ));

    // ========================================================================
    // Edges
    // ========================================================================

    // Blob acquisition flow
    dag.add_edge(edge("prepare_blob", "request", "execute_blob", "request"));
    dag.add_edge(edge("execute_blob", "response", "parse_blob", "response"));
    dag.add_edge(edge("prepare_blob", "source", "parse_blob", "source"));

    // Note: For inline sources, we need a way to bypass execute_blob.
    // This is handled by the skip_fetch flag - the executor should check this.
    // For now, we wire the full path. Conditional execution is a future enhancement.

    // Review prompt building (uses blob data)
    // The artifact comes from parse_blob.handle.data (extracted by the DAG runner)
    // For simplicity, we expect the caller to provide artifact directly for now.
    // A full implementation would use an Extract node to get blob.data.

    // LLM flow
    dag.add_edge(edge("prepare_prompt", "question", "prepare_llm", "question"));
    dag.add_edge(edge("prepare_prompt", "system_prompt", "prepare_llm", "system_prompt"));
    dag.add_edge(edge("prepare_llm", "request", "execute_llm", "request"));
    dag.add_edge(edge("execute_llm", "response", "parse_llm", "response"));
    dag.add_edge(edge("prepare_llm", "provider", "parse_llm", "provider"));

    // Response parsing
    dag.add_edge(edge("parse_llm", "answer", "parse_response", "answer"));
    // criteria is an entrypoint, flows to both prepare_prompt and parse_response

    dag
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
    let mut dag = Dag::new();

    // Node 1: PrepareReviewPrompt
    dag.add_node(Node::opaque(
        "prepare_prompt",
        vec![
            port("artifact", "String"),
            port("criteria", "Json"),
            optional("context", "String"),
        ],
        vec![
            port("question", "String"),
            port("system_prompt", "String"),
        ],
        ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
    ));

    // Node 2: PrepareSimpleRequest
    dag.add_node(Node::opaque(
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
        ],
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
    ));

    // Node 3: Execute LLM (I/O boundary)
    dag.add_node(Node::opaque(
        "execute_llm",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        ReviewGraphOp::Transport(TransportOps::Execute),
    ));

    // Node 4: ParseSimpleResponse
    dag.add_node(Node::opaque(
        "parse_llm",
        vec![
            port("provider", "String"),
            port("response", "TransportResponse"),
        ],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
    ));

    // Node 5: ParseReviewResponse
    dag.add_node(Node::opaque(
        "parse_response",
        vec![
            port("answer", "String"),
            port("criteria", "Json"),
        ],
        vec![
            port("output", "Json"),
            port("errors", "Json"),
        ],
        ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
    ));

    // Edges
    dag.add_edge(edge("prepare_prompt", "question", "prepare_llm", "question"));
    dag.add_edge(edge("prepare_prompt", "system_prompt", "prepare_llm", "system_prompt"));
    dag.add_edge(edge("prepare_llm", "request", "execute_llm", "request"));
    dag.add_edge(edge("execute_llm", "response", "parse_llm", "response"));
    dag.add_edge(edge("prepare_llm", "provider", "parse_llm", "provider"));
    dag.add_edge(edge("parse_llm", "answer", "parse_response", "answer"));

    dag
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
/// - `prepare_diff.repo_path` (optional): String — repo path override
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
///                                             prepare_llm → [execute_llm] → parse_llm
///                                                                               ↓
///                                                                        parse_response
/// ```
///
/// I/O Classification:
/// - Two TransportOps::Execute calls: git diff (read), LLM (read)
/// - Phase overall: Read-only
pub fn build_diff_review_graph_with(
    config: ReviewPipelineConfig,
) -> Dag<ReviewGraphOp> {
    let mut dag = Dag::new();

    let default_branch = config.default_branch.clone();

    // ========================================================================
    // Pipeline Config (zero-input node, emits constants)
    // ========================================================================

    dag.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            port("provider", "String"),
            port("model", "String"),
            port("criteria", "Json"),
        ],
        ReviewGraphOp::Review(ReviewOps::LoadPipelineConfig(config)),
    ));

    // ========================================================================
    // Git Diff Acquisition
    // ========================================================================

    // PrepareDiff - builds git diff request.
    // base_ref default comes from pipeline config (GitConfig::default_branch);
    // the optional port allows CLI override.
    dag.add_node(Node::opaque(
        "prepare_diff",
        vec![
            optional("base_ref", "String"),
            optional("repo_path", "String"),
        ],
        vec![port("request", "TransportRequest")],
        ReviewGraphOp::Git(GitOps::PrepareDiff {
            base_ref: default_branch,
            extensions: vec![],
        }),
    ));

    // Execute git diff (I/O boundary)
    dag.add_node(Node::opaque(
        "execute_diff",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        ReviewGraphOp::Transport(TransportOps::Execute),
    ));

    // ParseDiff - extracts diff content
    dag.add_node(Node::opaque(
        "parse_diff",
        vec![port("response", "TransportResponse")],
        vec![
            port("diff_files", "Map"),
            port("stats", "String"),
        ],
        ReviewGraphOp::Git(GitOps::ParseDiff),
    ));

    // ========================================================================
    // Diff → Artifact Formatting
    // ========================================================================

    dag.add_node(Node::opaque(
        "format_artifact",
        vec![port("diff_files", "Map")],
        vec![port("artifact", "String")],
        ReviewGraphOp::Review(ReviewOps::FormatDiffArtifact),
    ));

    // ========================================================================
    // Review Prompt Building
    // ========================================================================

    dag.add_node(Node::opaque(
        "prepare_prompt",
        vec![
            port("artifact", "String"),
            port("criteria", "Json"),
            optional("context", "String"),
        ],
        vec![
            port("question", "String"),
            port("system_prompt", "String"),
        ],
        ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
    ));

    // ========================================================================
    // LLM Interaction
    // ========================================================================

    dag.add_node(Node::opaque(
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
        ],
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
    ));

    dag.add_node(Node::opaque(
        "execute_llm",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        ReviewGraphOp::Transport(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_llm",
        vec![
            port("provider", "String"),
            port("response", "TransportResponse"),
        ],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
    ));

    // ========================================================================
    // Review Response Parsing
    // ========================================================================

    dag.add_node(Node::opaque(
        "parse_response",
        vec![
            port("answer", "String"),
            port("criteria", "Json"),
        ],
        vec![
            port("output", "Json"),
            port("errors", "Json"),
        ],
        ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
    ));

    // ========================================================================
    // Edges
    // ========================================================================

    // Config → downstream consumers
    dag.add_edge(edge("config", "provider", "prepare_llm", "provider"));
    dag.add_edge(edge("config", "model", "prepare_llm", "model"));
    dag.add_edge(edge("config", "criteria", "prepare_prompt", "criteria"));
    dag.add_edge(edge("config", "criteria", "parse_response", "criteria"));

    // Git diff flow
    dag.add_edge(edge("prepare_diff", "request", "execute_diff", "request"));
    dag.add_edge(edge("execute_diff", "response", "parse_diff", "response"));

    // Diff → artifact formatting
    dag.add_edge(edge("parse_diff", "diff_files", "format_artifact", "diff_files"));

    // Artifact → review prompt + LLM content
    dag.add_edge(edge("format_artifact", "artifact", "prepare_prompt", "artifact"));
    dag.add_edge(edge("format_artifact", "artifact", "prepare_llm", "content"));

    // LLM flow
    dag.add_edge(edge("prepare_prompt", "question", "prepare_llm", "question"));
    dag.add_edge(edge("prepare_prompt", "system_prompt", "prepare_llm", "system_prompt"));
    dag.add_edge(edge("prepare_llm", "request", "execute_llm", "request"));
    dag.add_edge(edge("execute_llm", "response", "parse_llm", "response"));
    dag.add_edge(edge("prepare_llm", "provider", "parse_llm", "provider"));

    // Response parsing
    dag.add_edge(edge("parse_llm", "answer", "parse_response", "answer"));

    dag
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
/// MergeOutputs accepts both a single ReviewOutput object and an array,
/// so each source can wire directly without wrapper nodes.
pub fn build_multi_source_review_graph() -> Dag<ReviewGraphOp> {
    build_multi_source_review_graph_with(ReviewPipelineConfig::gunbc_default())
}

/// Build a MultiSourceReviewPhase DAG with explicit pipeline config.
pub fn build_multi_source_review_graph_with(
    config: ReviewPipelineConfig,
) -> Dag<ReviewGraphOp> {
    let mut dag = Dag::new();

    // ========================================================================
    // Pipeline Config
    // ========================================================================

    dag.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            port("provider", "String"),
            port("model", "String"),
            port("criteria", "Json"),
        ],
        ReviewGraphOp::Review(ReviewOps::LoadPipelineConfig(config)),
    ));

    // ========================================================================
    // LLM Review Source (source 1)
    // ========================================================================

    dag.add_node(Node::opaque(
        "prepare_prompt",
        vec![
            port("artifact", "String"),
            port("criteria", "Json"),
            optional("context", "String"),
        ],
        vec![
            port("question", "String"),
            port("system_prompt", "String"),
        ],
        ReviewGraphOp::Review(ReviewOps::PrepareReviewPrompt),
    ));

    dag.add_node(Node::opaque(
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
        ],
        ReviewGraphOp::Llm(LlmOps::PrepareSimpleRequest),
    ));

    dag.add_node(Node::opaque(
        "execute_llm",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        ReviewGraphOp::Transport(TransportOps::Execute),
    ));

    dag.add_node(Node::opaque(
        "parse_llm",
        vec![
            port("provider", "String"),
            port("response", "TransportResponse"),
        ],
        vec![port("answer", "String")],
        ReviewGraphOp::Llm(LlmOps::ParseSimpleResponse),
    ));

    dag.add_node(Node::opaque(
        "parse_response",
        vec![
            port("answer", "String"),
            port("criteria", "Json"),
        ],
        vec![
            port("output", "Json"),
            port("errors", "Json"),
        ],
        ReviewGraphOp::Review(ReviewOps::ParseReviewResponse),
    ));

    // ========================================================================
    // Merge (combines sources)
    // ========================================================================

    dag.add_node(Node::opaque(
        "merge",
        vec![port("outputs", "Json")],
        vec![
            port("bundle", "Json"),
            port("conflicts", "Json"),
        ],
        ReviewGraphOp::Review(ReviewOps::MergeOutputs),
    ));

    // ========================================================================
    // Edges
    // ========================================================================

    // Config → downstream consumers
    dag.add_edge(edge("config", "provider", "prepare_llm", "provider"));
    dag.add_edge(edge("config", "model", "prepare_llm", "model"));
    dag.add_edge(edge("config", "criteria", "prepare_prompt", "criteria"));
    dag.add_edge(edge("config", "criteria", "parse_response", "criteria"));

    // LLM review flow
    dag.add_edge(edge("prepare_prompt", "question", "prepare_llm", "question"));
    dag.add_edge(edge("prepare_prompt", "system_prompt", "prepare_llm", "system_prompt"));
    dag.add_edge(edge("prepare_llm", "request", "execute_llm", "request"));
    dag.add_edge(edge("execute_llm", "response", "parse_llm", "response"));
    dag.add_edge(edge("prepare_llm", "provider", "parse_llm", "provider"));
    dag.add_edge(edge("parse_llm", "answer", "parse_response", "answer"));

    // Review output → merge (MergeOutputs accepts single object or array)
    dag.add_edge(edge("parse_response", "output", "merge", "outputs"));

    dag
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_review_phase_graph_structure() {
        let dag = build_review_phase_graph();
        assert_eq!(dag.nodes.len(), 8);
    }

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
    fn test_inline_review_graph_structure() {
        let dag = build_inline_review_graph();
        assert_eq!(dag.nodes.len(), 5);
        // No blob nodes - content provided directly
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
    fn test_diff_review_graph_structure() {
        let dag = build_diff_review_graph();
        // 10 nodes: config, prepare_diff, execute_diff, parse_diff,
        //           format_artifact, prepare_prompt, prepare_llm,
        //           execute_llm, parse_llm, parse_response
        assert_eq!(dag.nodes.len(), 10);
    }

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

        // prepare_diff has base_ref, repo_path as entrypoints (optional)
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
            .filter(|n| matches!(n.body, gunbc_ir::NodeBody::Opaque(ReviewGraphOp::Transport(_))))
            .collect();
        assert_eq!(transport_nodes.len(), 2, "should have execute_diff and execute_llm");
    }

    #[test]
    fn test_diff_review_graph_with_custom_config() {
        let config = ReviewPipelineConfig {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            criteria: crate::graph_mock::default_criteria(),
            default_branch: "develop".to_string(),
        };
        let dag = build_diff_review_graph_with(config);
        assert_eq!(dag.nodes.len(), 10);
    }

    // ========================================================================
    // MultiSourceReviewPhase tests
    // ========================================================================

    #[test]
    fn test_multi_source_review_graph_structure() {
        let dag = build_multi_source_review_graph();
        // 7 nodes: config, prepare_prompt, prepare_llm, execute_llm,
        //          parse_llm, parse_response, merge
        assert_eq!(dag.nodes.len(), 7);
    }

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
