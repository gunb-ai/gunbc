//! Cloud configuration ops.

use gunbc_exec::{
    optional_str_list_strict, optional_str_strict, require_str, ExecError, Executable, OutputMap,
};
use gunbc_ir::transport::cloud::{CloudProviderKind, CloudRuntimeKind, CloudSecretConfig};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudOps {
    /// Parse a CloudSecretConfig and emit standardized fields.
    ResolveConfig,
    /// Bind a secret name to the config (default: service).
    BindSecretName,
    /// Validate config and map to GCP-specific inputs.
    MapToGcpInputs { runtime: CloudRuntimeKind },
    /// Validate config and map to GCP-specific inputs for secret upsert.
    MapToGcpSecretInputs { runtime: CloudRuntimeKind },
    /// Emit a pre-resolved CloudSecretConfig as constant outputs.
    ///
    /// Replacement for CloudEnv: takes a serialized config and emits
    /// the same outputs (config, request_url, request_token) without
    /// reading any environment variables.
    ConstCloudConfig { config: CloudSecretConfig },
    /// Validate declared required scopes before business transport execution.
    ///
    /// This op is intentionally provider-neutral. It validates that
    /// `required_scopes` (when provided) are syntactically canonical.
    /// It emits `scope_verified=true` on success and errors on invalid scopes.
    ScopePreflight,
}

impl Executable for CloudOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CloudOps::ResolveConfig => {
                let config_val = inputs
                    .get("config")
                    .ok_or_else(|| ExecError::new("missing 'config' input"))?;
                let config = CloudSecretConfig::try_from(config_val)
                    .map_err(|e| ExecError::new(format!("invalid cloud config: {e}")))?;

                let mut out = OutputMap::new()
                    .str("provider", config.provider.as_str())
                    .str("runtime", config.runtime.as_str())
                    .str("audience", config.audience.as_str())
                    .str("project_or_account", config.project_or_account.as_str())
                    .str("secret", config.secret_name().as_str())
                    .ok()?;

                if let Some(version) = config.secret.version.as_ref() {
                    out.insert("version".to_string(), Value::Str(version.clone()));
                }
                if let Some(sa) = config.service_account_or_role.as_ref() {
                    out.insert(
                        "service_account_or_role".to_string(),
                        Value::Str(sa.clone()),
                    );
                }
                if let Some(impersonate) = config.impersonate_account_or_role.as_ref() {
                    out.insert(
                        "impersonate_account_or_role".to_string(),
                        Value::Str(impersonate.clone()),
                    );
                }

