//! Graph builder for LLM chat completion workflows.
//!
//! Provides a composable DAG for the LLM chat completion pattern:
//!
//! ```text
//! PrepareChatRequest (pure) → ResolveAuth (pure) → CredentialOp (env) → TransportOps::Execute (I/O) → ParseChatResponse (pure)
//! ```
//!
//! This graph can be embedded as a sub-DAG in larger workflows that need
//! LLM capabilities (code review, code generation, etc.).

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{
    add_transport_execute_parse_named_with_passthrough, build::*, Dag, DagBuilder, Node, Value,
};
use gunbc_lib_transport::{CredentialOp, TransportOps};
use std::collections::HashMap;

use crate::LlmOps;

/// Operation type for LLM chat completion graphs.
///
/// Union of pure LLM ops and the transport boundary.
#[derive(Debug, Clone)]
pub enum LlmGraphOp {
    /// Prepare a chat completion request (PURE - no I/O)
    Llm(LlmOps),
    /// Transport execution (BOUNDARY - actual I/O)
    Transport(TransportOps),
    /// Credential environment (BOUNDARY - resolves provider credentials)
    Cred(CredentialOp),
}

impl Executable for LlmGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            LlmGraphOp::Llm(op) => op.execute(inputs),
            LlmGraphOp::Transport(op) => op.execute(inputs),
            LlmGraphOp::Cred(op) => op.execute(inputs),
        }
    }
}

/// Build a chat completion DAG.
///
/// The resulting DAG has these entrypoints (unconnected inputs):
/// - `prepare.provider`: String — provider ID ("openai", "anthropic")
/// - `prepare.model`: String — model identifier
/// - `prepare.messages`: JSON array of {role, content}
/// - `prepare.system_prompt`: Optional string
/// - `prepare.temperature`: Optional f64
/// - `prepare.max_tokens`: Optional i64
///
/// And these boundaries (unconnected outputs):
/// - `parse.content`: String — generated text
/// - `parse.model`: String — model that responded
/// - `parse.finish_reason`: String
/// - `parse.input_tokens`: Int
/// - `parse.output_tokens`: Int
pub fn build_chat_completion_graph() -> Dag<LlmGraphOp> {
    let mut builder: DagBuilder<LlmGraphOp> = DagBuilder::new();

    // Node 1: PrepareChatRequest (pure)
    let prepare = builder
        .add_root_node(Node::opaque(
            "prepare",
            vec![
                port("provider", "String"),
                port("model", "String"),
                port("messages", "Json"),
                optional("system_prompt", "String"),
                optional("temperature", "Json"),
                optional("max_tokens", "Int"),
            ],
            vec![
                port("request", "TransportRequest"),
                port("provider", "String"),
                port("skip", "Bool"),
            ],
            LlmGraphOp::Llm(LlmOps::PrepareChatRequest),
        ))
        .expect("prepare node");

    // Node 2: Resolve auth requirements (pure)
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
                LlmGraphOp::Llm(LlmOps::ResolveAuth),
            ),
            &prepare,
        )
        .expect("resolve_auth node");

    // Node 3: Credential environment (resolves provider credentials)
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
                LlmGraphOp::Cred(CredentialOp::from_inputs(cred_port)),
            ),
            &resolve_auth,
        )
        .expect("credential_env node");

    // Nodes 4-5: Execute transport + ParseChatResponse
    let llm_triplet = add_transport_execute_parse_named_with_passthrough(
        &mut builder,
        &prepare,
        "execute",
        "parse",
        vec![port("provider", "String")],
        vec![resource("credential", "Credential", AccessMode::Read)],
        vec![
            port("content", "String"),
            port("model", "String"),
            port("finish_reason", "String"),
            port("input_tokens", "Int"),
            port("output_tokens", "Int"),
        ],
        LlmGraphOp::Llm(LlmOps::ParseChatResponse),
        LlmGraphOp::Transport(TransportOps::Execute),
        Some(&credential_env),
    )
    .expect("llm triplet");

    // Edges: prepare -> resolve_auth -> credential_env -> execute -> parse
    builder
        .add_edge(prepare.out("provider"), resolve_auth.in_port("provider"))
        .expect("prepare.provider -> resolve_auth.provider");
    builder
        .add_edge(
            resolve_auth.out("service"),
            credential_env.in_port("service"),
        )
        .expect("resolve_auth.service -> credential_env.service");
    builder
        .add_edge(resolve_auth.out("env_var"), credential_env.in_port("env_var"))
        .expect("resolve_auth.env_var -> credential_env.env_var");
    builder
        .add_edge(resolve_auth.out("scheme"), credential_env.in_port("scheme"))
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
        .expect("credential_env -> execute.res:credential");

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_chat_completion_graph_boundaries() {
        let dag = build_chat_completion_graph();
        let boundaries = detect_boundaries(&dag);

        // The parse node's outputs are boundaries (no downstream)
        assert!(
            boundaries.is_boundary_node(&"parse".into()),
            "parse should be a boundary"
        );

        // execute node's response feeds into parse, so not a boundary output
        // But execute itself is the transport boundary (I/O happens here)
    }

    #[test]
    fn test_chat_completion_graph_entrypoints() {
        let dag = build_chat_completion_graph();
        let entrypoints = detect_entrypoints(&dag);

        // prepare node's inputs are entrypoints
        assert!(
            entrypoints.is_entrypoint_node(&"prepare".into()),
            "prepare should have entrypoints"
        );
    }
}
