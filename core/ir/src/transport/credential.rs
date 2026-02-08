//! Credential lifecycle types.
//!
//! Provides a unified credential model that separates:
//! - **Source**: Where the secret comes from (`SecretSource`)
//! - **Value**: The token itself (`Secret`)
//! - **Scheme**: How it attaches to HTTP (`AuthScheme`)
//!
//! These compose into a [`Credential`] that can apply itself to a
//! [`RestRequest`] and participate in the DAG as a [`Resource`].

use crate::resource::{
    capability_marker, ensure_capability_marker, AccessMode, Resource, ResourceKind,
};
use crate::transport::rest::RestRequest;
use crate::value::{SecretString, Value};
use gunbc_infra::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// SecretSource
// ---------------------------------------------------------------------------

/// Where a secret was acquired — audit trail only, never contains the value.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretSource {
    /// Read from an environment variable at runtime.
    EnvVar(String),
    /// Obtained via a token exchange (e.g., OIDC, OAuth).
    Exchange { provider: String },
    /// Hardcoded / injected directly (tests, CLI flags).
    Static,
}

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretSource::EnvVar(var) => write!(f, "SecretSource::EnvVar({var:?})"),
            SecretSource::Exchange { provider } => {
                write!(f, "SecretSource::Exchange {{ provider: {provider:?} }}")
            }
            SecretSource::Static => write!(f, "SecretSource::Static"),
        }
    }
}

// ---------------------------------------------------------------------------
// Secret
// ---------------------------------------------------------------------------

/// A secret value with optional expiry and provenance tracking.
///
/// The inner value is private — use [`expose()`](Secret::expose) to access it.
/// Debug and Display both redact the value.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Secret {
    value: String,
    #[serde(default)]
    #[serde(
        serialize_with = "serialize_opt_system_time",
        deserialize_with = "deserialize_opt_system_time"
    )]
    expires_at: Option<SystemTime>,
    source: SecretSource,
}

fn serialize_opt_system_time<S: serde::Serializer>(
    time: &Option<SystemTime>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match time {
        Some(t) => {
            let millis = t
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            s.serialize_some(&millis)
        }
        None => s.serialize_none(),
    }
}

fn deserialize_opt_system_time<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<SystemTime>, D::Error> {
    let opt: Option<u64> = Option::deserialize(d)?;
    Ok(opt.map(|millis| std::time::UNIX_EPOCH + std::time::Duration::from_millis(millis)))
}

impl Secret {
    /// Create a secret read from an environment variable.
    pub fn from_env_var(var_name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            expires_at: None,
            source: SecretSource::EnvVar(var_name.into()),
        }
    }

    /// Create a static (hardcoded) secret.
    pub fn static_value(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            expires_at: None,
            source: SecretSource::Static,
        }
    }

    /// Create a secret with full control over all fields.
    pub fn new(
        value: impl Into<String>,
        source: SecretSource,
        expires_at: Option<SystemTime>,
    ) -> Self {
        Self {
            value: value.into(),
            expires_at,
            source,
        }
    }

    /// Expose the raw secret value.
    pub fn expose(&self) -> &str {
        &self.value
    }

    /// When this secret expires, if ever.
    pub fn expires_at(&self) -> Option<SystemTime> {
        self.expires_at
    }

    /// How the secret was acquired.
    pub fn source(&self) -> &SecretSource {
        &self.source
    }

    /// Whether this secret is still valid (not expired).
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => SystemTime::now() < expiry,
            None => true,
        }
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret(***)")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

// ---------------------------------------------------------------------------
// AuthScheme
// ---------------------------------------------------------------------------

/// How a credential attaches to an HTTP request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthScheme {
    /// `Authorization: Bearer {token}`
    Bearer,
    /// Custom header: `{name}: {token}`
    Header { name: String },
    /// `Authorization: Basic base64({username}:{token})`
    Basic { username: String },
}

// ---------------------------------------------------------------------------
// CredentialError
// ---------------------------------------------------------------------------

