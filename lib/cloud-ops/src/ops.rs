//! Cloud configuration ops.

use gunbc_exec::{optional_str_strict, require_str, ExecError, Executable, OutputMap};
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

                let service_account = match inputs
                    .get("impersonate_account_or_role")
                    .and_then(Value::as_str)
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    Some(value) => value.to_string(),
                    None => match inputs
                        .get("service_account_or_role")
                        .and_then(Value::as_str)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                    {
                        Some(value) => value.to_string(),
                        None => {
                            return Err(ExecError::new(
                                "missing service_account_or_role for gcp config",
                            ))
                        }
                    },
                };

                let mut out = OutputMap::new()
                    .str("audience", audience)
                    .str("project", project)
                    .str("secret", secret)
                    .str("service_account", service_account)
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

                if let Some(lifetime) = inputs.get("lifetime_seconds").and_then(Value::as_int) {
                    out.insert("lifetime_seconds".to_string(), Value::Int(lifetime));
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

                let service_account = match inputs
                    .get("impersonate_account_or_role")
                    .and_then(Value::as_str)
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    Some(value) => value.to_string(),
                    None => match inputs
                        .get("service_account_or_role")
                        .and_then(Value::as_str)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                    {
                        Some(value) => value.to_string(),
                        None => {
                            return Err(ExecError::new(
                                "missing service_account_or_role for gcp config",
                            ))
                        }
                    },
                };

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
