//! GitHub credential lifecycle graph (cloud secret manager).
//!
//! Resolves GitHub credentials via the cloud secret manager and performs
//! a minimal GitHub API call to validate the token.

use crate::env::CloudEnv;
use crate::graph::{build_cloud_secret_manager_credential_graph_gcp_github, CloudSecretManagerGraphOp};
use crate::ops::CloudOps;
use gunbc_exec::{require_response, ExecError, Executable, OutputMap};
use gunbc_ir::build::{optional, port, resource};
use gunbc_ir::transport::github::api::github_rest_request;
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::{
    add_transport_execute_parse_named_with_passthrough, AccessMode, Dag, DagBuilder, Node,
    NodeBody, Value,
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
    CloudEnv(CloudEnv),
    Cloud(CloudSecretManagerGraphOp),
    GitHub(GitHubCredentialOps),
    Transport(TransportOps),
}

impl Executable for GitHubCredentialGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GitHubCredentialGraphOp::CloudEnv(op) => op.execute(inputs),
            GitHubCredentialGraphOp::Cloud(op) => op.execute(inputs),
            GitHubCredentialGraphOp::GitHub(op) => op.execute(inputs),
            GitHubCredentialGraphOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build a minimal GitHub credential lifecycle graph.
#[gunbc_testgen_registry_macros::resource_test_target(
    skip,
    name = "github-credential-graph",
    builder = "build_github_credential_graph()",
)]
pub fn build_github_credential_graph() -> Dag<GitHubCredentialGraphOp> {
    let mut builder: DagBuilder<GitHubCredentialGraphOp> = DagBuilder::new();

    // Cloud environment (config + OIDC request inputs).
    let cloud_env = builder
        .add_root_node(Node::opaque(
            "cloud_env",
            vec![],
            vec![
                port("config", "CloudSecretConfig"),
                optional("request_url", "OptionalString"),
                optional("request_token", "OptionalString"),
            ],
            GitHubCredentialGraphOp::CloudEnv(CloudEnv::new()),
        ))
        .expect("cloud_env node");

    // Resolve auth (pure).
    let resolve_auth = builder
        .add_root_node(Node::opaque(
            "resolve_auth",
            vec![],
            vec![
                port("service", "String"),
                port("scheme", "String"),
                port("header_name", "String"),
            ],
            GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::ResolveAuth),
        ))
        .expect("resolve_auth node");

    // Bind secret name into the cloud config.
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
                GitHubCredentialGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(
                    CloudOps::BindSecretName,
                )),
            ),
            &[&cloud_env, &resolve_auth],
        )
        .expect("bind_secret node");

    builder
        .add_edge(cloud_env.out("config"), bind_secret.in_port("config"))
        .expect("cloud_env.config -> bind_secret.config");
    builder
        .add_edge(resolve_auth.out("service"), bind_secret.in_port("service"))
        .expect("resolve_auth.service -> bind_secret.service");

    // Cloud credential acquisition graph (GCP WIF + Secret Manager).
    let cloud_subdag = lift_cloud_dag(build_cloud_secret_manager_credential_graph_gcp_github());
    let cloud_credential = builder
        .add_node_after(Node::subdag("cloud_credential", cloud_subdag), &bind_secret)
        .expect("cloud_credential node");

    builder
        .add_edge(bind_secret.out("config"), cloud_credential.in_port("config"))
        .expect("bind_secret.config -> cloud_credential.config");
    builder
        .add_edge(
            resolve_auth.out("service"),
            cloud_credential.in_port("source_id"),
        )
        .expect("resolve_auth.service -> cloud_credential.source_id");
    builder
        .add_edge(resolve_auth.out("scheme"), cloud_credential.in_port("scheme"))
        .expect("resolve_auth.scheme -> cloud_credential.scheme");
    builder
        .add_edge(
            resolve_auth.out("header_name"),
            cloud_credential.in_port("header_name"),
        )
        .expect("resolve_auth.header_name -> cloud_credential.header_name");
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

    // GitHub rate limit request (pure).
    let prepare = builder
        .add_node_after(
            Node::opaque(
                "prepare_request",
                vec![],
                vec![port("request", "TransportRequest"), port("skip", "Bool")],
                GitHubCredentialGraphOp::GitHub(GitHubCredentialOps::PrepareRateLimit),
            ),
            &cloud_credential,
        )
        .expect("prepare_request node");

    // Execute transport + ParseStatus.
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
        Some(&cloud_credential),
    )
    .expect("transport triplet");

    builder
        .add_edge(
            cloud_credential.out("credential"),
            triplet.execute.in_port("res:credential"),
        )
        .expect("cloud_credential -> execute.res:credential");

    builder.build()
}

fn lift_cloud_dag(dag: Dag<CloudSecretManagerGraphOp>) -> Dag<GitHubCredentialGraphOp> {
    let mut lift = |op| GitHubCredentialGraphOp::Cloud(op);
    map_dag_ops(dag, &mut lift)
}

fn map_dag_ops<T, U, F>(dag: Dag<T>, f: &mut F) -> Dag<U>
where
    T: Clone,
    U: Clone,
    F: FnMut(T) -> U,
{
    let mut out = Dag::new();
    out.edges = dag.edges.clone();
    out.nodes = dag
        .nodes
        .into_iter()
        .map(|node| map_node_ops(node, f))
        .collect();
    out
}

fn map_node_ops<T, U, F>(node: Node<T>, f: &mut F) -> Node<U>
where
    T: Clone,
    U: Clone,
    F: FnMut(T) -> U,
{
    let Node {
        id,
        inputs,
        outputs,
        body,
        examples,
    } = node;
    let body = match body {
        NodeBody::Opaque(op) => NodeBody::Opaque(f(op)),
        NodeBody::SubDag(subdag) => NodeBody::SubDag(map_dag_ops(subdag, f)),
    };
    Node {
        id,
        inputs,
        outputs,
        body,
        examples,
    }
}
