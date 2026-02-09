//! Secret Manager service interface.
//!
//! Models the `secretmanager.googleapis.com/v1` API surface.
//! All methods produce `RestRequest` values for the DAG transport layer.

use super::base_urls::SECRET_MANAGER;
use super::MethodMeta;
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Secret Manager service interface.
///
/// Models the secretmanager.googleapis.com/v1 API surface.
/// Each method returns a `RestRequest` ready for execution.
pub trait SecretManagerService {
    /// Access a secret version's payload.
    ///
    /// `GET /v1/projects/{project}/secrets/{secret}/versions/{version}:access`
    fn access_secret_version(
        &self,
        project: &str,
        secret: &str,
        version: &str,
    ) -> RestRequest;

    /// Get secret metadata (check existence).
    ///
    /// `GET /v1/projects/{project}/secrets/{secret}`
    fn get_secret(&self, project: &str, secret: &str) -> RestRequest;

    /// Create a new secret.
    ///
    /// `POST /v1/projects/{project}/secrets`
    fn create_secret(&self, project: &str, secret_id: &str) -> RestRequest;

    /// Add a version to an existing secret.
    ///
    /// `POST /v1/projects/{project}/secrets/{secret}:addVersion`
    fn add_secret_version(
        &self,
        project: &str,
        secret: &str,
        payload_base64: &str,
    ) -> RestRequest;

    /// List all secrets in a project.
    ///
    /// `GET /v1/projects/{project}/secrets`
    fn list_secrets(&self, project: &str) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata (behavior declarations)
// ---------------------------------------------------------------------------

/// Metadata for `access_secret_version`.
pub const ACCESS_SECRET_VERSION_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/secrets/{secret}/versions/{version}:access",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["secretmanager.versions.access"],
    service: "secretmanager",
};

/// Metadata for `get_secret`.
pub const GET_SECRET_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/secrets/{secret}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["secretmanager.secrets.get"],
    service: "secretmanager",
};

/// Metadata for `create_secret`.
pub const CREATE_SECRET_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/secrets",
    http_method: HttpMethod::Post,
    idempotent: false,
    read_only: false,
    permissions: &["secretmanager.secrets.create"],
    service: "secretmanager",
};

/// Metadata for `add_secret_version`.
pub const ADD_SECRET_VERSION_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/secrets/{secret}:addVersion",
    http_method: HttpMethod::Post,
    idempotent: false,
    read_only: false,
    permissions: &["secretmanager.versions.add"],
    service: "secretmanager",
};

/// Metadata for `list_secrets`.
pub const LIST_SECRETS_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/secrets",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["secretmanager.secrets.list"],
    service: "secretmanager",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

/// REST-based implementation of the Secret Manager service.
///
/// Produces `RestRequest` values with proper URLs, headers, and auth.
#[derive(Debug, Clone)]
pub struct SecretManagerRest {
    auth: Option<Credential>,
}

impl SecretManagerRest {
    /// Create a new REST client with the given auth credential.
    pub fn new(auth: Credential) -> Self {
        Self { auth: Some(auth) }
    }

    /// Create a new REST client without auth (for testing).
    pub fn unauthenticated() -> Self {
        Self { auth: None }
    }

    fn base_request(&self, method: HttpMethod, path: &str) -> RestRequest {
        let url = format!("{}{}", SECRET_MANAGER, path);
        let mut req = match method {
            HttpMethod::Get => RestRequest::get(url),
            HttpMethod::Post => RestRequest::post(url),
            HttpMethod::Put => RestRequest::put(url),
            HttpMethod::Delete => RestRequest::delete(url),
            _ => RestRequest::get(url),
        };
        if let Some(ref auth) = self.auth {
            req = req.credential(auth.clone());
        }
        req
    }
}

impl SecretManagerService for SecretManagerRest {
    fn access_secret_version(
        &self,
        project: &str,
        secret: &str,
        version: &str,
    ) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/secrets/{}/versions/{}:access",
            project, secret, version
        );
        self.base_request(HttpMethod::Get, &path)
    }

    fn get_secret(&self, project: &str, secret: &str) -> RestRequest {
        let path = format!("/v1/projects/{}/secrets/{}", project, secret);
        self.base_request(HttpMethod::Get, &path)
    }

    fn create_secret(&self, project: &str, secret_id: &str) -> RestRequest {
        let path = format!("/v1/projects/{}/secrets", project);
        self.base_request(HttpMethod::Post, &path)
            .json(serde_json::json!({
                "replication": {
                    "automatic": {}
                }
            }))
            .query("secretId", secret_id)
    }

    fn add_secret_version(
        &self,
        project: &str,
        secret: &str,
        payload_base64: &str,
    ) -> RestRequest {
        let path = format!("/v1/projects/{}/secrets/{}:addVersion", project, secret);
        self.base_request(HttpMethod::Post, &path)
            .json(serde_json::json!({
                "payload": {
                    "data": payload_base64
                }
            }))
    }

    fn list_secrets(&self, project: &str) -> RestRequest {
        let path = format!("/v1/projects/{}/secrets", project);
        self.base_request(HttpMethod::Get, &path)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_secret_version_url() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.access_secret_version("my-project", "my-secret", "latest");
        assert!(req.url.contains("/v1/projects/my-project/secrets/my-secret/versions/latest:access"));
        assert_eq!(req.method, HttpMethod::Get);
    }

    #[test]
    fn test_get_secret_url() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.get_secret("my-project", "my-secret");
        assert!(req.url.contains("/v1/projects/my-project/secrets/my-secret"));
        assert_eq!(req.method, HttpMethod::Get);
    }

    #[test]
    fn test_create_secret_url_and_body() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.create_secret("my-project", "new-secret");
        assert!(req.url.contains("/v1/projects/my-project/secrets"));
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.body.is_some());
        assert!(req.query.contains_key("secretId"));
        assert_eq!(req.query["secretId"], "new-secret");
    }

    #[test]
    fn test_add_secret_version_body() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.add_secret_version("my-project", "my-secret", "c2VjcmV0");
        assert!(req.url.contains(":addVersion"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.unwrap();
        assert_eq!(body["payload"]["data"], "c2VjcmV0");
    }

    #[test]
    fn test_list_secrets_url() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.list_secrets("my-project");
        assert!(req.url.contains("/v1/projects/my-project/secrets"));
        assert_eq!(req.method, HttpMethod::Get);
    }

    #[test]
    fn test_method_meta_access_secret_version() {
        assert!(ACCESS_SECRET_VERSION_META.idempotent);
        assert!(ACCESS_SECRET_VERSION_META.read_only);
        assert_eq!(ACCESS_SECRET_VERSION_META.permissions, &["secretmanager.versions.access"]);
    }
}