                Ok(out)
            }
            CloudOps::BindSecretName => {
                let config_val = inputs
                    .get("config")
                    .ok_or_else(|| ExecError::new("missing 'config' input"))?;
                let mut config = CloudSecretConfig::try_from(config_val)
                    .map_err(|e| ExecError::new(format!("invalid cloud config: {e}")))?;
                let service = require_str(&inputs, "service")?;
                let secret_name = optional_str_strict(&inputs, "secret_name")?.unwrap_or(service);

                config.secret.name = secret_name.to_string();

                OutputMap::new().value("config", config.into()).ok()
            }
            CloudOps::MapToGcpInputs { runtime } => {
                let provider = require_str(&inputs, "provider")?;
                if CloudProviderKind::parse(provider) != Some(CloudProviderKind::Gcp) {
                    return Err(ExecError::new(format!(
                        "cloud config provider '{provider}' is not gcp"
                    )));
                }

                let runtime_str = require_str(&inputs, "runtime")?;
                if CloudRuntimeKind::parse(runtime_str) != Some(*runtime) {
                    return Err(ExecError::new(format!(
                        "cloud config runtime '{runtime_str}' does not match expected '{}'",
                        runtime.as_str()
                    )));
                }

                let audience = require_str(&inputs, "audience")?;
                let project = require_str(&inputs, "project_or_account")?;
                let secret = require_str(&inputs, "secret")?;
                let required_scopes =
                    optional_str_list_strict(&inputs, "required_scopes")?.unwrap_or_default();

                let service_account = inputs
                    .get("impersonate_account_or_role")
                    .and_then(Value::as_str)
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        inputs
                            .get("service_account_or_role")
                            .and_then(Value::as_str)
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                    })
                    .unwrap_or("")
                    .to_string();

                let mut out = OutputMap::new()
                    .str("audience", audience)
                    .str("project", project)
                    .str("secret", secret)
                    .str("service_account", service_account)
                    .str_list("required_scopes", required_scopes)
                    .ok()?;

                if let Some(version) = inputs.get("version").and_then(Value::as_str) {
                    out.insert("version".to_string(), Value::Str(version.to_string()));
                }

                // Pass-through inputs required by the GCP graph.
                let scheme = require_str(&inputs, "scheme")?;
                let source_id = require_str(&inputs, "source_id")?;
                out.insert("scheme".to_string(), Value::Str(scheme.to_string()));
                out.insert("source_id".to_string(), Value::Str(source_id.to_string()));

                if let Some(header_name) = inputs.get("header_name").and_then(Value::as_str) {
                    out.insert(
                        "header_name".to_string(),
                        Value::Str(header_name.to_string()),
                    );
                }
                if let Some(allow_impersonation) =
                    inputs.get("allow_impersonation").and_then(Value::as_bool)
                {
                    out.insert(
                        "allow_impersonation".to_string(),
                        Value::Bool(allow_impersonation),
                    );
                }

                Ok(out)
            }
            CloudOps::ConstCloudConfig { config } => {
                // Emit the same outputs as CloudEnv would, but from a pre-resolved config.
                let mut out = OutputMap::new().value("config", config.clone().into());

                // For GitHub Actions runtime, provide the OIDC request inputs.
                // These are still from env because they're ephemeral per-job values,
                // not infrastructure configuration.
                if matches!(config.runtime, CloudRuntimeKind::GitHubActions) {
                    let request_url =
                        std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").unwrap_or_default();
                    let request_token =
                        std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN").unwrap_or_default();
                    out = out
                        .str("request_url", &request_url)
                        .str("request_token", &request_token);
                }

                out.ok()
            }
            CloudOps::ScopePreflight => {
                let required_scopes = optional_str_list_strict(&inputs, "required_scopes")?
                    .ok_or_else(|| ExecError::new("missing required scope declarations"))?;
                if required_scopes.is_empty() {
                    return Err(ExecError::new(
                        "required scope declarations cannot be empty",
                    ));
                }

                for scope in &required_scopes {
                    validate_scope_id(scope)?;
                }

                OutputMap::new().bool("scope_verified", true).ok()
            }
            CloudOps::MapToGcpSecretInputs { runtime } => {
                let provider = require_str(&inputs, "provider")?;
                if CloudProviderKind::parse(provider) != Some(CloudProviderKind::Gcp) {
                    return Err(ExecError::new(format!(
                        "cloud config provider '{provider}' is not gcp"
                    )));
                }

                let runtime_str = require_str(&inputs, "runtime")?;
                if CloudRuntimeKind::parse(runtime_str) != Some(*runtime) {
                    return Err(ExecError::new(format!(
                        "cloud config runtime '{runtime_str}' does not match expected '{}'",
                        runtime.as_str()
                    )));
                }

                let audience = require_str(&inputs, "audience")?;
                let project = require_str(&inputs, "project_or_account")?;
                let secret = require_str(&inputs, "secret")?;

                let service_account = inputs
                    .get("impersonate_account_or_role")
                    .and_then(Value::as_str)
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        inputs
                            .get("service_account_or_role")
                            .and_then(Value::as_str)
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                    })
                    .unwrap_or("")
                    .to_string();

                let mut out = OutputMap::new()
                    .str("audience", audience)
                    .str("project", project)
                    .str("secret", secret)
                    .str("service_account", service_account)
                    .ok()?;

                if let Some(version) = inputs.get("version").and_then(Value::as_str) {
                    out.insert("version".to_string(), Value::Str(version.to_string()));
                }

                if let Some(lifetime) = inputs.get("lifetime_seconds").and_then(Value::as_int) {
                    out.insert("lifetime_seconds".to_string(), Value::Int(lifetime));
                }
                if let Some(interactive_allowed) =
                    inputs.get("interactive_allowed").and_then(Value::as_bool)
                {
                    out.insert(
                        "interactive_allowed".to_string(),
                        Value::Bool(interactive_allowed),
                    );
                }
                if let Some(allow_impersonation) =
                    inputs.get("allow_impersonation").and_then(Value::as_bool)
                {
                    out.insert(
                        "allow_impersonation".to_string(),
                        Value::Bool(allow_impersonation),
                    );
                }

                if matches!(runtime, CloudRuntimeKind::GitHubActions) {
                    let request_url = require_str(&inputs, "request_url")?;
                    let request_token = require_str(&inputs, "request_token")?;
                    out.insert(
                        "request_url".to_string(),
                        Value::Str(request_url.to_string()),
                    );
                    out.insert(
                        "request_token".to_string(),
                        Value::Str(request_token.to_string()),
                    );
                }

                Ok(out)
            }
        }
    }
}

