//! Self-describing diagnostics for resource acquisition.
//!
//! Every resource acquisition is a lock interaction. The lock (resource) and
//! key (credential/capability) know how to describe themselves. The error
//! system delegates to them instead of hardcoding auth-specific formatting.

use std::fmt;

use gunbc_ir::transport::credential::{AuthScheme, Credential, SecretSource};
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Projection functions: concrete types → diagnostic vocabulary
// ---------------------------------------------------------------------------

/// Project a [`Credential`] into a [`KeyIdentity`] for diagnostic display.
pub fn credential_as_key(cred: &Credential) -> KeyIdentity {
    let scheme = match cred.scheme() {
        AuthScheme::Bearer => "Bearer".to_string(),
        AuthScheme::Header { name } => format!("Header:{name}"),
        AuthScheme::Basic { username } => format!("Basic:{username}"),
    };
    let hint = cred.secret().hint();
    let source = match cred.secret().source() {
        SecretSource::EnvVar(var) => format!("env:{var}"),
        SecretSource::Exchange { provider } => format!("exchange:{provider}"),
        SecretSource::Static => "static".into(),
    };
    KeyIdentity {
        scheme,
        hint,
        source,
    }
}

/// Project a [`RestRequest`] into a [`LockIdentity`] for diagnostic display.
pub fn rest_request_as_lock(req: &RestRequest) -> LockIdentity {
    LockIdentity {
        resource: "AuthContext".into(),
        mode: "Read".into(),
        target: format!("{} {}", req.method, req.url),
    }
}

/// Build an [`AcquisitionDiagnostic`] from a REST request and optional credential.
pub fn rest_acquisition_diagnostic(
    req: &RestRequest,
    cred: Option<&Credential>,
) -> AcquisitionDiagnostic {
    AcquisitionDiagnostic {
        lock: rest_request_as_lock(req),
        key: cred.map(credential_as_key),
    }
}

// ---------------------------------------------------------------------------
// Core diagnostic types
// ---------------------------------------------------------------------------

/// Lock identity — what resource is being accessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockIdentity {
    pub resource: String,
    pub mode: String,
    pub target: String,
}

/// Key identity — what credential/capability is being used to access the resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyIdentity {
    pub scheme: String,
    pub hint: String,
    pub source: String,
}

/// A resource acquisition viewed as a lock interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquisitionDiagnostic {
    pub lock: LockIdentity,
    pub key: Option<KeyIdentity>,
}

impl fmt::Display for LockIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}): {}", self.resource, self.mode, self.target)
    }
}

impl fmt::Display for KeyIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (key: {}, source: {})",
            self.scheme, self.hint, self.source
        )
    }
}

impl fmt::Display for AcquisitionDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lock)?;
        if let Some(k) = &self.key {
            write!(f, " with {}", k)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::credential::Secret;

    #[test]
    fn credential_as_key_bearer_env() {
        let cred = Credential::new(
            Secret::from_env_var(
                "GITHUB_TOKEN",
                "ghp_1234567890abcdef1234567890abcdef12345678",
            ),
            AuthScheme::Bearer,
        );
        let key = credential_as_key(&cred);
        assert_eq!(key.scheme, "Bearer");
        assert_eq!(key.hint, "\"***\"");
        assert_eq!(key.source, "env:GITHUB_TOKEN");
    }

    #[test]
    fn credential_as_key_header_exchange() {
        let cred = Credential::new(
            Secret::new(
                "sk-ant-api-key-value",
                SecretSource::Exchange {
                    provider: "gcp".into(),
                },
                None,
            ),
            AuthScheme::Header {
                name: "x-api-key".into(),
            },
        );
        let key = credential_as_key(&cred);
        assert_eq!(key.scheme, "Header:x-api-key");
        assert_eq!(key.hint, "\"***\"");
        assert_eq!(key.source, "exchange:gcp");
    }

    #[test]
    fn credential_as_key_basic_static() {
        let cred = Credential::new(
            Secret::static_value("password123"),
            AuthScheme::Basic {
                username: "admin".into(),
            },
        );
        let key = credential_as_key(&cred);
        assert_eq!(key.scheme, "Basic:admin");
        assert_eq!(key.hint, "\"***\"");
        assert_eq!(key.source, "static");
    }

    #[test]
    fn rest_request_as_lock_formats_method_url() {
        let req = RestRequest::post("https://api.github.com/gists");
        let lock = rest_request_as_lock(&req);
        assert_eq!(lock.resource, "AuthContext");
        assert_eq!(lock.mode, "Read");
        assert_eq!(lock.target, "POST https://api.github.com/gists");
    }

    #[test]
    fn rest_acquisition_diagnostic_with_cred() {
        let req = RestRequest::post("https://api.github.com/gists");
        let cred = Credential::new(
            Secret::from_env_var(
                "GITHUB_TOKEN",
                "ghp_1234567890abcdef1234567890abcdef12345678",
            ),
            AuthScheme::Bearer,
        );
        let diag = rest_acquisition_diagnostic(&req, Some(&cred));
        assert!(diag.key.is_some());
        let display = diag.to_string();
        assert!(display.contains("POST https://api.github.com/gists"));
        assert!(display.contains("Bearer"));
        assert!(display.contains("\"***\""));
        assert!(display.contains("env:GITHUB_TOKEN"));
    }

    #[test]
    fn rest_acquisition_diagnostic_no_cred() {
        let req = RestRequest::get("https://api.example.com/public");
        let diag = rest_acquisition_diagnostic(&req, None);
        assert!(diag.key.is_none());
        assert_eq!(
            diag.to_string(),
            "AuthContext (Read): GET https://api.example.com/public"
        );
    }

    #[test]
    fn lock_identity_display() {
        let lock = LockIdentity {
            resource: "AuthContext".into(),
            mode: "Read".into(),
            target: "POST https://api.github.com/gists".into(),
        };
        assert_eq!(
            lock.to_string(),
            "AuthContext (Read): POST https://api.github.com/gists"
        );
    }

    #[test]
    fn key_identity_display() {
        let key = KeyIdentity {
            scheme: "Bearer".into(),
            hint: "\"***\"".into(),
            source: "env:GITHUB_TOKEN".into(),
        };
        assert_eq!(
            key.to_string(),
            "Bearer (key: \"***\", source: env:GITHUB_TOKEN)"
        );
    }

    #[test]
    fn acquisition_diagnostic_with_key() {
        let diag = AcquisitionDiagnostic {
            lock: LockIdentity {
                resource: "AuthContext".into(),
                mode: "Read".into(),
                target: "POST https://api.github.com/gists".into(),
            },
            key: Some(KeyIdentity {
                scheme: "Bearer".into(),
                hint: "\"***\"".into(),
                source: "env:GITHUB_TOKEN".into(),
            }),
        };
        assert_eq!(
            diag.to_string(),
            "AuthContext (Read): POST https://api.github.com/gists with Bearer (key: \"***\", source: env:GITHUB_TOKEN)"
        );
    }

    #[test]
    fn acquisition_diagnostic_without_key() {
        let diag = AcquisitionDiagnostic {
            lock: LockIdentity {
                resource: "Filesystem".into(),
                mode: "Write".into(),
                target: "/tmp/output.json".into(),
            },
            key: None,
        };
        assert_eq!(diag.to_string(), "Filesystem (Write): /tmp/output.json");
    }
}