/// Errors during credential acquisition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialError {
    /// The required environment variable is not set.
    MissingEnvVar { var_name: String },
    /// The credential has expired.
    Expired { service: String },
    /// Generic acquisition failure.
    AcquisitionFailed { service: String, message: String },
}

impl fmt::Display for CredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialError::MissingEnvVar { var_name } => {
                write!(f, "missing environment variable '{var_name}'")
            }
            CredentialError::Expired { service } => {
                write!(f, "credential for '{service}' has expired")
            }
            CredentialError::AcquisitionFailed { service, message } => {
                write!(f, "failed to acquire credential for '{service}': {message}")
            }
        }
    }
}

impl std::error::Error for CredentialError {}

// ---------------------------------------------------------------------------
// Credential
// ---------------------------------------------------------------------------

/// A fully resolved credential: secret + how to attach it.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    secret: Secret,
    scheme: AuthScheme,
}

impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credential")
            .field("secret", &self.secret)
            .field("scheme", &self.scheme)
            .finish()
    }
}

impl Credential {
    /// Create a new credential.
    pub fn new(secret: Secret, scheme: AuthScheme) -> Self {
        Self { secret, scheme }
    }

    /// The secret value.
    pub fn secret(&self) -> &Secret {
        &self.secret
    }

    /// The auth scheme.
    pub fn scheme(&self) -> &AuthScheme {
        &self.scheme
    }

    /// Whether this credential is still valid (secret not expired).
    pub fn is_valid(&self) -> bool {
        self.secret.is_valid()
    }

    /// A stable identifier for the credential source (used in resource IDs).
    pub fn source_id(&self) -> String {
        match &self.secret.source {
            SecretSource::EnvVar(var) => var.clone(),
            SecretSource::Exchange { provider } => provider.clone(),
            SecretSource::Static => "static".to_string(),
        }
    }

    /// Apply this credential to a REST request.
    ///
    /// Sets the appropriate header(s) and clears `request.auth` since
    /// the credential is now directly embedded.
    pub fn apply(&self, req: &mut RestRequest) {
        let token = self.secret.expose();
        match &self.scheme {
            AuthScheme::Bearer => {
                req.headers
                    .insert("Authorization".to_string(), format!("Bearer {token}"));
            }
            AuthScheme::Header { name } => {
                req.headers.insert(name.clone(), token.to_string());
            }
            AuthScheme::Basic { username } => {
                let encoded = base64_encode(&format!("{username}:{token}"));
                req.headers
                    .insert("Authorization".to_string(), format!("Basic {encoded}"));
            }
        }
        req.auth = None;
    }
}

// ---------------------------------------------------------------------------
// base64 helper (duplicated from lib/transport executor — core/ir can't
// depend on lib/transport)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Resource trait impl
// ---------------------------------------------------------------------------

impl Resource for Credential {
    fn resource_id(&self) -> ResourceId {
        ResourceId::new(format!("credential:{}", self.source_id()))
    }

    fn access_mode(&self) -> AccessMode {
        AccessMode::Read
    }

    fn kind(&self) -> ResourceKind {
        ResourceKind::Capability
    }
}

// ---------------------------------------------------------------------------
// Value conversions (capability-marker pattern)
// ---------------------------------------------------------------------------

impl From<Credential> for Value {
    fn from(cred: Credential) -> Self {
        let mut map = BTreeMap::new();

        // Secret value as a SecretString
        map.insert(
            "token".to_string(),
            Value::Secret(SecretString::new(cred.secret.value)),
        );

        // Source
        match &cred.secret.source {
            SecretSource::EnvVar(var) => {
                map.insert("source_type".to_string(), Value::Str("env_var".to_string()));
                map.insert("source_id".to_string(), Value::Str(var.clone()));
            }
            SecretSource::Exchange { provider } => {
                map.insert(
                    "source_type".to_string(),
                    Value::Str("exchange".to_string()),
                );
                map.insert("source_id".to_string(), Value::Str(provider.clone()));
            }
            SecretSource::Static => {
                map.insert("source_type".to_string(), Value::Str("static".to_string()));
            }
        }

        // Expiry (millis since epoch, if set)
        if let Some(expiry) = cred.secret.expires_at {
            let millis = expiry
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            map.insert("expires_at".to_string(), Value::Int(millis));
        }

        // Scheme
        match &cred.scheme {
            AuthScheme::Bearer => {
                map.insert("scheme".to_string(), Value::Str("bearer".to_string()));
            }
            AuthScheme::Header { name } => {
                map.insert("scheme".to_string(), Value::Str("header".to_string()));
                map.insert("scheme_header".to_string(), Value::Str(name.clone()));
            }
            AuthScheme::Basic { username } => {
                map.insert("scheme".to_string(), Value::Str("basic".to_string()));
                map.insert("scheme_username".to_string(), Value::Str(username.clone()));
            }
        }

        // Capability marker
        map.insert("cap".to_string(), Value::Secret(capability_marker()));

        Value::Map(map)
    }
}

