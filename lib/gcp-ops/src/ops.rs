//! Pure GCP ops for WIF + Secret Manager.

use gunbc_exec::{require_bool, require_str, ExecError, Executable, OutputMap};
use gunbc_ir::transport::rest::RestRequest;
use gunbc_ir::transport::{ShellRequest, TransportResponse};
use gunbc_ir::{AuthScheme, Credential, Secret, SecretSource, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Prepare the OIDC token request for GitHub Actions.
    PrepareGitHubOidcRequest,
    /// Parse the OIDC token response from GitHub Actions.
    ParseGitHubOidcResponse,
    /// Prepare the OIDC token request for GCP metadata server.
    PrepareMetadataOidcRequest,
    /// Parse the OIDC token response from metadata server.
    ParseMetadataOidcResponse,
    /// Prepare a local gcloud access token request.
    PrepareLocalAccessToken,
    /// Parse local gcloud access token response.
    ParseLocalAccessToken,
    /// Parse local auth check response (best-effort, non-failing).
    ParseLocalAuthCheck,
    /// Prepare local interactive auth login request.
    PrepareLocalAuthLogin,
    /// Parse local interactive auth login response.
    ParseLocalAuthLogin,
    /// Prepare the STS token exchange request.
    PrepareStsExchange,
    /// Parse the STS token exchange response.
    ParseStsExchange,
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
            GcpOps::PrepareGitHubOidcRequest => {
                let audience = require_str(&inputs, "audience")?;
                let request_url = match inputs.get("request_url") {
                    Some(value) => value
                        .as_str()
                        .ok_or_else(|| ExecError::new("missing or invalid 'request_url' input"))?,
                    None => "",
                };
                let request_token = match inputs.get("request_token") {
                    Some(value) => value.as_str().ok_or_else(|| {
                        ExecError::new("missing or invalid 'request_token' input")
                    })?,
                    None => "",
                };

                let skip = request_url.trim().is_empty() || request_token.trim().is_empty();
                let url = if skip {
                    "https://example.invalid/oidc".to_string()
                } else {
                    format!(
                        "{}?audience={}",
                        request_url,
                        url_encode_component(audience)
                    )
                };
                let req = if skip {
                    RestRequest::get(url)
                } else {
                    RestRequest::get(url)
                        .header("Authorization", format!("Bearer {}", request_token))
                };

                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", skip)
                    .ok()
            }
            GcpOps::ParseGitHubOidcResponse => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().str("subject_token", "").ok(),
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
                let token = rest
                    .body
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExecError::new("missing 'value' in GitHub OIDC response"))?;
                OutputMap::new().str("subject_token", token).ok()
            }
            GcpOps::PrepareMetadataOidcRequest => {
                let audience = require_str(&inputs, "audience")?;
                let url = format!(
                    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity?audience={}&format=full",
                    url_encode_component(audience)
                );
                let req = RestRequest::get(url).header("Metadata-Flavor", "Google");
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseMetadataOidcResponse => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().str("subject_token", "").ok(),
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
                // Metadata server returns raw string; executor wraps it as {\"raw\": \"...\"}.
                if let Some(raw) = rest.body.get("raw").and_then(|v| v.as_str()) {
                    return OutputMap::new().str("subject_token", raw).ok();
                }
                if let Some(raw) = rest.body.as_str() {
                    return OutputMap::new().str("subject_token", raw).ok();
                }
                Err(ExecError::new(
                    "missing raw OIDC token from metadata response",
                ))
            }
            GcpOps::PrepareLocalAccessToken => {
                let req = ShellRequest::new("gcloud")
                    .args(["auth", "print-access-token"])
                    .into_transport_request();
                OutputMap::new()
                    .request("request", req)
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseLocalAccessToken => {
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
                let shell = match response {
                    TransportResponse::Shell(s) => s,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected shell response, got {:?}",
                            other
                        )));
                    }
                };
                if shell.exit_code != 0 {
                    return Err(ExecError::new(format!(
                        "gcloud auth print-access-token failed: {}",
                        shell.stderr.trim()
                    )));
                }
                let token = shell.stdout.trim();
                if token.is_empty() {
                    return Err(ExecError::new(
                        "gcloud auth print-access-token returned empty token",
                    ));
                }
                // gcloud doesn't return TTL in this path; keep a stable contract by emitting 0.
                OutputMap::new()
                    .str("access_token", token)
                    .int("expires_in", 0)
                    .ok()
            }
            GcpOps::ParseLocalAuthCheck => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().bool("exists", false).ok(),
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let shell = match response {
                    TransportResponse::Shell(s) => s,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected shell response, got {:?}",
                            other
                        )));
                    }
                };
                let exists = shell.exit_code == 0 && !shell.stdout.trim().is_empty();
                OutputMap::new().bool("exists", exists).ok()
            }
            GcpOps::PrepareLocalAuthLogin => {
                let exists = require_bool(&inputs, "exists")?;
                let interactive_allowed = inputs
                    .get("interactive_allowed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if !exists && !interactive_allowed {
                    return Err(ExecError::new(
                        "local auth session missing and interactive login is disabled for this action",
                    ));
                }

                let req = ShellRequest::new("gcloud")
                    .args(["auth", "login", "--update-adc"])
                    .into_transport_request();
                OutputMap::new()
                    .request("request", req)
                    .bool("skip", exists)
                    .ok()
            }
            GcpOps::ParseLocalAuthLogin => {
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().bool("ok", true).ok(),
                    Some(Value::Response(r)) => r,
                    _ => return Err(ExecError::new("missing or invalid 'response' input")),
                };
                let shell = match response {
                    TransportResponse::Shell(s) => s,
                    other => {
                        return Err(ExecError::new(format!(
                            "expected shell response, got {:?}",
                            other
                        )));
                    }
                };
                if shell.exit_code != 0 {
                    return Err(ExecError::new(format!(
                        "gcloud auth login failed: {}",
                        shell.stderr.trim()
                    )));
                }
                OutputMap::new().bool("ok", true).ok()
            }
            GcpOps::PrepareStsExchange => {
                let audience = require_str(&inputs, "audience")?;
                let subject_token = require_str(&inputs, "subject_token")?;

                let body = serde_json::json!({
                    "audience": audience,
                    "grant_type": "urn:ietf:params:oauth:grant-type:token-exchange",
                    "requested_token_type": "urn:ietf:params:oauth:token-type:access_token",
                    "subject_token_type": "urn:ietf:params:oauth:token-type:jwt",
                    "subject_token": subject_token,
                });

                let req = RestRequest::post("https://sts.googleapis.com/v1/token").json(body);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpOps::ParseStsExchange => {
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
                let access_token = rest
                    .body
                    .get("access_token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExecError::new("missing access_token in STS response"))?;
                let expires_in = rest
                    .body
                    .get("expires_in")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                OutputMap::new()
                    .str("access_token", access_token)
                    .int("expires_in", expires_in)
                    .ok()
            }
            GcpOps::PrepareImpersonate => {
                let access_token = require_str(&inputs, "access_token")?;
                let service_account = require_str(&inputs, "service_account")?;
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
                let response = match inputs.get("response") {
                    Some(Value::Skipped) => return OutputMap::new().str("access_token", "").ok(),
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
                let token = rest
                    .body
                    .get("accessToken")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ExecError::new("missing accessToken in impersonation response")
                    })?;
                let _ = rest
                    .body
                    .get("expireTime")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                OutputMap::new().str("access_token", token).ok()
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
                let body = serde_json::json!({
                    "payload": { "data": base64_encode(secret_value.expose()) }
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
                let service_account = require_str(&inputs, "service_account")?;
                let should = !service_account.trim().is_empty();
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
        }
    }
}

// ---------------------------------------------------------------------------
// Small helpers (url-encode and base64 decode) to avoid extra deps.
// ---------------------------------------------------------------------------

fn url_encode_component(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if is_unreserved_url_byte(b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0f) as usize] as char);
        }
    }
    out
}

