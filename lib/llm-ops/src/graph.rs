//! Graph builder for LLM chat completion workflows.
//!
//! Provides a composable DAG for the LLM chat completion pattern:
//!
//! ```text
//! PrepareChatRequest (pure) → ResolveAuth (pure) → ScopePreflight (pure)
//!   → ConstCloudConfig → CloudSecretManager (subdag)
//!   → TransportOps::Execute (I/O) → ParseChatResponse (pure)
//! ```
//!
//! This graph can be embedded as a sub-DAG in larger workflows that need
//! LLM capabilities (code review, code generation, etc.).

use gunbc_delegate_macros::DelegateExecutable;
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::{add_transport_triplet_named_with_passthrough, build::*, Dag, DagBuilder, Node};
use gunbc_lib_cloud_ops::{
    build_cloud_secret_manager_credential_graph_from_config, graph_cloud_config, CloudOps,
    CloudSecretManagerGraphOp,
};
use gunbc_lib_transport::TransportOps;

use crate::LlmOps;

/// Operation type for LLM chat completion graphs.
///
/// Union of pure LLM ops and the transport boundary.
#[derive(Debug, Clone, DelegateExecutable)]
pub enum LlmGraphOp {
    /// Prepare a chat completion request (PURE - no I/O)
    Llm(LlmOps),
    /// Transport execution (BOUNDARY - actual I/O)
    Transport(TransportOps),
    /// Cloud credential flow (GCP/AWS/Azure graph)
    Cloud(CloudSecretManagerGraphOp),
}

/// Build a chat completion DAG.
///
/// The resulting DAG has these entrypoints (unconnected inputs):
/// - `chat_completion.provider`: String — provider ID ("openai", "anthropic")
/// - `chat_completion.model`: String — model identifier
/// - `chat_completion.messages`: JSON array of {role, content}
/// - `chat_completion.system_prompt`: Optional string
/// - `chat_completion.temperature`: Optional f64
/// - `chat_completion.max_tokens`: Optional i64
/// - `resolve_auth.provider`: String — provider ID (same value as above)
///
/// And these boundaries (unconnected outputs):
/// - `chat_completion.content`: String — generated text
/// - `chat_completion.model`: String — model that responded
/// - `chat_completion.finish_reason`: String
/// - `chat_completion.input_tokens`: Int
/// - `chat_completion.output_tokens`: Int
pub fn build_chat_completion_graph() -> Dag<LlmGraphOp> {
    build_chat_completion_graph_with_config(graph_cloud_config())
}

/// Build a chat completion DAG with explicit cloud config.
pub fn build_chat_completion_graph_with_config(cloud_config: CloudSecretConfig) -> Dag<LlmGraphOp> {
    let mut builder: DagBuilder<LlmGraphOp> = DagBuilder::new();

    // Node 0: Cloud environment (config + OIDC request inputs)
    let cloud_env = builder
        .add_root_node(Node::opaque(
            "cloud_env",
            vec![],
            vec![
                port("config", "CloudSecretConfig"),
                optional("request_url", "OptionalString"),
                optional("request_token", "OptionalString"),
            ],
            LlmGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(
                CloudOps::ConstCloudConfig {
                    config: cloud_config.clone(),
                },
            )),
        ))
        .expect("cloud_env node");

    // Node 1: Resolve auth requirements (pure)
    let resolve_auth = builder
        .add_root_node(Node::opaque(
            "resolve_auth",
            vec![port("provider", "String")],
            vec![
                port("service", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
                list("required_scopes", "String"),
                port("interactive_allowed", "Bool"),
            ],
            LlmGraphOp::Llm(LlmOps::ResolveAuth),
        ))
        .expect("resolve_auth node");

    // Node 3: Bind secret name onto cloud config
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
                LlmGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::BindSecretName)),
            ),
            &[&cloud_env, &resolve_auth],
        )
        .expect("bind_secret node");

    // Node 4: Cloud credential acquisition graph (GCP WIF + Secret Manager)
    let cloud_subdag = lift_cloud_dag(build_cloud_secret_manager_credential_graph_from_config(
        &cloud_config,
    ));
    let cloud_credential = builder
        .add_node_after(Node::subdag("cloud_credential", cloud_subdag), &bind_secret)
        .expect("cloud_credential node");

    // Node 5: Scope preflight gate (pure; fails fast on invalid/empty scopes)
    let scope_preflight = builder
        .add_node_after(
            Node::opaque(
                "scope_preflight",
                vec![list("required_scopes", "String")],
                vec![port("scope_verified", "Bool")],
                LlmGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(CloudOps::ScopePreflight)),
            ),
            &resolve_auth,
        )
        .expect("scope_preflight node");

    // Chat completion transport triplet (prepare + execute + parse).
    let llm_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "chat_completion",
        "prepare",
        "execute",
        "parse",
        vec![
            port("provider", "String"),
            port("model", "String"),
            port("messages", "Json"),
            optional("system_prompt", "OptionalString"),
            optional("temperature", "OptionalJson"),
            optional("max_tokens", "OptionalInt"),
        ],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![port("provider", "String")],
        vec![
            port("content", "String"),
            port("model", "String"),
            port("finish_reason", "String"),
            port("input_tokens", "Int"),
            port("output_tokens", "Int"),
        ],
        LlmGraphOp::Llm(LlmOps::PrepareChatRequest),
        LlmGraphOp::Llm(LlmOps::ParseChatResponse),
        LlmGraphOp::Transport(TransportOps::Execute),
        Some(&cloud_credential),
    )
    .expect("llm triplet");

    // Edges: resolve_auth -> bind_secret -> cloud_credential -> triplet
    builder
        .add_edge(
            resolve_auth.out("required_scopes"),
            scope_preflight.in_port("required_scopes"),
        )
        .expect("resolve_auth.required_scopes -> scope_preflight.required_scopes");
    builder
        .add_edge(
            scope_preflight.out("scope_verified"),
            llm_triplet.in_port("scope_verified"),
        )
        .expect("scope_preflight.scope_verified -> execute.scope_verified");
    builder
        .add_edge(cloud_env.out("config"), bind_secret.in_port("config"))
        .expect("cloud_env.config -> bind_secret.config");
    builder
        .add_edge(resolve_auth.out("service"), bind_secret.in_port("service"))
        .expect("resolve_auth.service -> bind_secret.service");
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
    builder
        .add_edge(
            cloud_credential.out("credential"),
            llm_triplet.in_port("res:credential"),
        )
        .expect("cloud_credential -> execute.res:credential");

    builder.build()
}

fn lift_cloud_dag(dag: Dag<CloudSecretManagerGraphOp>) -> Dag<LlmGraphOp> {
    dag.map_ops(&mut LlmGraphOp::Cloud)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_chat_completion_graph_boundaries() {
        let dag = build_chat_completion_graph();
        let boundaries = detect_boundaries(&dag);

        // The chat_completion SubDag's outputs are boundaries (no downstream)
        assert!(
            boundaries.is_boundary_node(&"chat_completion".into()),
            "chat_completion should be a boundary"
        );

        // execute node's response feeds into parse, so not a boundary output
        // But execute itself is the transport boundary (I/O happens here)
    }

    #[test]
    fn test_chat_completion_graph_entrypoints() {
        let dag = build_chat_completion_graph();
        let entrypoints = detect_entrypoints(&dag);

        // chat_completion SubDag's inputs are entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"chat_completion".into()),
            "chat_completion should have entrypoints"
        );
    }
}
