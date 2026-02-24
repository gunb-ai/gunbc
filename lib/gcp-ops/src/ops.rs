//! Pure GCP ops for WIF + Secret Manager.

use gunbc_exec::{
    optional_bool_strict, optional_str_list_strict, require_bool, require_str,
    ExecError, Executable, OutputMap,
};
use gunbc_ir::transport::file::FileRequest;
use gunbc_ir::transport::rest::RestRequest;
use gunbc_ir::transport::{ShellRequest, TransportResponse};
use gunbc_ir::{AuthScheme, Credential, Secret, SecretSource, Value};

use crate::services::iam::{IamRest, IamService};
use crate::services::local_auth::{GcloudCli, GcloudLoginOptions, LocalAuthService};
use crate::services::resource_manager::{ResourceManagerRest, ResourceManagerService};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Runtime environment used to acquire OIDC tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GcpRuntimeKind {
    /// GitHub Actions OIDC.
    GitHubActions,
    /// GCP metadata server (GCE / GKE / Cloud Run).
    GcpMetadata,
    /// Local developer workstation using gcloud auth.
    LocalDev,
}

impl GcpRuntimeKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "github" | "github-actions" => Some(GcpRuntimeKind::GitHubActions),
            "gcp" | "gcp-metadata" | "metadata" => Some(GcpRuntimeKind::GcpMetadata),
            "local" | "local-dev" | "dev" => Some(GcpRuntimeKind::LocalDev),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GcpOps {
    /// Resolve runtime kind from an input string ("github" or "gcp").
    ResolveRuntime,
    /// Prepare to check if the ADC file exists (file transport).
    PrepareCheckAdc,
    /// Parse ADC file existence check response.
    ParseCheckAdc,
    /// Prepare to read the ADC credentials file (file transport).
    PrepareReadAdc,
    /// Parse ADC credentials JSON (extract client_id, client_secret, refresh_token).
    ParseAdcCredentials,
    /// Prepare OAuth2 token refresh request (REST transport).
    PrepareOAuth2Refresh,
    /// Parse OAuth2 token refresh response (extract access_token, expires_in).
    ParseOAuth2Refresh,
    /// Prepare the IAM service account impersonation request.
    PrepareImpersonate,
    /// Parse the impersonation response.
    ParseImpersonate,
    /// Prepare Secret Manager access request.
    PrepareSecretAccess,
    /// Parse Secret Manager access response (base64 decode).
    ParseSecretAccess,
    /// Prepare Secret Manager get secret request.
    PrepareSecretGet,
    /// Parse Secret Manager get secret response (exists bool).
    ParseSecretGet,
    /// Prepare Secret Manager create secret request.
    PrepareSecretCreate,
    /// Parse Secret Manager create secret response.
    ParseSecretCreate,
    /// Prepare Secret Manager add version request.
    PrepareSecretAddVersion,
    /// Parse Secret Manager add version response.
    ParseSecretAddVersion,
    /// Build a credential from a secret for a specific service.
    BuildCredential,
    /// Determine if impersonation should be used (non-empty SA).
    ShouldImpersonate,
    /// Compose a secret name from prefix + service (optional delimiter).
    ComposeSecretName,
    /// Try OAuth2 token refresh — catches auth errors as recoverable.
    ///
    /// Like `ParseOAuth2Refresh` but instead of failing on auth errors
    /// (invalid_rapt, invalid_grant, etc.), outputs `needs_reauth: true`
    /// so the DAG can fall back to `gcloud auth login --update-adc`.
    ParseTryRefresh,
    /// Prepare `gcloud auth login --update-adc` shell command.
    ///
    /// Accepts `needs_reauth: Bool` — when false, outputs `skip: true`.
    PrepareGcloudAuth,
    /// Parse the result of `gcloud auth login --update-adc`.
    ///
    /// Validates exit code 0 and outputs `ok: Bool`.
    ParseGcloudAuth,
    /// Merge auth results from the try-refresh and retry-refresh branches.
    ///
    /// Takes optional access_token/expires_in from both paths and outputs
    /// the non-skipped values.
    MergeAuthResult,
    /// Prepare a REST request to read the IAM policy of a GCP project.
    ///
    /// Accepts `access_token`, `project`, and `service_account`.
    /// Skips when `service_account` is empty.
    /// Outputs a REST request for `getIamPolicy` + the SA and role for
    /// downstream binding check.
    PrepareEnsureIamBinding,
    /// Check the IAM policy and, if the required binding is missing,
    /// output a `setIamPolicy` request.
    ///
    /// Takes the `getIamPolicy` response, `access_token`, `project`,
    /// and `service_account`. If the binding already exists, outputs
    /// `skip: true`. Otherwise outputs a `setIamPolicy` REST request
    /// with the updated policy (preserving etag for optimistic concurrency).
    CheckAndPrepareIamBinding,
    /// Parse the result of `setIamPolicy` (or handle skip).
    ///
    /// Outputs `ok: Bool`.
    ParseSetIamBinding,
    /// Prepare a REST request to read IAM policy for a specific service account.
    ///
    /// Accepts `access_token`, `project`, `service_account`, and `member`.
    /// Skips when any required input is empty.
    PrepareEnsureSaIamBinding,
    /// Check service-account IAM policy and output setIamPolicy request if missing.
    ///
    /// Ensures `member` has `roles/iam.workloadIdentityUser` on the target SA.
    CheckAndPrepareSaIamBinding,
    /// Parse the result of service-account `setIamPolicy` (or handle skip).
    ParseSetSaIamBinding,
}