impl TryFrom<&Value> for Credential {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let map = match value {
            Value::Map(m) => m,
            _ => return Err("expected map for Credential".to_string()),
        };

        ensure_capability_marker(map, "Credential")?;

        // Token
        let token = match map.get("token") {
            Some(Value::Secret(s)) => s.expose().to_string(),
            _ => return Err("Credential missing 'token' secret".to_string()),
        };

        // Source
        let source_type = map
            .get("source_type")
            .and_then(Value::as_str)
            .ok_or_else(|| "Credential missing 'source_type'".to_string())?;
        let source = match source_type {
            "env_var" => {
                let id = map
                    .get("source_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Credential missing 'source_id' for env_var".to_string())?;
                SecretSource::EnvVar(id.to_string())
            }
            "exchange" => {
                let id = map
                    .get("source_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Credential missing 'source_id' for exchange".to_string())?;
                SecretSource::Exchange {
                    provider: id.to_string(),
                }
            }
            "static" => SecretSource::Static,
            other => return Err(format!("unknown Credential source_type: {other}")),
        };

        // Expiry
        let expires_at = map
            .get("expires_at")
            .and_then(Value::as_int)
            .map(|millis| std::time::UNIX_EPOCH + std::time::Duration::from_millis(millis as u64));

        // Scheme
        let scheme_str = map
            .get("scheme")
            .and_then(Value::as_str)
            .ok_or_else(|| "Credential missing 'scheme'".to_string())?;
        let scheme = match scheme_str {
            "bearer" => AuthScheme::Bearer,
            "header" => {
                let name = map
                    .get("scheme_header")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Credential missing 'scheme_header'".to_string())?;
                AuthScheme::Header {
                    name: name.to_string(),
                }
            }
            "basic" => {
                let username = map
                    .get("scheme_username")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Credential missing 'scheme_username'".to_string())?;
                AuthScheme::Basic {
                    username: username.to_string(),
                }
            }
            other => return Err(format!("unknown Credential scheme: {other}")),
        };

        let secret = Secret::new(token, source, expires_at);
        Ok(Credential::new(secret, scheme))
    }
}

impl TryFrom<Value> for Credential {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Credential::try_from(&value)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn secret_redaction() {
        let secret = Secret::static_value("super-secret-token");
        assert_eq!(format!("{secret:?}"), "Secret(***)");
        assert_eq!(format!("{secret}"), "***");
        assert_eq!(secret.expose(), "super-secret-token");
    }

    #[test]
    fn credential_validity() {
        // Expired credential
        let past = SystemTime::now() - Duration::from_secs(3600);
        let expired = Credential::new(
            Secret::new("tok", SecretSource::Static, Some(past)),
            AuthScheme::Bearer,
        );
        assert!(!expired.is_valid());

        // Future credential
        let future = SystemTime::now() + Duration::from_secs(3600);
        let valid = Credential::new(
            Secret::new("tok", SecretSource::Static, Some(future)),
            AuthScheme::Bearer,
        );
        assert!(valid.is_valid());

        // No expiry
        let forever = Credential::new(Secret::static_value("tok"), AuthScheme::Bearer);
        assert!(forever.is_valid());
    }