fn validate_scope_id(scope: &str) -> Result<(), ExecError> {
    if scope.trim() != scope {
        return Err(ExecError::new(format!(
            "scope '{}' has leading/trailing whitespace",
            scope
        )));
    }
    if scope.is_empty() {
        return Err(ExecError::new("scope id cannot be empty"));
    }
    if !scope.contains(':') {
        return Err(ExecError::new(format!(
            "scope '{}' must contain ':' delimiter",
            scope
        )));
    }
    if scope.starts_with(':') || scope.ends_with(':') || scope.contains("::") {
        return Err(ExecError::new(format!(
            "scope '{}' has invalid ':' placement",
            scope
        )));
    }
    if !scope.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == ':' || c == '_' || c == '-' || c == '.'
    }) {
        return Err(ExecError::new(format!(
            "scope '{}' contains invalid characters (allowed: [a-z0-9:_-.])",
            scope
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_to_gcp_inputs_allows_empty_service_account() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("gcp".to_string()));
        inputs.insert("runtime".to_string(), Value::Str("local".to_string()));
        inputs.insert("audience".to_string(), Value::Str("local-dev".to_string()));
        inputs.insert(
            "project_or_account".to_string(),
            Value::Str("gunbai-secrets".to_string()),
        );
        inputs.insert(
            "secret".to_string(),
            Value::Str("dev-github-token".to_string()),
        );
        inputs.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        inputs.insert("source_id".to_string(), Value::Str("github".to_string()));

        let out = CloudOps::MapToGcpInputs {
            runtime: CloudRuntimeKind::LocalDev,
        }
        .execute(inputs)
        .expect("missing SA should not fail mapping");

        assert_eq!(out.get("service_account").and_then(Value::as_str), Some(""));
    }

    #[test]
    fn map_to_gcp_inputs_prefers_impersonate_account() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("gcp".to_string()));
        inputs.insert("runtime".to_string(), Value::Str("local".to_string()));
        inputs.insert("audience".to_string(), Value::Str("local-dev".to_string()));
        inputs.insert(
            "project_or_account".to_string(),
            Value::Str("gunbai-secrets".to_string()),
        );
        inputs.insert(
            "secret".to_string(),
            Value::Str("dev-github-token".to_string()),
        );
        inputs.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        inputs.insert("source_id".to_string(), Value::Str("github".to_string()));
        inputs.insert(
            "service_account_or_role".to_string(),
            Value::Str("base@p.iam.gserviceaccount.com".to_string()),
        );
        inputs.insert(
            "impersonate_account_or_role".to_string(),
            Value::Str("imp@p.iam.gserviceaccount.com".to_string()),
        );

        let out = CloudOps::MapToGcpInputs {
            runtime: CloudRuntimeKind::LocalDev,
        }
        .execute(inputs)
        .expect("mapping should succeed");

        assert_eq!(
            out.get("service_account").and_then(Value::as_str),
            Some("imp@p.iam.gserviceaccount.com")
        );
    }

    #[test]
    fn map_to_gcp_inputs_passes_required_scopes() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("gcp".to_string()));
        inputs.insert("runtime".to_string(), Value::Str("local".to_string()));
        inputs.insert("audience".to_string(), Value::Str("local-dev".to_string()));
        inputs.insert(
            "project_or_account".to_string(),
            Value::Str("gunbai-secrets".to_string()),
        );
        inputs.insert(
            "secret".to_string(),
            Value::Str("dev-github-token".to_string()),
        );
        inputs.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        inputs.insert("source_id".to_string(), Value::Str("github".to_string()));
        inputs.insert(
            "required_scopes".to_string(),
            Value::str_list(vec![
                "llm:chat_completion".to_string(),
                "review:code_review".to_string(),
            ]),
        );

        let out = CloudOps::MapToGcpInputs {
            runtime: CloudRuntimeKind::LocalDev,
        }
        .execute(inputs)
        .expect("mapping should pass required scopes through");

        assert_eq!(
            out.get("required_scopes").and_then(Value::as_str_list),
            Some(vec![
                "llm:chat_completion".to_string(),
                "review:code_review".to_string(),
            ])
        );
    }

    #[test]
    fn map_to_gcp_inputs_passes_allow_impersonation() {
        let mut inputs = HashMap::new();
        inputs.insert("provider".to_string(), Value::Str("gcp".to_string()));
        inputs.insert("runtime".to_string(), Value::Str("local".to_string()));
        inputs.insert("audience".to_string(), Value::Str("local-dev".to_string()));
        inputs.insert(
            "project_or_account".to_string(),
            Value::Str("gunbai-secrets".to_string()),
        );
        inputs.insert(
            "secret".to_string(),
            Value::Str("dev-github-token".to_string()),
        );
        inputs.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        inputs.insert("source_id".to_string(), Value::Str("github".to_string()));
        inputs.insert("allow_impersonation".to_string(), Value::Bool(false));

        let out = CloudOps::MapToGcpInputs {
            runtime: CloudRuntimeKind::LocalDev,
        }
        .execute(inputs)
        .expect("mapping should pass allow_impersonation through");

        assert_eq!(out.get("allow_impersonation"), Some(&Value::Bool(false)));
    }

    #[test]
    fn scope_preflight_accepts_valid_scopes() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "required_scopes".to_string(),
            Value::str_list(vec![
                "gist:write".to_string(),
                "review:code_review".to_string(),
            ]),
        );

        let out = CloudOps::ScopePreflight
            .execute(inputs)
            .expect("valid required_scopes should pass");
        assert_eq!(out.get("scope_verified"), Some(&Value::Bool(true)));
    }

    #[test]
    fn scope_preflight_rejects_missing_required_scopes() {
        let mut inputs = HashMap::new();
        inputs.insert("skip".to_string(), Value::Bool(false));

        let err = CloudOps::ScopePreflight
            .execute(inputs)
            .expect_err("missing required_scopes should fail fast");
        assert!(
            err.to_string()
                .contains("missing required scope declarations"),
            "error should mention missing declarations, got: {}",
            err
        );
    }

    #[test]
    fn scope_preflight_rejects_empty_required_scopes() {
        let mut inputs = HashMap::new();
        inputs.insert("required_scopes".to_string(), Value::str_list(Vec::new()));

        let err = CloudOps::ScopePreflight
            .execute(inputs)
            .expect_err("empty required_scopes should fail fast");
        assert!(
            err.to_string()
                .contains("required scope declarations cannot be empty"),
            "error should mention empty declarations, got: {}",
            err
        );
    }

    #[test]
    fn scope_preflight_rejects_invalid_scope_id() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "required_scopes".to_string(),
            Value::str_list(vec!["BAD_SCOPE".to_string()]),
        );

        let err = CloudOps::ScopePreflight
            .execute(inputs)
            .expect_err("invalid scope format should fail");
        assert!(
            err.to_string().contains("must contain ':' delimiter"),
            "error should describe invalid scope format, got: {}",
            err
        );
    }
}