impl Executable for GcpOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GcpOps::ResolveRuntime => {
                let raw = require_str(&inputs, "runtime")?;
                let kind = GcpRuntimeKind::parse(raw)
                    .ok_or_else(|| ExecError::new("unknown runtime (expected github|gcp|local)"))?;
                let out = match kind {
                    GcpRuntimeKind::GitHubActions => "github",
                    GcpRuntimeKind::GcpMetadata => "gcp",
                    GcpRuntimeKind::LocalDev => "local",
                };
                OutputMap::new().str("runtime", out).ok()
            }
            GcpOps::PrepareCheckAdc => {
                let adc_path = adc_file_path();
                let req = FileRequest::exists(&adc_path);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseCheckAdc => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().bool("exists", false).ok(),
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let file = match response {
                    TransportResponse::File(f) => f,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected File response, got {:?}",
                            other
                        )));
                    }
                };
                let exists = file.exists.unwrap_or(false);
                OutputMap::new().bool("exists", exists).ok()
            }
            GcpOps::PrepareReadAdc => {
                let exists = match inputs.get("exists") {
                    Some(Value::Bool(b)) => *b,
                    _ => false,
                };
                if !exists {
                    let path = adc_file_path();
                    return Err(ExecError::new(format!(
                        "ADC file not found at {path}. Run `gcloud auth application-default login` and retry."
                    )));
                }
                let adc_path = adc_file_path();
                let req = FileRequest::read(&adc_path);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseAdcCredentials => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new()
                            .str("client_id", "")
                            .str("client_secret", "")
                            .str("refresh_token", "")
                            .str("token_type", "")
                            .ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let file = match response {
                    TransportResponse::File(f) => f,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected File response, got {:?}",
                            other
                        )));
                    }
                };
                let content = file.content.as_deref().ok_or_else(|| {
                    let path = adc_file_path();
                    ExecError::new(format!(
                        "ADC file at {path} is empty or unreadable. Run `gcloud auth application-default login` to recreate it."
                    ))
                })?;
                let json: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| ExecError::new(format!("ADC file is not valid JSON: {e}")))?;

                let client_id = json.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
                let client_secret = json
                    .get("client_secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let refresh_token = json
                    .get("refresh_token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let token_type = json
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("authorized_user");

                if refresh_token.is_empty() {
                    return Err(ExecError::new(
                        "ADC file missing refresh_token. Run `gcloud auth application-default login` to recreate."
                    ));
                }

                OutputMap::new()
                    .str("client_id", client_id)
                    .str("client_secret", client_secret)
                    .str("refresh_token", refresh_token)
                    .str("token_type", token_type)
                    .ok()
            }
            GcpOps::PrepareOAuth2Refresh => {
                let client_id = require_str(&inputs, "client_id")?;
                let client_secret = require_str(&inputs, "client_secret")?;
                let refresh_token = require_str(&inputs, "refresh_token")?;

                // If credentials are empty (upstream was skipped), skip the OAuth2 call.
                if client_id.is_empty() || client_secret.is_empty() || refresh_token.is_empty() {
                    let placeholder = RestRequest::post("https://oauth2.googleapis.com/token");
                    return OutputMap::new()
                        .request("request", placeholder.into())
                        .bool("skip", true)
                        .ok();
                }

                let body = serde_json::json!({
                    "client_id": client_id,
                    "client_secret": client_secret,
                    "refresh_token": refresh_token,
                    "grant_type": "refresh_token",
                });

                let req = RestRequest::post("https://oauth2.googleapis.com/token").json(body);

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseOAuth2Refresh => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new()
                            .str("access_token", "")
                            .int("expires_in", 0)
                            .ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };
                if !rest.is_success() {
                    let error_desc = rest
                        .body
                        .get("error_description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown error");
                    return Err(ExecError::new(format!(
                        "OAuth2 token refresh failed (status {}): {}",
                        rest.status, error_desc
                    )));
                }
                let access_token = rest
                    .body
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ExecError::new("missing access_token in OAuth2 refresh response")
                    })?;
                let expires_in = rest
                    .body
                    .get("expires_in")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(3599);
                OutputMap::new()
                    .str("access_token", access_token)
                    .int("expires_in", expires_in)
                    .ok()
            }
            GcpOps::PrepareImpersonate => {
                let access_token = require_str(&inputs, "access_token")?;
                let service_account = require_str(&inputs, "service_account")?;
                let should_impersonate = optional_bool_strict(&inputs, "should_impersonate")?
                    .unwrap_or_else(|| !service_account.trim().is_empty());
                if !should_impersonate || service_account.trim().is_empty() {
                    // No impersonation target configured; downstream parse node will
                    // fall back to the base access token.
                    return OutputMap::new()
                        .request(
                            "request",
                            RestRequest::post("https://example.invalid/impersonate").into(),
                        )
                        .bool("skip", true)
                        .ok();
                }
                let lifetime_seconds = match inputs.get("lifetime_seconds") {
                    Some(value) => value.as_int().ok_or_else(|| {
                        ExecError::new("missing or invalid 'lifetime_seconds' input")
                    })?,
                    None => 3600,
                };

                let body = serde_json::json!({
                    "scope": ["https://www.googleapis.com/auth/cloud-platform"],
                    "lifetime": format!("{}s", lifetime_seconds),
                });

                let url = format!(
                    "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{}:generateAccessToken",
                    service_account
                );
                let req = RestRequest::post(url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .json(body);

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseImpersonate => {
                let base_access_token =
                    optional_secret_or_str(&inputs, "base_access_token")?.unwrap_or_default();
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new()
                            .str("access_token", base_access_token)
                            .str("expires_at", "")
                            .ok()
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };
                if !rest.is_success() {
                    let details = impersonation_error_summary(&rest.body);
                    return Err(ExecError::new(format!(
                        "impersonation failed (status {}): {}",
                        rest.status, details
                    )));
                }
                let token = rest
                    .body
                    .get("accessToken")
                    .or_else(|| rest.body.get("access_token"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        let details = impersonation_error_summary(&rest.body);
                        ExecError::new(format!(
                            "impersonation response missing accessToken (status {}): {}",
                            rest.status, details
                        ))
                    })?;
                let expires_at = rest
                    .body
                    .get("expireTime")
                    .or_else(|| rest.body.get("expire_time"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                OutputMap::new()
                    .str("access_token", token)
                    .str("expires_at", expires_at)
                    .ok()
            }
            GcpOps::PrepareSecretAccess => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let secret = require_str(&inputs, "secret")?;
                let version = match inputs.get("version") {
                    Some(value) => value
                        .as_str()
                        .ok_or_else(|| ExecError::new("missing or invalid 'version' input"))?,
                    None => "latest",
                };

                let url = format!(
                    "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}/versions/{}:access",
                    project, secret, version
                );
                let req = RestRequest::get(url)
                    .header("Authorization", format!("Bearer {}", access_token));

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseSecretAccess => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().str("secret", "").ok(),
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };
                if !rest.is_success() {
                    let details = impersonation_error_summary(&rest.body);
                    return Err(ExecError::new(format!(
                        "Secret Manager access failed (status {}): {}",
                        rest.status, details
                    )));
                }
                let data = rest
                    .body
                    .get("payload")
                    .and_then(|p| p.get("data"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExecError::new("missing payload.data in secret response"))?;
                let decoded = base64_decode(data)
                    .map_err(|e| ExecError::new(format!("base64 decode failed: {}", e)))?;
                let secret = String::from_utf8(decoded)
                    .map_err(|e| ExecError::new(format!("secret not utf8: {}", e)))?;
                OutputMap::new().str("secret", secret).ok()
            }
            GcpOps::PrepareSecretGet => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let secret = require_str(&inputs, "secret")?;

                let url = format!(
                    "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}",
                    project, secret
                );
                let req = RestRequest::get(url)
                    .header("Authorization", format!("Bearer {}", access_token));

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseSecretGet => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().bool("exists", false).ok(),
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };
                let exists = match rest.status {
                    200..=299 => true,
                    404 => false,
                    other => {
                        return Err(ExecError::new(format!(
                            "unexpected status {} when checking secret",
                            other
                        )));
                    }
                };
                OutputMap::new().bool("exists", exists).ok()
            }
            GcpOps::PrepareSecretCreate => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let secret = require_str(&inputs, "secret")?;
                let exists = require_bool(&inputs, "exists")?;

                let url = format!(
                    "https://secretmanager.googleapis.com/v1/projects/{}/secrets",
                    project
                );
                let body = serde_json::json!({
                    "replication": { "automatic": {} }
                });
                let req = RestRequest::post(url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .query("secretId", secret)
                    .json(body);

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", exists)
                    .ok()
            }
            GcpOps::ParseSecretCreate => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().bool("ok", true).ok(),
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };
                if !(200..=299).contains(&rest.status) {
                    return Err(ExecError::new(format!(
                        "secret create failed (status {})",
                        rest.status
                    )));
                }
                OutputMap::new().bool("ok", true).ok()
            }
            GcpOps::PrepareSecretAddVersion => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let secret = require_str(&inputs, "secret")?;
                if let Some(value) = inputs.get("create_done") {
                    if !matches!(value, Value::Skipped) {
                        value.as_bool().ok_or_else(|| {
                            ExecError::new("missing or invalid 'create_done' input")
                        })?;
                    }
                }
                let secret_value = inputs
                    .get("secret_value")
                    .and_then(Value::as_secret)
                    .ok_or_else(|| ExecError::new("missing secret_value (Secret)"))?;

                let url = format!(
                    "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}:addVersion",
                    project, secret
                );
                #[allow(clippy::disallowed_methods)] // Approved: transport boundary — secret sent to GCP Secret Manager
                let encoded_secret = base64_encode(secret_value.expose_plaintext_for_transport());
                let body = serde_json::json!({
                    "payload": { "data": encoded_secret }
                });
                let req = RestRequest::post(url)
                    .header("Authorization", format!("Bearer {}", access_token))
                    .json(body);

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseSecretAddVersion => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().str("version", "").ok(),
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };
                if !(200..=299).contains(&rest.status) {
                    return Err(ExecError::new(format!(
                        "secret addVersion failed (status {})",
                        rest.status
                    )));
                }
                let name = rest.body.get("name").and_then(|v| v.as_str()).unwrap_or("");
                OutputMap::new().str("version", name).ok()
            }
            GcpOps::BuildCredential => {
                let secret = require_str(&inputs, "secret")?;
                let scheme = require_str(&inputs, "scheme")?;
                let _required_scopes =
                    optional_str_list_strict(&inputs, "required_scopes")?.unwrap_or_default();
                let header_name =
                    match inputs.get("header_name") {
                        Some(value) => Some(value.as_str().ok_or_else(|| {
                            ExecError::new("missing or invalid 'header_name' input")
                        })?),
                        None => None,
                    };
                let source_id = require_str(&inputs, "source_id")?;
                let expires_at = None;

                let scheme = match scheme {
                    "bearer" => AuthScheme::Bearer,
                    "header" => {
                        let name = header_name.ok_or_else(|| {
                            ExecError::new("scheme 'header' requires header_name")
                        })?;
                        AuthScheme::Header {
                            name: name.to_string(),
                        }
                    }
                    other => {
                        return Err(ExecError::new(format!(
                            "unknown scheme '{}' (expected bearer|header)",
                            other
                        )));
                    }
                };

                let secret = Secret::new(
                    secret,
                    SecretSource::Exchange {
                        provider: source_id.to_string(),
                    },
                    expires_at,
                );
                let cred = Credential::new(secret, scheme);
                OutputMap::new().value("credential", cred.into()).ok()
            }
            GcpOps::ShouldImpersonate => {
                let allow_impersonation =
                    optional_bool_strict(&inputs, "allow_impersonation")?.unwrap_or(true);
                let has_service_account = inputs
                    .get("service_account")
                    .and_then(Value::as_str)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                let should = allow_impersonation && has_service_account;
                OutputMap::new().bool("should", should).ok()
            }
            GcpOps::ComposeSecretName => {
                let prefix = require_str(&inputs, "prefix")?;
                let service = require_str(&inputs, "service")?;
                let delimiter = inputs
                    .get("delimiter")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                OutputMap::new()
                    .str("secret", format!("{prefix}{delimiter}{service}"))
                    .ok()
            }
            GcpOps::ParseTryRefresh => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new()
                            .bool("needs_reauth", true)
                            .value("access_token", Value::Unit)
                            .value("expires_in", Value::Unit)
                            .ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };
                if !rest.is_success() {
                    let error_desc = rest
                        .body
                        .get("error_description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let error_code = rest
                        .body
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    // Check if this is a recoverable auth error that gcloud re-auth can fix.
                    if is_reauth_error(error_code, error_desc) {
                        return OutputMap::new()
                            .bool("needs_reauth", true)
                            .value("access_token", Value::Unit)
                            .value("expires_in", Value::Unit)
                            .ok();
                    }

                    // Non-auth error — fail immediately (e.g., quota, network).
                    return Err(ExecError::new(format!(
                        "OAuth2 token refresh failed (status {}): {}",
                        rest.status, error_desc
                    )));
                }
                let access_token = rest
                    .body
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ExecError::new("missing access_token in OAuth2 refresh response")
                    })?;
                let expires_in = rest
                    .body
                    .get("expires_in")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(3599);
                OutputMap::new()
                    .bool("needs_reauth", false)
                    .str("access_token", access_token)
                    .int("expires_in", expires_in)
                    .ok()
            }
            GcpOps::PrepareGcloudAuth => {
                let needs_reauth = match inputs.get("needs_reauth") {
                    Some(Value::Bool(b)) => *b,
                    Some(Value::Skipped) => false,
                    _ => false,
                };
                if !needs_reauth {
                    let placeholder = ShellRequest::new("true");
                    return OutputMap::new()
                        .request("request", placeholder.into())
                        .bool("skip", true)
                        .ok();
                }
                let req = GcloudCli.login_update_adc(GcloudLoginOptions::from_env());
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseGcloudAuth => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new().bool("ok", true).ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let shell = match response {
                    TransportResponse::Shell(s) => s,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected Shell response, got {:?}",
                            other
                        )));
                    }
                };
                if shell.exit_code != 0 {
                    let stderr = shell.stderr.trim();
                    let stdout = shell.stdout.trim();
                    let detail = if !stderr.is_empty() {
                        stderr.to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        "no captured output (interactive output may have streamed to terminal)"
                            .to_string()
                    };
                    return Err(ExecError::new(format!(
                        "gcloud auth login failed (exit {}): {}",
                        shell.exit_code, detail
                    )));
                }
                OutputMap::new().bool("ok", true).ok()
            }
            GcpOps::PrepareEnsureIamBinding => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let service_account = require_str(&inputs, "service_account")?;

                if service_account.is_empty() || project.is_empty() {
                    return OutputMap::new()
                        .value("request", Value::Skipped)
                        .bool("skip", true)
                        .str("service_account", service_account)
                        .str("project", project)
                        .ok();
                }

                // Build getIamPolicy REST request using user's access token.
                let cred = Credential::new(Secret::static_value(access_token), AuthScheme::Bearer);
                let svc = ResourceManagerRest::new(cred);
                let req = svc.get_iam_policy(project);

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .str("service_account", service_account)
                    .str("project", project)
                    .ok()
            }
            GcpOps::CheckAndPrepareIamBinding => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new()
                            .value("request", Value::Skipped)
                            .bool("skip", true)
                            .ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };

                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let service_account = require_str(&inputs, "service_account")?;
                let role = "roles/secretmanager.secretAccessor";
                let member = format!("serviceAccount:{}", service_account);

                // If getIamPolicy failed (e.g. 403), tolerate it and skip
                // the set step — the SA may already have the role.
                if !rest.is_success() {
                    return OutputMap::new()
                        .value("request", Value::Skipped)
                        .bool("skip", true)
                        .ok();
                }

                // Extract the policy, either direct or from an envelope.
                let mut policy = match crate::services::IamPolicy::extract(&rest.body) {
                    Some(p) => p,
                    None => crate::services::IamPolicy {
                        bindings: vec![],
                        etag: None,
                        version: None,
                    },
                };

                let changed = policy.ensure_member(role, &member);

                if !changed {
                    return OutputMap::new()
                        .value("request", Value::Skipped)
                        .bool("skip", true)
                        .ok();
                }

                // Build setIamPolicy REST request.
                let cred = Credential::new(Secret::static_value(access_token), AuthScheme::Bearer);
                let svc = ResourceManagerRest::new(cred);
                let req = svc.set_iam_policy(
                    project,
                    serde_json::to_value(policy).unwrap_or(serde_json::json!({})),
                );

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseSetIamBinding => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new().bool("ok", true).ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };

                // Tolerate permission errors gracefully.
                if !rest.is_success() {
                    let details = impersonation_error_summary(&rest.body);
                    if details.contains("PERMISSION_DENIED")
                        || details.contains("403")
                        || details.contains("does not have")
                    {
                        return OutputMap::new().bool("ok", true).ok();
                    }
                    return Err(ExecError::new(format!(
                        "setIamPolicy failed (status {}): {}",
                        rest.status, details
                    )));
                }
                OutputMap::new().bool("ok", true).ok()
            }
            GcpOps::PrepareEnsureSaIamBinding => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let service_account = require_str(&inputs, "service_account")?;
                let member = require_str(&inputs, "member")?;

                if project.is_empty() || service_account.is_empty() || member.is_empty() {
                    return OutputMap::new()
                        .value("request", Value::Skipped)
                        .bool("skip", true)
                        .str("project", project)
                        .str("service_account", service_account)
                        .str("member", member)
                        .ok();
                }

                let cred = Credential::new(Secret::static_value(access_token), AuthScheme::Bearer);
                let svc = IamRest::new(cred);
                let req = svc.get_service_account_iam_policy(project, service_account);

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .str("project", project)
                    .str("service_account", service_account)
                    .str("member", member)
                    .ok()
            }
            GcpOps::CheckAndPrepareSaIamBinding => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new()
                            .value("request", Value::Skipped)
                            .bool("skip", true)
                            .ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };

                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let service_account = require_str(&inputs, "service_account")?;
                let member = require_str(&inputs, "member")?;
                let role = "roles/iam.workloadIdentityUser";

                if !rest.is_success() {
                    return OutputMap::new()
                        .value("request", Value::Skipped)
                        .bool("skip", true)
                        .ok();
                }

                // Extract policy, handling both direct and envelope formats.
                let mut policy = match crate::services::IamPolicy::extract(&rest.body) {
                    Some(p) => p,
                    None => crate::services::IamPolicy {
                        bindings: vec![],
                        etag: None,
                        version: None,
                    },
                };

                let changed = policy.ensure_member(role, member);

                if !changed {
                    return OutputMap::new()
                        .value("request", Value::Skipped)
                        .bool("skip", true)
                        .ok();
                }

                let cred = Credential::new(Secret::static_value(access_token), AuthScheme::Bearer);
                let svc = IamRest::new(cred);
                let req = svc.set_service_account_iam_policy(
                    project,
                    service_account,
                    serde_json::to_value(policy).unwrap_or(serde_json::json!({})),
                );

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseSetSaIamBinding => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => {
                        return OutputMap::new().bool("ok", true).ok();
                    }
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let rest = match response {
                    TransportResponse::Rest(r) => r,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected REST response, got {:?}",
                            other
                        )));
                    }
                };

                if !rest.is_success() {
                    let details = impersonation_error_summary(&rest.body);
                    if details.contains("PERMISSION_DENIED")
                        || details.contains("403")
                        || details.contains("does not have")
                    {
                        return OutputMap::new().bool("ok", true).ok();
                    }
                    return Err(ExecError::new(format!(
                        "setServiceAccountIamPolicy failed (status {}): {}",
                        rest.status, details
                    )));
                }
                OutputMap::new().bool("ok", true).ok()
            }
            GcpOps::MergeAuthResult => {
                // Try the "try" path first (direct refresh succeeded).
                if let Some(token) = inputs.get("try_access_token") {
                    if let Some(s) = token.as_str() {
                        if !s.is_empty() {
                            let expires_in = inputs
                                .get("try_expires_in")
                                .and_then(|v| v.as_int())
                                .unwrap_or(3599);
                            return OutputMap::new()
                                .str("access_token", s)
                                .int("expires_in", expires_in)
                                .ok();
                        }
                    }
                }
                // Fall back to the "retry" path (after gcloud re-auth).
                if let Some(token) = inputs.get("retry_access_token") {
                    if let Some(s) = token.as_str() {
                        if !s.is_empty() {
                            let expires_in = inputs
                                .get("retry_expires_in")
                                .and_then(|v| v.as_int())
                                .unwrap_or(3599);
                            return OutputMap::new()
                                .str("access_token", s)
                                .int("expires_in", expires_in)
                                .ok();
                        }
                    }
                }
                Err(ExecError::new(
                    "no valid access token from either refresh path — run `make login` to re-authenticate",
                ))
            }
        }
    }
}

