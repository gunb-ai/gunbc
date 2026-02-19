//! Typed GCP service interfaces.
//!
//! Each module models a GCP API surface as a Rust trait with:
//! - Typed methods (not flat enum variants)
//! - REST endpoint metadata (URL, HTTP method, permissions)
//! - Behavior properties (idempotent, read-only)
//!
//! Design follows:
//! - gunb.ai's `gcloud.Admin` interface (typed methods per service)
//! - the-gunbai's behavior-based understanding model (MethodMeta)
//!
//! Service traits produce `RestRequest` values that flow through the
//! existing DAG transport layer (prepare → execute → parse).

pub mod cloud_run;
pub mod compute_engine;
pub mod iam;
pub mod load_balancer;
pub mod local_auth;
pub mod resource_manager;
pub mod secret_manager;
pub mod storage;
pub mod workload_identity;

pub use cloud_run::CloudRunService;
pub use compute_engine::ComputeEngineService;
pub use iam::IamService;
pub use load_balancer::LoadBalancerService;
pub use local_auth::{GcloudCli, GcloudLoginOptions, LocalAuthService};
pub use resource_manager::ResourceManagerService;
pub use secret_manager::SecretManagerService;
pub use storage::StorageService;
pub use workload_identity::WorkloadIdentityService;

use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// Behavior metadata for a service method.
///
/// Declares the REST endpoint, HTTP method, and behavioral properties.
/// This enables tooling to inspect service contracts without executing them.
///
/// All fields are `'static` -- these are compile-time constants, not runtime data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodMeta {
    /// REST endpoint path template (e.g., "/v1/projects/{project}/secrets/{secret}").
    pub endpoint: &'static str,
    /// HTTP method.
    pub http_method: HttpMethod,
    /// Whether the method is idempotent (safe to retry).
    pub idempotent: bool,
    /// Whether the method is read-only (no side effects).
    pub read_only: bool,
    /// Required IAM permissions.
    pub permissions: &'static [&'static str],
    /// GCP API service name (for rate limiting).
    pub service: &'static str,
}

/// Shared REST client pattern for GCP service implementations.
///
/// Implementors provide a base URL and optional credential; the trait
/// provides authenticated HTTP helper methods as defaults.
pub trait GcpRestClient {
    /// Base URL for the service (e.g. `https://run.googleapis.com`).
    fn base_url(&self) -> &str;

    /// Optional credential for authentication.
    fn credential(&self) -> Option<&Credential>;

    /// Build an authenticated request at a specific base URL.
    fn authed_request_at(&self, base_url: &str, method: HttpMethod, path: &str) -> RestRequest {
        let url = format!("{}{}", base_url, path);
        let mut req = match method {
            HttpMethod::Get => RestRequest::get(url),
            HttpMethod::Post => RestRequest::post(url),
            HttpMethod::Put => RestRequest::put(url),
            HttpMethod::Delete => RestRequest::delete(url),
            _ => {
                let mut r = RestRequest::post(url);
                r.method = method;
                r
            }
        };
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    /// Build an authenticated request at the configured base URL.
    fn authed_request(&self, method: HttpMethod, path: &str) -> RestRequest {
        self.authed_request_at(self.base_url(), method, path)
    }

    /// Authenticated GET at the configured base URL.
    fn authed_get(&self, path: &str) -> RestRequest {
        self.authed_request(HttpMethod::Get, path)
    }

    /// Authenticated POST at the configured base URL.
    fn authed_post(&self, path: &str) -> RestRequest {
        self.authed_request(HttpMethod::Post, path)
    }

    /// Authenticated PATCH at the configured base URL.
    fn authed_patch(&self, path: &str) -> RestRequest {
        self.authed_request(HttpMethod::Patch, path)
    }

    /// Authenticated PUT at the configured base URL.
    fn authed_put(&self, path: &str) -> RestRequest {
        self.authed_request(HttpMethod::Put, path)
    }

    /// Authenticated DELETE at the configured base URL.
    fn authed_delete(&self, path: &str) -> RestRequest {
        self.authed_request(HttpMethod::Delete, path)
    }
}

macro_rules! impl_gcp_rest_client {
    ($ty:ty, $base_url:expr) => {
        impl GcpRestClient for $ty {
            fn base_url(&self) -> &str {
                $base_url
            }

            fn credential(&self) -> Option<&Credential> {
                self.auth.as_ref()
            }
        }
    };
}

pub(crate) use impl_gcp_rest_client;

/// GCP API base URLs.
pub mod base_urls {
    /// Secret Manager API.
    pub const SECRET_MANAGER: &str = "https://secretmanager.googleapis.com";
    /// Compute Engine API.
    pub const COMPUTE: &str = "https://compute.googleapis.com";
    /// Cloud Run API.
    pub const RUN: &str = "https://run.googleapis.com";
    /// IAM API.
    pub const IAM: &str = "https://iam.googleapis.com";
    /// IAM Credentials API (for impersonation / token generation).
    pub const IAM_CREDENTIALS: &str = "https://iamcredentials.googleapis.com";
    /// Cloud Resource Manager API.
    pub const RESOURCE_MANAGER: &str = "https://cloudresourcemanager.googleapis.com";
    /// Cloud Storage API.
    pub const STORAGE: &str = "https://storage.googleapis.com";
    /// STS (Security Token Service) API.
    pub const STS: &str = "https://sts.googleapis.com";
    /// OAuth2 token endpoint.
    pub const OAUTH2: &str = "https://oauth2.googleapis.com";
}
