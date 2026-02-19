//! Secret Manager service interface.
//!
//! Models the `secretmanager.googleapis.com/v1` API surface.
//! All methods produce `RestRequest` values for the DAG transport layer.

use super::base_urls::SECRET_MANAGER;
use super::{GcpRestClient, MethodMeta};
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
    fn access_secret_version(&self, project: &str, secret: &str, version: &str) -> RestRequest;

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
    fn add_secret_version(&self, project: &str, secret: &str, payload_base64: &str) -> RestRequest;

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

super::impl_gcp_rest_client_constructors!(SecretManagerRest);
super::impl_gcp_rest_client!(SecretManagerRest, SECRET_MANAGER);

impl SecretManagerService for SecretManagerRest {
    fn access_secret_version(&self, project: &str, secret: &str, version: &str) -> RestRequest {
        self.request_from_meta(
            &ACCESS_SECRET_VERSION_META,
            &[
                ("project", project),
                ("secret", secret),
                ("version", version),
            ],
            &[],
        )
    }

    fn get_secret(&self, project: &str, secret: &str) -> RestRequest {
        self.request_from_meta(
            &GET_SECRET_META,
            &[("project", project), ("secret", secret)],
            &[],
        )
    }

    fn create_secret(&self, project: &str, secret_id: &str) -> RestRequest {
        self.request_from_meta(
            &CREATE_SECRET_META,
            &[("project", project)],
            &[("secretId", secret_id)],
        )
        .json(serde_json::json!({
            "replication": {
                "automatic": {}
            }
        }))
    }

    fn add_secret_version(&self, project: &str, secret: &str, payload_base64: &str) -> RestRequest {
        self.request_from_meta(
            &ADD_SECRET_VERSION_META,
            &[("project", project), ("secret", secret)],
            &[],
        )
        .json(serde_json::json!({
            "payload": {
                "data": payload_base64
            }
        }))
    }

    fn list_secrets(&self, project: &str) -> RestRequest {
        self.request_from_meta(&LIST_SECRETS_META, &[("project", project)], &[])
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_request_matches_meta(
        req: &RestRequest,
        meta: &MethodMeta,
        path_params: &[(&str, &str)],
        query_params: &[(&str, &str)],
    ) {
        let expected = meta.build_request(SECRET_MANAGER, path_params, query_params);
        assert_eq!(req.method, expected.method);
        assert_eq!(req.url, expected.url);
    }

    #[test]
    fn test_access_secret_version_url() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.access_secret_version("my-project", "my-secret", "latest");
        assert!(req
            .url
            .contains("/v1/projects/my-project/secrets/my-secret/versions/latest:access"));
        assert_eq!(req.method, HttpMethod::Get);
        assert_request_matches_meta(
            &req,
            &ACCESS_SECRET_VERSION_META,
            &[
                ("project", "my-project"),
                ("secret", "my-secret"),
                ("version", "latest"),
            ],
            &[],
        );
    }

    #[test]
    fn test_get_secret_url() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.get_secret("my-project", "my-secret");
        assert!(req
            .url
            .contains("/v1/projects/my-project/secrets/my-secret"));
        assert_eq!(req.method, HttpMethod::Get);
        assert_request_matches_meta(
            &req,
            &GET_SECRET_META,
            &[("project", "my-project"), ("secret", "my-secret")],
            &[],
        );
    }

    #[test]
    fn test_create_secret_url_and_body() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.create_secret("my-project", "new-secret");
        assert!(req.url.contains("/v1/projects/my-project/secrets"));
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.body.is_some());
        // `build_request` appends query params to the URL instead of the `RestRequest.query` map.
        assert!(
            req.url.ends_with("?secretId=new-secret") || req.url.contains("?secretId=new-secret&")
        );
        assert_request_matches_meta(
            &req,
            &CREATE_SECRET_META,
            &[("project", "my-project")],
            &[("secretId", "new-secret")],
        );
    }

    #[test]
    fn test_add_secret_version_body() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.add_secret_version("my-project", "my-secret", "c2VjcmV0");
        assert!(req.url.contains(":addVersion"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.as_ref().unwrap();
        assert_eq!(body["payload"]["data"], "c2VjcmV0");
        assert_request_matches_meta(
            &req,
            &ADD_SECRET_VERSION_META,
            &[("project", "my-project"), ("secret", "my-secret")],
            &[],
        );
    }

    #[test]
    fn test_list_secrets_url() {
        let svc = SecretManagerRest::unauthenticated();
        let req = svc.list_secrets("my-project");
        assert!(req.url.contains("/v1/projects/my-project/secrets"));
        assert_eq!(req.method, HttpMethod::Get);
        assert_request_matches_meta(&req, &LIST_SECRETS_META, &[("project", "my-project")], &[]);
    }

    #[test]
    fn test_method_meta_access_secret_version() {
        const { assert!(ACCESS_SECRET_VERSION_META.idempotent) };
        const { assert!(ACCESS_SECRET_VERSION_META.read_only) };
        assert_eq!(
            ACCESS_SECRET_VERSION_META.permissions,
            &["secretmanager.versions.access"]
        );
    }
}