/// Auth error patterns that indicate a recoverable token issue.
///
/// When matched, the DAG falls back to `gcloud auth login --update-adc`
/// instead of failing. Based on gunb.ai's `isPermanentAuthFailure` +
/// Google's OAuth2 error codes.
fn is_reauth_error(error_code: &str, error_description: &str) -> bool {
    let combined = format!("{} {}", error_code, error_description).to_lowercase();
    const PATTERNS: &[&str] = &[
        "invalid_rapt",
        "invalid_grant",
        "expired or revoked",
        "unauthenticated",
        "reauth",
        "invalid_client",
    ];
    PATTERNS.iter().any(|p| combined.contains(p))
}

// ---------------------------------------------------------------------------
// Small helpers (url-encode and base64 decode) to avoid extra deps.
// ---------------------------------------------------------------------------

/// Default path to the Google Cloud Application Default Credentials file.
///
/// Typically `~/.config/gcloud/application_default_credentials.json`.
pub(crate) fn adc_file_path() -> String {
    if let Ok(path) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        return path;
    }
    let home = std::env::var("HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".config/gcloud/application_default_credentials.json")
        .to_string_lossy()
        .into_owned()
}

#[allow(clippy::disallowed_methods)] // Approved: transport boundary — secret extracted for service request
fn optional_secret_or_str(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Result<Option<String>, ExecError> {
    match inputs.get(key) {
        None | Some(Value::Skipped) => Ok(None),
        Some(Value::Str(value)) => Ok(Some(value.clone())),
        Some(Value::Secret(value)) => Ok(Some(value.expose_plaintext_for_transport().to_string())),
        Some(_) => Err(ExecError::new(format!(
            "invalid '{}' input: expected String or Secret",
            key
        ))),
    }
}

fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut sextets: Vec<u8> = Vec::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z' => sextets.push(b - b'A'),
            b'a'..=b'z' => sextets.push(b - b'a' + 26),
            b'0'..=b'9' => sextets.push(b - b'0' + 52),
            b'+' => sextets.push(62),
            b'/' => sextets.push(63),
            b'=' => sextets.push(64),
            b' ' | b'\n' | b'\r' | b'\t' => {}
            other => {
                return Err(format!("invalid base64 char 0x{other:02x}"));
            }
        }
    }

    if sextets.len() & 3 != 0 {
        return Err("invalid base64 length".to_string());
    }

    let total_chunks = sextets.len() / 4;
    let mut out = Vec::with_capacity(total_chunks * 3);
    for (idx, chunk) in sextets.chunks(4).enumerate() {
        let v0 = chunk[0];
        let v1 = chunk[1];
        let v2 = chunk[2];
        let v3 = chunk[3];

        if v0 == 64 || v1 == 64 {
            return Err("invalid base64 padding".to_string());
        }
        if v2 == 64 && v3 != 64 {
            return Err("invalid base64 padding".to_string());
        }

        let pad = if v2 == 64 {
            2
        } else if v3 == 64 {
            1
        } else {
            0
        };
        if pad > 0 && idx != total_chunks.saturating_sub(1) {
            return Err("invalid base64 padding".to_string());
        }

        let b0 = (v0 << 2) | (v1 >> 4);
        out.push(b0);

        if v2 != 64 {
            let b1 = ((v1 & 0x0f) << 4) | (v2 >> 2);
            out.push(b1);
        }
        if v3 != 64 {
            let b2 = ((v2 & 0x03) << 6) | v3;
            out.push(b2);
        }
    }

    Ok(out)
}