fn is_unreserved_url_byte(b: u8) -> bool {
    matches!(
        b,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
    )
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

    if !sextets.len().is_multiple_of(4) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::ShellResponse;

    #[test]
    fn parse_local_auth_check_treats_failed_token_probe_as_missing_auth() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Shell(ShellResponse::failed(
                1,
                "not logged in",
            ))),
        );

        let outputs = GcpOps::ParseLocalAuthCheck
            .execute(inputs)
            .expect("check parser should not fail on unauthenticated sessions");

        assert_eq!(outputs.get("exists"), Some(&Value::Bool(false)));
    }

    #[test]
    fn prepare_local_auth_login_requires_interactive_when_auth_missing() {
        let mut inputs = HashMap::new();
        inputs.insert("exists".to_string(), Value::Bool(false));

        let err = GcpOps::PrepareLocalAuthLogin
            .execute(inputs)
            .expect_err("missing auth with interactive disabled should fail");
        assert!(
            err.to_string().contains("interactive"),
            "error should explain why local login did not run"
        );
    }

    #[test]
    fn prepare_local_auth_login_skips_when_auth_exists() {
        let mut inputs = HashMap::new();
        inputs.insert("exists".to_string(), Value::Bool(true));

        let outputs = GcpOps::PrepareLocalAuthLogin
            .execute(inputs)
            .expect("prepare local auth login should succeed");
        assert_eq!(outputs.get("skip"), Some(&Value::Bool(true)));
    }
}
