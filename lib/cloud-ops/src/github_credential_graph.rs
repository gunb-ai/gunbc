//! GitHub credential lifecycle graph (cloud secret manager).
//!
//! Resolves GitHub credentials via the cloud secret manager and performs
//! a minimal GitHub API call to validate the token.

use crate::config_loader::graph_cloud_config;
use crate::graph::{
    build_cloud_secret_manager_credential_graph_from_config, CloudSecretManagerGraphOp,
};
use crate::ops::CloudOps;
use gunbc_delegate_macros::DelegateExecutable;
use gunbc_exec::{require_response, ExecError, Executable, OutputMap};
use gunbc_ir::build::{list, optional, port, resource};
use gunbc_ir::transport::gist::GITHUB_SECRET_ID;
use gunbc_ir::transport::github::api::github_rest_request;
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::{
    add_transport_triplet_named_with_passthrough, AccessMode, BuilderError, Dag, DagBuilder, Node,
    Value,
};
use gunbc_lib_transport::TransportOps;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum GitHubCredentialOps {
    /// Emit service/scheme for GitHub.
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
                .str("secret_name", GITHUB_SECRET_ID)
                .str("scheme", "bearer")
                .str("header_name", "")
                .str_list("required_scopes", vec!["github:api".to_string()])
                .bool("interactive_allowed", true)
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

#[derive(Debug, Clone, DelegateExecutable)]
pub enum GitHubCredentialGraphOp {
    Cloud(CloudSecretManagerGraphOp),
    GitHub(GitHubCredentialOps),
    Transport(TransportOps),
}

/// Build a minimal GitHub credential lifecycle graph.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "github-credential-graph",
    builder = "build_github_credential_graph()",
    returns_result
)]
pub fn build_github_credential_graph() -> Result<Dag<GitHubCredentialGraphOp>, BuilderError> {
    let config = graph_cloud_config();
    let mut builder: DagBuilder<GitHubCredentialGraphOp> = DagBuilder::new();

    // Cloud environment — pre-resolved config (no env var reads).
    let cloud_env = builder.add_root_node(Node::opaque(
        "cloud_env",
        vec![],
        vec![
            port("config", "CloudSecretConfig"),
            optional("request_url", "OptionalString"),
            optional("request_token", "OptionalString"),
        ],
        GitHubCredentialGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(
            CloudOps::ConstCloudConfig {
                config: config.clone(),
            },
        )),
    ))?;

    // Resolve auth (pure).
    let resolve_auth = builder.add_root_node(Node::opaque(
        "resolve_auth",
        vec![],
        vec![
            port("service", "String"),
            port("secret_name", "String"),
            port("scheme", "String"),
            port("header_name", "String"),
            list("required_scopes", "String"),
            port("interactive_allowed", "Bool"),
        ],
        GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::ResolveAuth),
    ))?;

    // Bind secret name into the cloud config.
    let bind_secret = builder.add_node_after_all(
        Node::opaque(
            "bind_secret",
            vec![
                port("config", "CloudSecretConfig"),
                port("service", "String"),
                optional("secret_name", "OptionalString"),
            ],
            vec![port("config", "CloudSecretConfig")],
            GitHubCredentialGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(
                CloudOps::BindSecretName,
            )),
        ),
        &[&cloud_env, &resolve_auth],
    )?;

    builder.add_edge(cloud_env.out("config"), bind_secret.in_port("config"))?;
    builder.add_edge(resolve_auth.out("service"), bind_secret.in_port("service"))?;
    builder.add_edge(
        resolve_auth.out("secret_name"),
        bind_secret.in_port("secret_name"),
    )?;

    // Cloud credential acquisition graph — dispatched from config.
    let cloud_subdag = lift_cloud_dag(build_cloud_secret_manager_credential_graph_from_config(
        &config,
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

    // Scope preflight: fail fast on invalid/missing required scope declarations.
    let scope_preflight = builder.add_node_after(
        Node::opaque(
            "scope_preflight",
            vec![list("required_scopes", "String")],
            vec![port("scope_verified", "Bool")],
            GitHubCredentialGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(
                CloudOps::ScopePreflight,
            )),
        ),
        &resolve_auth,
    )?;
    builder.add_edge(
        resolve_auth.out("required_scopes"),
        scope_preflight.in_port("required_scopes"),
    )?;

    // GitHub rate limit transport triplet.
    let triplet = add_transport_triplet_named_with_passthrough(
        &mut builder,
        "credential_check",
        "prepare_request",
        "execute",
        "parse_status",
        vec![],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![],
        vec![port("status", "Int"), port("ok", "Bool")],
        GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::PrepareRateLimit),
        GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::ParseStatus),
        GitHubCredentialGraphOp::Transport(TransportOps::Execute),
        Some(&cloud_credential),
    )?;
    builder.add_edge(
        scope_preflight.out("scope_verified"),
        triplet.in_port("scope_verified"),
    )?;

    builder.add_edge(
        cloud_credential.out("credential"),
        triplet.in_port("res:credential"),
    )?;

    Ok(builder.build())
}

fn lift_cloud_dag(dag: Dag<CloudSecretManagerGraphOp>) -> Dag<GitHubCredentialGraphOp> {
    dag.map_ops(&mut GitHubCredentialGraphOp::Cloud)
}