    #[test]
    fn auth_scheme_apply() {
        // Bearer
        let cred = Credential::new(Secret::static_value("my-token"), AuthScheme::Bearer);
        let mut req = RestRequest::get("https://api.example.com");
        cred.apply(&mut req);
        assert_eq!(
            req.headers.get("Authorization"),
            Some(&"Bearer my-token".to_string())
        );
        assert!(req.auth.is_none());

        // Custom header
        let cred = Credential::new(
            Secret::static_value("sk-ant-123"),
            AuthScheme::Header {
                name: "x-api-key".to_string(),
            },
        );
        let mut req = RestRequest::get("https://api.example.com");
        cred.apply(&mut req);
        assert_eq!(
            req.headers.get("x-api-key"),
            Some(&"sk-ant-123".to_string())
        );
        assert!(req.auth.is_none());

        // Basic
        let cred = Credential::new(
            Secret::static_value("password"),
            AuthScheme::Basic {
                username: "user".to_string(),
            },
        );
        let mut req = RestRequest::get("https://api.example.com");
        cred.apply(&mut req);
        let auth_header = req.headers.get("Authorization").unwrap();
        assert!(auth_header.starts_with("Basic "));
        // Verify the base64 encodes "user:password"
        let expected = format!("Basic {}", base64_encode("user:password"));
        assert_eq!(auth_header, &expected);
        assert!(req.auth.is_none());
    }

    #[test]
    fn value_round_trip() {
        // Bearer with env var source
        let cred = Credential::new(
            Secret::from_env_var("GITHUB_TOKEN", "ghp_abc123"),
            AuthScheme::Bearer,
        );
        let value: Value = cred.into();
        let restored = Credential::try_from(&value).expect("round-trip should succeed");
        assert_eq!(restored.secret.expose(), "ghp_abc123");
        assert!(matches!(restored.scheme, AuthScheme::Bearer));
        assert!(matches!(
            restored.secret.source,
            SecretSource::EnvVar(ref v) if v == "GITHUB_TOKEN"
        ));

        // Header with exchange source
        let cred = Credential::new(
            Secret::new(
                "tok",
                SecretSource::Exchange {
                    provider: "oidc".to_string(),
                },
                None,
            ),
            AuthScheme::Header {
                name: "x-api-key".to_string(),
            },
        );
        let value: Value = cred.into();
        let restored = Credential::try_from(value).expect("round-trip should succeed");
        assert!(matches!(
            restored.scheme,
            AuthScheme::Header { ref name } if name == "x-api-key"
        ));
        assert!(matches!(
            restored.secret.source,
            SecretSource::Exchange { ref provider } if provider == "oidc"
        ));

        // Basic with static source and expiry
        let expiry = SystemTime::now() + Duration::from_secs(3600);
        let cred = Credential::new(
            Secret::new("pass", SecretSource::Static, Some(expiry)),
            AuthScheme::Basic {
                username: "admin".to_string(),
            },
        );
        let value: Value = cred.into();
        let restored = Credential::try_from(&value).expect("round-trip should succeed");
        assert_eq!(restored.secret.expose(), "pass");
        assert!(matches!(
            restored.scheme,
            AuthScheme::Basic { ref username } if username == "admin"
        ));
        assert!(restored.secret.expires_at.is_some());
    }

    #[test]
    fn capability_marker_rejection() {
        // Build a map without the capability marker
        let mut map = BTreeMap::new();
        map.insert("token".to_string(), Value::Secret(SecretString::new("tok")));
        map.insert("scheme".to_string(), Value::Str("bearer".to_string()));
        map.insert("source_type".to_string(), Value::Str("static".to_string()));

        let value = Value::Map(map);
        let result = Credential::try_from(&value);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("capability marker"));
    }

    #[test]
    fn resource_trait() {
        let cred = Credential::new(
            Secret::from_env_var("GITHUB_TOKEN", "ghp_abc"),
            AuthScheme::Bearer,
        );

        assert_eq!(
            cred.resource_id(),
            ResourceId::new("credential:GITHUB_TOKEN")
        );
        assert_eq!(cred.access_mode(), AccessMode::Read);
        assert_eq!(cred.kind(), ResourceKind::Capability);
    }
}
