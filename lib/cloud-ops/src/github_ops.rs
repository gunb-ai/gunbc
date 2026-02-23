//! GitHub credential operations.
//!
//! Runtime ops for GitHub credential validation: resolve auth contract,
//! prepare rate-limit probe, and parse the response to verify scopes.

use gunbc_exec::{optional_str_list_strict, require_response, ExecError, Executable, OutputMap};
use gunbc_ir::transport::gist::GITHUB_SECRET_ID;
use gunbc_ir::transport::github::api::github_rest_request;
use gunbc_ir::transport::rest::RestResponse;
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::Value;
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
