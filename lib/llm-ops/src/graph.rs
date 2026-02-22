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

use gunbc_exec::DynOp;
use gunbc_ir::transport::cloud::CloudSecretConfig;
use gunbc_ir::{
    add_transport_triplet_named_with_passthrough, build::*, validate_authenticate_bindings,
    typed_port, AuthenticatePhase, AuthenticatePhaseBinding, BuilderError, CloudSecretConfigTag,
    Dag, DagBuilder, Node, NonEmptyStringTag, SecretNameTag,
};
use gunbc_lib_cloud_ops::{
    build_cloud_secret_manager_credential_graph_from_config, graph_cloud_config, CloudOps,
    CloudSecretManagerGraphOp,
};
use gunbc_lib_transport::TransportOps;

use crate::LlmOps;

pub type LlmGraphOp = DynOp;

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
        .unwrap_or_else(|err| panic!("chat completion graph should build: {err}"))
}

/// Build a chat completion DAG with explicit cloud config.
pub fn build_chat_completion_graph_with_config(
    cloud_config: CloudSecretConfig,
) -> Result<Dag<LlmGraphOp>, BuilderError> {
    validate_authenticate_bindings(&llm_authenticate_bindings())
        .map_err(|err| BuilderError::InternalInvariant(err.to_string()))?;

    let mut builder: DagBuilder<LlmGraphOp> = DagBuilder::new();

    // Node 0: Cloud environment (config + OIDC request inputs)
    let cloud_env = builder.add_root_node(Node::opaque(
        "cloud_env",
        vec![],
        vec![
            typed_port::<CloudSecretConfigTag>("config").into(),
            optional("request_url", "Url"),
            optional("request_token", "Secret"),
        ],
        DynOp::new(CloudOps::ConstCloudConfig {
            config: cloud_config.clone(),
        }),
    ))?;

    // Node 1: Resolve auth requirements (pure)
    let resolve_auth = builder.add_root_node(Node::opaque(
        "resolve_auth",
        vec![typed_port::<NonEmptyStringTag>("provider").into()],
        vec![
            typed_port::<NonEmptyStringTag>("service").into(),
            typed_port::<SecretNameTag>("secret_name").into(),
            optional("allow_impersonation", "OptionalBool"),
            typed_port::<NonEmptyStringTag>("scheme").into(),
            typed_port::<NonEmptyStringTag>("header_name").into(),
            list("required_scopes", "NonEmptyString"),
            typed_port::<bool>("interactive_allowed").into(),
        ],
        DynOp::new(LlmOps::ResolveAuth),
    ))?;

    // Node 3: Bind secret name onto cloud config
    let bind_secret = builder.add_node_after_all(
        Node::opaque(
            "bind_secret",
            vec![
                typed_port::<CloudSecretConfigTag>("config").into(),
                typed_port::<NonEmptyStringTag>("service").into(),
                typed_port::<SecretNameTag>("secret_name").into(),
            ],
            vec![typed_port::<CloudSecretConfigTag>("config").into()],
            DynOp::new(CloudOps::BindSecretName),
        ),
        &[&cloud_env, &resolve_auth],
    )?;

    // Node 4: Cloud credential acquisition graph (GCP WIF + Secret Manager)
    let cloud_subdag = lift_cloud_dag(build_cloud_secret_manager_credential_graph_from_config(
        &cloud_config,
    )?);
    let cloud_credential =
        builder.add_node_after(Node::subdag("cloud_credential", cloud_subdag), &bind_secret)?;

    // Node 5: Scope preflight gate (pure; fails fast on invalid/empty scopes)
    let scope_preflight = builder.add_node_after(
        Node::opaque(
            "scope_preflight",
            vec![list("required_scopes", "NonEmptyString")],
            vec![typed_port::<bool>("scope_verified").into()],
            DynOp::new(CloudOps::ScopePreflight),
        ),
        &resolve_auth,
    )?;

    // Chat completion transport triplet (prepare + execute + parse).
    let llm_triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "chat_completion",
        "prepare",
        "execute",
        "parse",
        vec![
            typed_port::<NonEmptyStringTag>("provider").into(),
            typed_port::<NonEmptyStringTag>("model").into(),
            typed_port::<serde_json::Value>("messages").into(),
            optional("system_prompt", "OptionalString"),
            optional("temperature", "OptionalJson"),
            optional("max_tokens", "OptionalInt"),
        ],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![typed_port::<NonEmptyStringTag>("provider").into()],
        vec![
            typed_port::<NonEmptyStringTag>("content").into(),
            typed_port::<NonEmptyStringTag>("model").into(),
            typed_port::<NonEmptyStringTag>("finish_reason").into(),
            typed_port::<i64>("input_tokens").into(),
            typed_port::<i64>("output_tokens").into(),
        ],
        DynOp::new(LlmOps::PrepareChatRequest),
        DynOp::new(LlmOps::ParseChatResponse),
        DynOp::new(TransportOps::Execute),
        Some(&cloud_credential),
    )?;

    // Edges: resolve_auth -> bind_secret -> cloud_credential -> triplet
    builder.add_edge(
        resolve_auth.out("required_scopes"),
        scope_preflight.in_port("required_scopes"),
    )?;
    builder.add_edge(
        scope_preflight.out("scope_verified"),
        llm_triplet.in_port("scope_verified"),
    )?;
    builder.add_edge(cloud_env.out("config"), bind_secret.in_port("config"))?;
    builder.add_edge(resolve_auth.out("service"), bind_secret.in_port("service"))?;
    builder.add_edge(
        resolve_auth.out("secret_name"),
        bind_secret.in_port("secret_name"),
    )?;
    builder.add_edge(
        resolve_auth.out("allow_impersonation"),
        cloud_credential.in_port("allow_impersonation"),
    )?;
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
    builder.add_edge(
        cloud_credential.out("credential"),
        llm_triplet.in_port("res:credential"),
    )?;

    Ok(builder.build())
}

fn llm_authenticate_bindings() -> Vec<AuthenticatePhaseBinding> {
    vec![
        AuthenticatePhaseBinding::new(AuthenticatePhase::ResolveContext, "cloud_env"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::SelectFlow, "resolve_auth"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::AcquireBaseIdentity, "cloud_credential"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::ExchangeOrDerive, "cloud_credential"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::MaybeImpersonate, "cloud_credential"),
        AuthenticatePhaseBinding::new(AuthenticatePhase::FinalizeCredential, "scope_preflight"),
    ]
}

fn lift_cloud_dag(dag: Dag<CloudSecretManagerGraphOp>) -> Dag<LlmGraphOp> {
    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn llm_authenticate_bindings_follow_canonical_chain() {
        assert!(validate_authenticate_bindings(&llm_authenticate_bindings()).is_ok());
    }

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
