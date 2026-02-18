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
use gunbc_exec::{
    optional_str_list_strict, require_response, DynOp, ExecError, Executable, OutputMap,
};
use gunbc_ir::build::{list, optional, port, resource};
use gunbc_ir::transport::gist::GITHUB_SECRET_ID;
use gunbc_ir::transport::github::api::github_rest_request;
use gunbc_ir::transport::rest::RestResponse;
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::{
    add_transport_triplet_named_with_passthrough, AccessMode, BuilderError, Dag, DagBuilder, Node,
    Value,
};
use gunbc_lib_transport::TransportOps;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum GitHubCredentialOps {
    /// Emit service/scheme for GitHub.
    ResolveAuth,
    /// Build a REST request to a benign GitHub endpoint.
    PrepareRateLimit,
    /// Extract status + success from REST response.
    ParseStatus,
}

fn granted_scopes_from_headers(response: &RestResponse) -> HashSet<String> {
    response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("x-oauth-scopes"))
        .map(|(_, value)| {
            value
                .split(',')
                .map(|scope| scope.trim().to_ascii_lowercase())
                .filter(|scope| !scope.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn scope_aliases(required_scope: &str) -> Vec<String> {
    let normalized = required_scope.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut aliases = vec![normalized.clone()];
    if let Some(stripped) = normalized.strip_prefix("github:") {
        aliases.push(stripped.to_string());
    }
    if let Some((service, _action)) = normalized.split_once(':') {
        aliases.push(service.to_string());
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

fn is_scope_satisfied(required_scope: &str, granted_scopes: &HashSet<String>) -> bool {
    if required_scope.eq_ignore_ascii_case("github:api") {
        // A successful API response proves the token can call GitHub APIs.
        return true;
    }

    scope_aliases(required_scope)
        .iter()
        .any(|alias| granted_scopes.contains(alias))
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
                let required_scopes =
                    optional_str_list_strict(&inputs, "required_scopes")?.unwrap_or_default();
                let req = github_rest_request("/rate_limit");
                OutputMap::new()
                    .request("request", TransportRequest::Rest(req))
                    .bool("skip", false)
                    .str_list("required_scopes", required_scopes)
                    .ok()
            }
            GitHubCredentialOps::ParseStatus => {
                let response = require_response(&inputs, "response")?;
                let required_scopes =
                    optional_str_list_strict(&inputs, "required_scopes")?.unwrap_or_default();
                match response {
                    TransportResponse::Rest(rest) => {
                        if rest.is_success() && !required_scopes.is_empty() {
                            let granted_scopes = granted_scopes_from_headers(rest);
                            let mut missing = Vec::new();
                            for required in &required_scopes {
                                if !is_scope_satisfied(required, &granted_scopes) {
                                    missing.push(required.clone());
                                }
                            }
                            if !missing.is_empty() {
                                let mut granted = granted_scopes.into_iter().collect::<Vec<_>>();
                                granted.sort();
                                let granted_text = if granted.is_empty() {
                                    "<none>".to_string()
                                } else {
                                    granted.join(", ")
                                };
                                return Err(ExecError::new(format!(
                                    "GitHub token missing required scopes [{}]; granted [{}]",
                                    missing.join(", "),
                                    granted_text
                                )));
                            }
                        }

                        OutputMap::new()
                            .int("status", rest.status as i64)
                            .bool("ok", rest.is_success())
                            .ok()
                    }
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
        GitHubCredentialGraphOp::Cloud(DynOp::new(CloudOps::ConstCloudConfig {
            config: config.clone(),
        })),
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
            GitHubCredentialGraphOp::Cloud(DynOp::new(CloudOps::BindSecretName)),
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
            GitHubCredentialGraphOp::Cloud(DynOp::new(CloudOps::ScopePreflight)),
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
        vec![list("required_scopes", "String")],
        vec![
            optional("scope_verified", "OptionalBool"),
            resource("credential", "Credential", AccessMode::Read),
        ],
        vec![list("required_scopes", "String")],
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
        resolve_auth.out("required_scopes"),
        triplet.in_port("required_scopes"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::rest::RestResponse;
    use serde_json::json;

    #[test]
    fn parse_status_accepts_required_scope_from_provider_headers() {
        let mut headers = HashMap::new();
        headers.insert("x-oauth-scopes".to_string(), "repo, gist".to_string());
        let response = RestResponse {
            status: 200,
            headers,
            body: json!({}),
        };

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(response)),
        );
        inputs.insert(
            "required_scopes".to_string(),
            Value::str_list(vec!["gist:write".to_string()]),
        );

        let out = GitHubCredentialOps::ParseStatus
            .execute(inputs)
            .expect("granted scopes should satisfy required scope");
        assert_eq!(out.get("ok"), Some(&Value::Bool(true)));
        assert_eq!(out.get("status"), Some(&Value::Int(200)));
    }

    #[test]
    fn parse_status_rejects_missing_required_scope() {
        let mut headers = HashMap::new();
        headers.insert("x-oauth-scopes".to_string(), "repo".to_string());
        let response = RestResponse {
            status: 200,
            headers,
            body: json!({}),
        };

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(response)),
        );
        inputs.insert(
            "required_scopes".to_string(),
            Value::str_list(vec!["gist:write".to_string()]),
        );

        let err = GitHubCredentialOps::ParseStatus
            .execute(inputs)
            .expect_err("missing scope should fail credential verification");
        assert!(
            err.to_string().contains("missing required scopes"),
            "error should mention missing required scopes, got: {err}"
        );
    }

    #[test]
    fn parse_status_treats_github_api_scope_as_success_based() {
        let response = RestResponse::ok(json!({}));

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(response)),
        );
        inputs.insert(
            "required_scopes".to_string(),
            Value::str_list(vec!["github:api".to_string()]),
        );

        let out = GitHubCredentialOps::ParseStatus
            .execute(inputs)
            .expect("github:api should pass for successful API response");
        assert_eq!(out.get("ok"), Some(&Value::Bool(true)));
    }
}
