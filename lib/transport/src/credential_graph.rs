//! Credential lifecycle graph for GitHub.
//!
//! Provides a minimal DAG that:
//! resolve_auth → credential_env → execute → parse_status
//!
//! This is used for credential lifecycle tests and live integration checks.

use crate::{CredentialOp, TransportOps};
use gunbc_exec::{require_response, ExecError, Executable, OutputMap};
use gunbc_ir::build::{port, resource};
use gunbc_ir::{
    add_transport_execute_parse_named_with_passthrough, AccessMode, Dag, DagBuilder, Node, Value,
};
use gunbc_ir::transport::github::api::github_rest_request;
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum GitHubCredentialOps {
    /// Emit service/env_var/scheme for GitHub.
    ResolveAuth,
    /// Build a REST request to a benign GitHub endpoint.
    PrepareRateLimit,
    /// Extract status + success from REST response.
    ParseStatus,
}

impl Executable for GitHubCredentialOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GitHubCredentialOps::ResolveAuth => OutputMap::new()
                .str("service", "github")
                .str("env_var", "GITHUB_TOKEN")
                .str("scheme", "bearer")
                .str("header_name", "")
                .ok(),
            GitHubCredentialOps::PrepareRateLimit => {
                let req = github_rest_request("/rate_limit");
                OutputMap::new()
                    .request("request", TransportRequest::Rest(req))
                    .bool("skip", false)
                    .ok()
            }
            GitHubCredentialOps::ParseStatus => {
                let response = require_response(&inputs, "response")?;
                match response {
                    TransportResponse::Rest(rest) => OutputMap::new()
                        .int("status", rest.status as i64)
                        .bool("ok", rest.is_success())
                        .ok(),
                    other => Err(ExecError::new(format!(
                        "expected REST response, got {:?}",
                        other
                    ))),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum GitHubCredentialGraphOp {
    GitHub(GitHubCredentialOps),
    Cred(CredentialOp),
    Transport(TransportOps),
}

impl Executable for GitHubCredentialGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GitHubCredentialGraphOp::GitHub(op) => op.execute(inputs),
            GitHubCredentialGraphOp::Cred(op) => op.execute(inputs),
            GitHubCredentialGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build a minimal GitHub credential lifecycle graph.
pub fn build_github_credential_graph() -> Dag<GitHubCredentialGraphOp> {
    let mut builder: DagBuilder<GitHubCredentialGraphOp> = DagBuilder::new();

    // Node 1: Resolve auth (pure)
    let resolve_auth = builder
        .add_root_node(Node::opaque(
            "resolve_auth",
            vec![],
            vec![
                port("service", "String"),
                port("env_var", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
            ],
            GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::ResolveAuth),
        ))
        .expect("resolve_auth node");

    // Node 2: Credential environment (resolves GitHub token)
    let cred_port = "credential:github";
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
                GitHubCredentialGraphOp::Cred(CredentialOp::from_inputs(cred_port)),
            ),
            &resolve_auth,
        )
        .expect("credential_env node");

    // Node 3: Prepare request (pure)
    let prepare = builder
        .add_node_after(
            Node::opaque(
                "prepare_request",
                vec![],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::PrepareRateLimit),
            ),
            &credential_env,
        )
        .expect("prepare_request node");

    // Nodes 4-5: Execute transport + ParseStatus
    let triplet = add_transport_execute_parse_named_with_passthrough(
        &mut builder,
        &prepare,
        "execute",
        "parse_status",
        vec![],
        vec![resource("credential", "Credential", AccessMode::Read)],
        vec![port("status", "Int"), port("ok", "Bool")],
        GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::ParseStatus),
        GitHubCredentialGraphOp::Transport(TransportOps::Execute),
        Some(&credential_env),
    )
    .expect("transport triplet");

    builder
        .add_edge(
            credential_env.out(cred_port),
            triplet.execute.in_port("res:credential"),
        )
        .expect("credential_env -> execute.res:credential");

    builder.build()
}
