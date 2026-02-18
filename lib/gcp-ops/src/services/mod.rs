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

pub mod iam;
pub mod compute_engine;
pub mod cloud_run;
pub mod local_auth;
pub mod load_balancer;
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

use gunbc_ir::transport::http::HttpMethod;

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