fn impersonation_error_summary(body: &serde_json::Value) -> String {
    if let Some(err) = body.get("error") {
        if let Some(message) = err.get("message").and_then(|v| v.as_str()) {
            return message.to_string();
        }
        if let Some(status) = err.get("status").and_then(|v| v.as_str()) {
            return status.to_string();
        }
        if let Some(code) = err.get("code").and_then(|v| v.as_i64()) {
            return format!("error code {code}");
        }
    }

    if let Some(message) = body.get("message").and_then(|v| v.as_str()) {
        return message.to_string();
    }
    if let Some(desc) = body.get("error_description").and_then(|v| v.as_str()) {
        return desc.to_string();
    }
    if let Some(status) = body.get("status").and_then(|v| v.as_str()) {
        return status.to_string();
    }

    let rendered = body.to_string();
    if rendered.len() > 240 {
        format!("{}...", &rendered[..240])
    } else {
        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::file::FileResponse;
    use gunbc_ir::transport::rest::RestResponse;

    // ==========================================================================
    // ADC + OAuth2 ops tests
    // ==========================================================================

    #[test]
    fn prepare_check_adc_produces_file_exists_request() {
        let inputs = HashMap::new();
        let outputs = GcpOps::PrepareCheckAdc
            .execute(inputs)
            .expect("should succeed");
        let req = outputs.get("request").expect("should have request");
        // The request should be a File transport request
        match req {
            Value::Request(gunbc_ir::transport::TransportRequest::File(f)) => {
                assert!(f.path.contains("application_default_credentials.json"));
                assert_eq!(f.operation, gunbc_ir::transport::file::FileOp::Exists);
            }
            other => panic!("expected File request, got {:?}", other),
        }
    }

    #[test]
    fn parse_check_adc_returns_false_when_missing() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse::exists_result(
                "/fake/path",
                false,
            ))),
        );
        let outputs = GcpOps::ParseCheckAdc
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("exists"), Some(&Value::Bool(false)));
    }

    #[test]
    fn parse_check_adc_succeeds_when_file_exists() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse::exists_result(
                "/fake/path",
                true,
            ))),
        );
        let outputs = GcpOps::ParseCheckAdc
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("exists"), Some(&Value::Bool(true)));
    }

    #[test]
    fn prepare_read_adc_produces_file_read_request() {
        let mut inputs = HashMap::new();
        inputs.insert("exists".to_string(), Value::Bool(true));
        let outputs = GcpOps::PrepareReadAdc
            .execute(inputs)
            .expect("should succeed");
        match outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::File(f))) => {
                assert!(f.path.contains("application_default_credentials.json"));
                assert_eq!(f.operation, gunbc_ir::transport::file::FileOp::Read);
            }
            other => panic!("expected File request, got {:?}", other),
        }
    }

    #[test]
    fn prepare_read_adc_fails_when_not_exists() {
        let mut inputs = HashMap::new();
        inputs.insert("exists".to_string(), Value::Bool(false));
        let err = GcpOps::PrepareReadAdc
            .execute(inputs)
            .expect_err("missing ADC should fail early");
        assert!(
            err.to_string()
                .contains("gcloud auth application-default login"),
            "error should include remediation command, got: {}",
            err
        );
    }

    #[test]
    fn parse_adc_credentials_extracts_tokens() {
        let adc_json = serde_json::json!({
            "client_id": "my-client-id.apps.googleusercontent.com",
            "client_secret": "my-secret",
            "refresh_token": "1//refresh-token",
            "type": "authorized_user"
        });
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse::read_ok(
                "/fake/path",
                adc_json.to_string(),
            ))),
        );
        let outputs = GcpOps::ParseAdcCredentials
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(
            outputs.get("client_id").and_then(|v| v.as_str()),
            Some("my-client-id.apps.googleusercontent.com")
        );
        assert_eq!(
            outputs.get("refresh_token").and_then(|v| v.as_str()),
            Some("1//refresh-token")
        );
        assert_eq!(
            outputs.get("token_type").and_then(|v| v.as_str()),
            Some("authorized_user")
        );
    }

    #[test]
    fn parse_adc_credentials_fails_on_missing_refresh_token() {
        let adc_json = serde_json::json!({
            "client_id": "id",
            "client_secret": "secret",
            "type": "authorized_user"
        });
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse::read_ok(
                "/fake/path",
                adc_json.to_string(),
            ))),
        );
        let err = GcpOps::ParseAdcCredentials
            .execute(inputs)
            .expect_err("should fail on missing refresh_token");
        assert!(err.to_string().contains("refresh_token"));
    }

    #[test]
    fn prepare_oauth2_refresh_builds_rest_request() {
        let mut inputs = HashMap::new();
        inputs.insert("client_id".to_string(), Value::Str("id".to_string()));
        inputs.insert(
            "client_secret".to_string(),
            Value::Str("secret".to_string()),
        );
        inputs.insert("refresh_token".to_string(), Value::Str("token".to_string()));
        let outputs = GcpOps::PrepareOAuth2Refresh
            .execute(inputs)
            .expect("should succeed");
        match outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(r))) => {
                assert!(r.url.contains("oauth2.googleapis.com/token"));
                assert_eq!(r.method, gunbc_ir::transport::http::HttpMethod::Post);
                let body = r.body.as_ref().expect("should have body");
                assert_eq!(body["grant_type"], "refresh_token");
            }
            other => panic!("expected REST request, got {:?}", other),
        }
    }

    #[test]
    fn parse_oauth2_refresh_extracts_token() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(
                serde_json::json!({
                    "access_token": "ya29.a-token",
                    "expires_in": 3600,
                    "token_type": "Bearer"
                }),
            ))),
        );
        let outputs = GcpOps::ParseOAuth2Refresh
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(
            outputs.get("access_token").and_then(|v| v.as_str()),
            Some("ya29.a-token")
        );
        assert_eq!(
            outputs.get("expires_in").and_then(|v| v.as_int()),
            Some(3600)
        );
    }

    #[test]
    fn parse_oauth2_refresh_fails_on_error_response() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::new(
                401,
                serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": "Token has been expired or revoked."
                }),
            ))),
        );
        let err = GcpOps::ParseOAuth2Refresh
            .execute(inputs)
            .expect_err("should fail on error response");
        assert!(err.to_string().contains("expired or revoked"));
    }

    #[test]
    fn parse_impersonate_reports_status_and_error_message() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::new(
                403,
                serde_json::json!({
                    "error": {
                        "code": 403,
                        "message": "Permission iam.serviceAccounts.getAccessToken denied",
                        "status": "PERMISSION_DENIED"
                    }
                }),
            ))),
        );

        let err = GcpOps::ParseImpersonate
            .execute(inputs)
            .expect_err("impersonation failure should bubble up");
        let msg = err.to_string();
        assert!(msg.contains("status 403"), "msg: {msg}");
        assert!(
            msg.contains("Permission iam.serviceAccounts.getAccessToken denied"),
            "msg: {msg}"
        );
    }

    #[test]
    fn parse_impersonate_accepts_access_token_snake_case() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(
                serde_json::json!({
                    "access_token": "ya29.impersonated"
                }),
            ))),
        );

        let outputs = GcpOps::ParseImpersonate
            .execute(inputs)
            .expect("snake_case access token should parse");
        assert_eq!(
            outputs.get("access_token").and_then(|v| v.as_str()),
            Some("ya29.impersonated")
        );
        assert_eq!(outputs.get("expires_at").and_then(|v| v.as_str()), Some(""));
    }

    #[test]
    fn parse_impersonate_surfaces_expire_time_output() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(
                serde_json::json!({
                    "accessToken": "ya29.impersonated",
                    "expireTime": "2025-01-01T00:00:00Z"
                }),
            ))),
        );

        let outputs = GcpOps::ParseImpersonate
            .execute(inputs)
            .expect("impersonation response should parse");
        assert_eq!(
            outputs.get("access_token").and_then(|v| v.as_str()),
            Some("ya29.impersonated")
        );
        assert_eq!(
            outputs.get("expires_at").and_then(|v| v.as_str()),
            Some("2025-01-01T00:00:00Z")
        );
    }

    #[test]
    fn should_impersonate_requires_service_account_by_default() {
        let mut inputs = HashMap::new();
        inputs.insert("service_account".to_string(), Value::Str(String::new()));

        let outputs = GcpOps::ShouldImpersonate
            .execute(inputs)
            .expect("should_impersonate should evaluate");
        assert_eq!(outputs.get("should"), Some(&Value::Bool(false)));
    }

    #[test]
    fn should_impersonate_respects_allow_impersonation_flag() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "service_account".to_string(),
            Value::Str("svc@p.iam.gserviceaccount.com".to_string()),
        );
        inputs.insert("allow_impersonation".to_string(), Value::Bool(false));

        let outputs = GcpOps::ShouldImpersonate
            .execute(inputs)
            .expect("should_impersonate should evaluate");
        assert_eq!(outputs.get("should"), Some(&Value::Bool(false)));
    }

    #[test]
    fn prepare_impersonate_skips_when_service_account_empty() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "access_token".to_string(),
            Value::Str("base-token".to_string()),
        );
        inputs.insert("service_account".to_string(), Value::Str(String::new()));

        let outputs = GcpOps::PrepareImpersonate
            .execute(inputs)
            .expect("empty service account should skip impersonation");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_impersonate_uses_base_token_when_skipped() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Skipped);
        inputs.insert(
            "base_access_token".to_string(),
            Value::Str("base-token".to_string()),
        );

        let outputs = GcpOps::ParseImpersonate
            .execute(inputs)
            .expect("skipped impersonation should return base token");
        assert_eq!(
            outputs.get("access_token").and_then(|v| v.as_str()),
            Some("base-token")
        );
        assert_eq!(outputs.get("expires_at").and_then(|v| v.as_str()), Some(""));
    }

    #[test]
    fn build_credential_accepts_required_scopes_list() {
        let mut inputs = HashMap::new();
        inputs.insert("secret".to_string(), Value::Str("tok".to_string()));
        inputs.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        inputs.insert("source_id".to_string(), Value::Str("openai".to_string()));
        inputs.insert(
            "required_scopes".to_string(),
            Value::str_list(vec!["llm:chat_completion".to_string()]),
        );

        let outputs = GcpOps::BuildCredential
            .execute(inputs)
            .expect("valid required_scopes list should be accepted");
        assert!(
            outputs.contains_key("credential"),
            "build_credential should return a credential"
        );
    }

    #[test]
    fn build_credential_rejects_wrong_required_scopes_type() {
        let mut inputs = HashMap::new();
        inputs.insert("secret".to_string(), Value::Str("tok".to_string()));
        inputs.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        inputs.insert("source_id".to_string(), Value::Str("openai".to_string()));
        inputs.insert("required_scopes".to_string(), Value::Int(1));

        let err = GcpOps::BuildCredential
            .execute(inputs)
            .expect_err("wrong required_scopes type should fail");
        assert!(
            err.to_string().contains("required_scopes"),
            "error should mention required_scopes, got: {}",
            err
        );
    }

    // ==========================================================================
    // ParseTryRefresh tests
    // ==========================================================================

    #[test]
    fn parse_try_refresh_succeeds_on_valid_response() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(
                serde_json::json!({
                    "access_token": "ya29.fresh",
                    "expires_in": 3600,
                }),
            ))),
        );
        let outputs = GcpOps::ParseTryRefresh
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("needs_reauth"), Some(&Value::Bool(false)));
        assert_eq!(
            outputs.get("access_token").and_then(|v| v.as_str()),
            Some("ya29.fresh")
        );
        assert_eq!(
            outputs.get("expires_in").and_then(|v| v.as_int()),
            Some(3600)
        );
    }

    #[test]
    fn parse_try_refresh_flags_reauth_on_invalid_rapt() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::new(
                400,
                serde_json::json!({
                    "error": "invalid_rapt",
                    "error_description": "reauth related error (invalid_rapt)"
                }),
            ))),
        );
        let outputs = GcpOps::ParseTryRefresh
            .execute(inputs)
            .expect("should not fail on recoverable auth error");
        assert_eq!(outputs.get("needs_reauth"), Some(&Value::Bool(true)));
        assert_eq!(outputs.get("access_token"), Some(&Value::Unit));
    }

    #[test]
    fn parse_try_refresh_flags_reauth_on_invalid_grant() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::new(
                401,
                serde_json::json!({
                    "error": "invalid_grant",
                    "error_description": "Token has been expired or revoked."
                }),
            ))),
        );
        let outputs = GcpOps::ParseTryRefresh
            .execute(inputs)
            .expect("should not fail on expired token");
        assert_eq!(outputs.get("needs_reauth"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_try_refresh_fails_on_non_auth_error() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::new(
                429,
                serde_json::json!({
                    "error": "rate_limit_exceeded",
                    "error_description": "too many requests"
                }),
            ))),
        );
        let err = GcpOps::ParseTryRefresh
            .execute(inputs)
            .expect_err("non-auth error should fail");
        assert!(
            err.to_string().contains("too many requests"),
            "msg: {}",
            err
        );
    }

    #[test]
    fn prepare_gcloud_auth_builds_shell_request_when_needed() {
        let mut inputs = HashMap::new();
        inputs.insert("needs_reauth".to_string(), Value::Bool(true));
        let outputs = GcpOps::PrepareGcloudAuth
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(false)));
        match outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Shell(s))) => {
                assert_eq!(s.command, "gcloud");
                assert!(s.args.len() >= 3);
                assert_eq!(s.args[0], "auth");
                assert_eq!(s.args[1], "login");
                assert_eq!(s.args[2], "--update-adc");
                assert!(s.passthrough);
            }
            other => panic!("expected Shell request, got {:?}", other),
        }
    }

    #[test]
    fn prepare_gcloud_auth_skips_when_not_needed() {
        let mut inputs = HashMap::new();
        inputs.insert("needs_reauth".to_string(), Value::Bool(false));
        let outputs = GcpOps::PrepareGcloudAuth
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_gcloud_auth_succeeds_on_zero_exit() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(
                gunbc_ir::transport::ShellResponse {
                    exit_code: 0,
                    stdout: "You are now logged in.".to_string(),
                    stderr: String::new(),
                },
            )),
        );
        let outputs = GcpOps::ParseGcloudAuth
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("ok"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_gcloud_auth_fails_on_nonzero_exit() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(
                gunbc_ir::transport::ShellResponse {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: "ERROR: gcloud crashed".to_string(),
                },
            )),
        );
        let err = GcpOps::ParseGcloudAuth
            .execute(inputs)
            .expect_err("nonzero exit should fail");
        assert!(err.to_string().contains("gcloud auth login failed"));
        assert!(err.to_string().contains("gcloud crashed"));
    }

    #[test]
    fn parse_gcloud_auth_nonzero_without_output_has_interactive_hint() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(
                gunbc_ir::transport::ShellResponse {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: String::new(),
                },
            )),
        );
        let err = GcpOps::ParseGcloudAuth
            .execute(inputs)
            .expect_err("nonzero exit should fail");
        assert!(err
            .to_string()
            .contains("interactive output may have streamed"));
    }

    #[test]
    fn merge_auth_result_prefers_try_path() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "try_access_token".to_string(),
            Value::Str("ya29.try".to_string()),
        );
        inputs.insert("try_expires_in".to_string(), Value::Int(3600));
        inputs.insert("retry_access_token".to_string(), Value::Unit);
        inputs.insert("retry_expires_in".to_string(), Value::Unit);
        let outputs = GcpOps::MergeAuthResult
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(
            outputs.get("access_token").and_then(|v| v.as_str()),
            Some("ya29.try")
        );
    }

    #[test]
    fn merge_auth_result_falls_back_to_retry_path() {
        let mut inputs = HashMap::new();
        inputs.insert("try_access_token".to_string(), Value::Unit);
        inputs.insert("try_expires_in".to_string(), Value::Unit);
        inputs.insert(
            "retry_access_token".to_string(),
            Value::Str("ya29.retry".to_string()),
        );
        inputs.insert("retry_expires_in".to_string(), Value::Int(3600));
        let outputs = GcpOps::MergeAuthResult
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(
            outputs.get("access_token").and_then(|v| v.as_str()),
            Some("ya29.retry")
        );
    }

    #[test]
    fn merge_auth_result_fails_when_both_empty() {
        let mut inputs = HashMap::new();
        inputs.insert("try_access_token".to_string(), Value::Unit);
        inputs.insert("try_expires_in".to_string(), Value::Unit);
        inputs.insert("retry_access_token".to_string(), Value::Unit);
        inputs.insert("retry_expires_in".to_string(), Value::Unit);
        let err = GcpOps::MergeAuthResult
            .execute(inputs)
            .expect_err("both empty should fail");
        assert!(err.to_string().contains("make login"));
    }

    // ==========================================================================
    // PrepareEnsureIamBinding / ParseEnsureIamBinding tests
    // ==========================================================================

    #[test]
    fn prepare_ensure_iam_builds_rest_request() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert(
            "project".to_string(),
            Value::Str("gunbai-secrets".to_string()),
        );
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@gunbai-secrets.iam.gserviceaccount.com".to_string()),
        );
        let outputs = GcpOps::PrepareEnsureIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(false)));
        match outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(r))) => {
                assert!(r.url.contains(":getIamPolicy"));
            }
            other => panic!("expected REST request, got {:?}", other),
        }
        // service_account and project pass through
        assert_eq!(
            outputs.get("service_account"),
            Some(&Value::Str(
                "sa@gunbai-secrets.iam.gserviceaccount.com".to_string()
            ))
        );
    }

    #[test]
    fn prepare_ensure_iam_skips_when_sa_empty() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert(
            "project".to_string(),
            Value::Str("gunbai-secrets".to_string()),
        );
        inputs.insert("service_account".to_string(), Value::Str(String::new()));
        let outputs = GcpOps::PrepareEnsureIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn check_iam_binding_skips_when_already_bound() {
        use gunbc_ir::transport::rest::RestResponse;
        let policy = serde_json::json!({
            "bindings": [{
                "role": "roles/secretmanager.secretAccessor",
                "members": ["serviceAccount:sa@project.iam.gserviceaccount.com"]
            }],
            "etag": "abc123"
        });
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(policy))),
        );
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert("project".to_string(), Value::Str("project".to_string()));
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@project.iam.gserviceaccount.com".to_string()),
        );
        let outputs = GcpOps::CheckAndPrepareIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn check_iam_binding_builds_set_request_when_missing() {
        use gunbc_ir::transport::rest::RestResponse;
        let policy = serde_json::json!({
            "bindings": [],
            "etag": "abc123"
        });
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(policy))),
        );
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert("project".to_string(), Value::Str("project".to_string()));
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@project.iam.gserviceaccount.com".to_string()),
        );
        let outputs = GcpOps::CheckAndPrepareIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(false)));
        match outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(r))) => {
                assert!(r.url.contains(":setIamPolicy"));
            }
            other => panic!("expected REST request, got {:?}", other),
        }
    }

    #[test]
    fn check_iam_binding_tolerates_403() {
        use gunbc_ir::transport::rest::RestResponse;
        let error_response = RestResponse::new(
            403,
            serde_json::json!({"error": {"message": "PERMISSION_DENIED"}}),
        );
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(error_response)),
        );
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert("project".to_string(), Value::Str("project".to_string()));
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@project.iam.gserviceaccount.com".to_string()),
        );
        let outputs = GcpOps::CheckAndPrepareIamBinding
            .execute(inputs)
            .expect("should tolerate 403");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_set_iam_binding_succeeds_on_ok() {
        use gunbc_ir::transport::rest::RestResponse;
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(
                serde_json::json!({"bindings": []}),
            ))),
        );
        let outputs = GcpOps::ParseSetIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("ok"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_set_iam_binding_handles_skipped() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Skipped);
        let outputs = GcpOps::ParseSetIamBinding
            .execute(inputs)
            .expect("should handle skipped");
        assert_eq!(outputs.get("ok"), Some(&Value::Bool(true)));
    }

    #[test]
    fn prepare_ensure_sa_iam_builds_rest_request() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert("project".to_string(), Value::Str("project".to_string()));
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@project.iam.gserviceaccount.com".to_string()),
        );
        inputs.insert(
            "member".to_string(),
            Value::Str("principalSet://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc".to_string()),
        );
        let outputs = GcpOps::PrepareEnsureSaIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(false)));
        match outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(r))) => {
                assert!(
                    r.url.contains(":getIamPolicy"),
                    "expected service-account getIamPolicy request"
                );
            }
            other => panic!("expected REST request, got {:?}", other),
        }
    }

    #[test]
    fn prepare_ensure_sa_iam_skips_when_member_empty() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert("project".to_string(), Value::Str("project".to_string()));
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@project.iam.gserviceaccount.com".to_string()),
        );
        inputs.insert("member".to_string(), Value::Str(String::new()));
        let outputs = GcpOps::PrepareEnsureSaIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn check_sa_iam_binding_skips_when_already_bound() {
        use gunbc_ir::transport::rest::RestResponse;
        let policy = serde_json::json!({
            "bindings": [{
                "role": "roles/iam.workloadIdentityUser",
                "members": ["principalSet://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc"]
            }],
            "etag": "abc123"
        });
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(policy))),
        );
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert("project".to_string(), Value::Str("project".to_string()));
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@project.iam.gserviceaccount.com".to_string()),
        );
        inputs.insert(
            "member".to_string(),
            Value::Str("principalSet://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc".to_string()),
        );
        let outputs = GcpOps::CheckAndPrepareSaIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn check_sa_iam_binding_builds_set_request_when_missing() {
        use gunbc_ir::transport::rest::RestResponse;
        let policy = serde_json::json!({
            "bindings": [],
            "etag": "abc123"
        });
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(policy))),
        );
        inputs.insert(
            "access_token".to_string(),
            Value::Str("mock-token".to_string()),
        );
        inputs.insert("project".to_string(), Value::Str("project".to_string()));
        inputs.insert(
            "service_account".to_string(),
            Value::Str("sa@project.iam.gserviceaccount.com".to_string()),
        );
        inputs.insert(
            "member".to_string(),
            Value::Str("principalSet://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc".to_string()),
        );
        let outputs = GcpOps::CheckAndPrepareSaIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(false)));
        match outputs.get("request") {
            Some(Value::Request(gunbc_ir::transport::TransportRequest::Rest(r))) => {
                assert!(
                    r.url.contains(":setIamPolicy"),
                    "expected service-account setIamPolicy request"
                );
                let body = r
                    .body
                    .clone()
                    .expect("setIamPolicy request should include policy body");
                assert!(
                    body.to_string().contains("roles/iam.workloadIdentityUser"),
                    "policy body should include workloadIdentityUser role"
                );
            }
            other => panic!("expected REST request, got {:?}", other),
        }
    }

    #[test]
    fn parse_set_sa_iam_binding_succeeds_on_ok() {
        use gunbc_ir::transport::rest::RestResponse;
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(
                serde_json::json!({"bindings": []}),
            ))),
        );
        let outputs = GcpOps::ParseSetSaIamBinding
            .execute(inputs)
            .expect("should succeed");
        assert_eq!(outputs.get("ok"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parse_set_sa_iam_binding_handles_skipped() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Skipped);
        let outputs = GcpOps::ParseSetSaIamBinding
            .execute(inputs)
            .expect("should handle skipped");
        assert_eq!(outputs.get("ok"), Some(&Value::Bool(true)));
    }
}
