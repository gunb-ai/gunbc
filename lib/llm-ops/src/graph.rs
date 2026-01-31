//! Graph builder for LLM chat completion workflows.
//!
//! Provides a composable DAG for the LLM chat completion pattern:
//!
//! ```text
//! PrepareChatRequest (pure) → TransportOps::Execute (I/O) → ParseChatResponse (pure)
//! ```
//!
//! This graph can be embedded as a sub-DAG in larger workflows that need
//! LLM capabilities (code review, code generation, etc.).

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::{build::*, Dag, Node, Value};
use gunbc_lib_transport::TransportOps;
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
}

impl Executable for LlmGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            LlmGraphOp::Llm(op) => op.execute(inputs),
            LlmGraphOp::Transport(op) => op.execute(inputs),
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
    let mut dag = Dag::new();

    // Node 1: PrepareChatRequest (pure)
    dag.add_node(Node::opaque(
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
        ],
        LlmGraphOp::Llm(LlmOps::PrepareChatRequest),
    ));

    // Node 2: Execute transport (I/O boundary)
    dag.add_node(Node::opaque(
        "execute",
        vec![port("request", "TransportRequest")],
        vec![port("response", "TransportResponse")],
        LlmGraphOp::Transport(TransportOps::Execute),
    ));

    // Node 3: ParseChatResponse (pure)
    dag.add_node(Node::opaque(
        "parse",
        vec![
            port("provider", "String"),
            port("response", "TransportResponse"),
        ],
        vec![
            port("content", "String"),
            port("model", "String"),
            port("finish_reason", "String"),
            port("input_tokens", "Int"),
            port("output_tokens", "Int"),
        ],
        LlmGraphOp::Llm(LlmOps::ParseChatResponse),
    ));

    // Edges: prepare -> execute -> parse
    dag.add_edge(edge("prepare", "request", "execute", "request"));
    dag.add_edge(edge("execute", "response", "parse", "response"));
    dag.add_edge(edge("prepare", "provider", "parse", "provider"));

    dag
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn test_chat_completion_graph_structure() {
        let dag = build_chat_completion_graph();
        assert_eq!(dag.nodes.len(), 3);
        assert_eq!(dag.edges.len(), 3);
    }

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
