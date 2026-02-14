//! Workload Identity Federation service interface.
//!
//! Models the WIF subset of `iam.googleapis.com/v1` for discovering
//! and managing workload identity pools and providers.

use super::base_urls::IAM;
use super::MethodMeta;
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Workload Identity Federation service interface.
pub trait WorkloadIdentityService {
    /// List WIF pools in a project.
    ///
    /// `GET /v1/projects/{project}/locations/global/workloadIdentityPools`
    fn list_pools(&self, project: &str) -> RestRequest;

    /// Get a specific WIF pool.
    ///
    /// `GET /v1/projects/{project}/locations/global/workloadIdentityPools/{pool}`
    fn get_pool(&self, project: &str, pool_id: &str) -> RestRequest;

    /// List providers in a WIF pool.
    ///
    /// `GET /v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers`
    fn list_providers(&self, project: &str, pool_id: &str) -> RestRequest;

    /// Get a specific WIF provider.
    ///
    /// `GET /v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers/{provider}`
    fn get_provider(&self, project: &str, pool_id: &str, provider_id: &str) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata
// ---------------------------------------------------------------------------

/// Metadata for `list_pools`.
pub const LIST_POOLS_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/locations/global/workloadIdentityPools",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["iam.workloadIdentityPools.list"],
    service: "iam",
};

/// Metadata for `get_pool`.
pub const GET_POOL_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/locations/global/workloadIdentityPools/{pool}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["iam.workloadIdentityPools.get"],
    service: "iam",
};

/// Metadata for `list_providers`.
pub const LIST_PROVIDERS_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["iam.workloadIdentityPoolProviders.list"],
    service: "iam",
};

/// Metadata for `get_provider`.
pub const GET_PROVIDER_META: MethodMeta = MethodMeta {
    endpoint:
        "/v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers/{provider}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["iam.workloadIdentityPoolProviders.get"],
    service: "iam",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

/// REST-based implementation of the Workload Identity service.
#[derive(Debug, Clone)]
pub struct WorkloadIdentityRest {
    auth: Option<Credential>,
}

impl WorkloadIdentityRest {
    /// Create a new REST client with the given auth credential.
    pub fn new(auth: Credential) -> Self {
        Self { auth: Some(auth) }
    }

    /// Create a new REST client without auth (for testing).
    pub fn unauthenticated() -> Self {
        Self { auth: None }
    }

    fn authed_get(&self, path: &str) -> RestRequest {
        let url = format!("{}{}", IAM, path);
        let mut req = RestRequest::get(url);
        if let Some(ref auth) = self.auth {
            req = req.credential(auth.clone());
        }
        req
    }
}

impl WorkloadIdentityService for WorkloadIdentityRest {
    fn list_pools(&self, project: &str) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/locations/global/workloadIdentityPools",
            project
        );
        self.authed_get(&path)
    }

    fn get_pool(&self, project: &str, pool_id: &str) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/locations/global/workloadIdentityPools/{}",
            project, pool_id
        );
        self.authed_get(&path)
    }

    fn list_providers(&self, project: &str, pool_id: &str) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/locations/global/workloadIdentityPools/{}/providers",
            project, pool_id
        );
        self.authed_get(&path)
    }

    fn get_provider(&self, project: &str, pool_id: &str, provider_id: &str) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/locations/global/workloadIdentityPools/{}/providers/{}",
            project, pool_id, provider_id
        );
        self.authed_get(&path)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_pools_url() {
        let svc = WorkloadIdentityRest::unauthenticated();
        let req = svc.list_pools("my-project");
        assert!(req.url.contains("/workloadIdentityPools"));
        assert_eq!(req.method, HttpMethod::Get);
    }

    #[test]
    fn test_get_pool_url() {
        let svc = WorkloadIdentityRest::unauthenticated();
        let req = svc.get_pool("my-project", "github-pool");
        assert!(req.url.contains("workloadIdentityPools/github-pool"));
    }

    #[test]
    fn test_list_providers_url() {
        let svc = WorkloadIdentityRest::unauthenticated();
        let req = svc.list_providers("my-project", "github-pool");
        assert!(req
            .url
            .contains("workloadIdentityPools/github-pool/providers"));
    }

    #[test]
    fn test_get_provider_url() {
        let svc = WorkloadIdentityRest::unauthenticated();
        let req = svc.get_provider("my-project", "github-pool", "github");
        assert!(req.url.contains("providers/github"));
    }
}
