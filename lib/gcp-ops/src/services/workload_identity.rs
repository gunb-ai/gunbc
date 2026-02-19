//! Workload Identity Federation service interface.
//!
//! Models the WIF subset of `iam.googleapis.com/v1` for discovering
//! and managing workload identity pools and providers.

use super::base_urls::IAM;
use super::{GcpRestClient, MethodMeta};
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;
use std::collections::HashMap;

/// Configuration payload for creating/updating WIF providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifProviderConfig {
    /// OIDC issuer URI (e.g. `https://token.actions.githubusercontent.com`).
    pub oidc_issuer_uri: String,
    /// Attribute mapping from Google fields to token assertions.
    pub attribute_mapping: HashMap<String, String>,
    /// Optional CEL expression restricting valid tokens.
    pub attribute_condition: Option<String>,
}

impl WifProviderConfig {
    fn to_provider_json(&self) -> serde_json::Value {
        let mut body = serde_json::json!({
            "oidc": {
                "issuerUri": self.oidc_issuer_uri
            },
            "attributeMapping": self.attribute_mapping,
        });
        if let Some(condition) = &self.attribute_condition {
            body["attributeCondition"] = serde_json::json!(condition);
        }
        body
    }
}

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

    /// Create a workload identity pool.
    ///
    /// `POST /v1/projects/{project}/locations/global/workloadIdentityPools?workloadIdentityPoolId={pool}`
    fn create_pool(&self, project: &str, pool_id: &str, display_name: &str) -> RestRequest;

    /// Create a workload identity provider in a pool.
    ///
    /// `POST /v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers?workloadIdentityPoolProviderId={provider}`
    fn create_provider(
        &self,
        project: &str,
        pool_id: &str,
        provider_id: &str,
        config: &WifProviderConfig,
    ) -> RestRequest;

    /// Update an existing workload identity provider.
    ///
    /// `PATCH /v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers/{provider}?updateMask=oidc,attributeMapping,attributeCondition`
    fn update_provider(
        &self,
        project: &str,
        pool_id: &str,
        provider_id: &str,
        config: &WifProviderConfig,
    ) -> RestRequest;
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

/// Metadata for `create_pool`.
pub const CREATE_POOL_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/locations/global/workloadIdentityPools?workloadIdentityPoolId={pool}",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["iam.workloadIdentityPools.create"],
    service: "iam",
};

/// Metadata for `create_provider`.
pub const CREATE_PROVIDER_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers?workloadIdentityPoolProviderId={provider}",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["iam.workloadIdentityPoolProviders.create"],
    service: "iam",
};

/// Metadata for `update_provider`.
pub const UPDATE_PROVIDER_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/locations/global/workloadIdentityPools/{pool}/providers/{provider}?updateMask=oidc,attributeMapping,attributeCondition",
    http_method: HttpMethod::Patch,
    idempotent: true,
    read_only: false,
    permissions: &["iam.workloadIdentityPoolProviders.update"],
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

super::impl_gcp_rest_client_constructors!(WorkloadIdentityRest);
super::impl_gcp_rest_client!(WorkloadIdentityRest, IAM);

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

    fn create_pool(&self, project: &str, pool_id: &str, display_name: &str) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/locations/global/workloadIdentityPools?workloadIdentityPoolId={}",
            project, pool_id
        );
        self.authed_post(&path)
            .json(serde_json::json!({ "displayName": display_name }))
    }

    fn create_provider(
        &self,
        project: &str,
        pool_id: &str,
        provider_id: &str,
        config: &WifProviderConfig,
    ) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/locations/global/workloadIdentityPools/{}/providers?workloadIdentityPoolProviderId={}",
            project, pool_id, provider_id
        );
        self.authed_post(&path).json(config.to_provider_json())
    }

    fn update_provider(
        &self,
        project: &str,
        pool_id: &str,
        provider_id: &str,
        config: &WifProviderConfig,
    ) -> RestRequest {
        let path = format!(
            "/v1/projects/{}/locations/global/workloadIdentityPools/{}/providers/{}?updateMask=oidc,attributeMapping,attributeCondition",
            project, pool_id, provider_id
        );
        self.authed_patch(&path).json(config.to_provider_json())
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

    fn sample_provider_config() -> WifProviderConfig {
        WifProviderConfig {
            oidc_issuer_uri: "https://token.actions.githubusercontent.com".to_string(),
            attribute_mapping: HashMap::from([
                ("google.subject".to_string(), "assertion.sub".to_string()),
                (
                    "attribute.repository".to_string(),
                    "assertion.repository".to_string(),
                ),
            ]),
            attribute_condition: Some("assertion.repository_owner == 'gunb-ai'".to_string()),
        }
    }

    #[test]
    fn test_create_pool_request_shape() {
        let svc = WorkloadIdentityRest::unauthenticated();
        let req = svc.create_pool("my-project", "github-pool", "GitHub Pool");
        assert!(req.url.contains("workloadIdentityPoolId=github-pool"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.expect("create pool should include JSON body");
        assert_eq!(body["displayName"].as_str(), Some("GitHub Pool"));
    }

    #[test]
    fn test_create_provider_request_shape() {
        let svc = WorkloadIdentityRest::unauthenticated();
        let cfg = sample_provider_config();
        let req = svc.create_provider("my-project", "github-pool", "github", &cfg);
        assert!(req
            .url
            .contains("providers?workloadIdentityPoolProviderId=github"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.expect("create provider should include JSON body");
        assert_eq!(
            body["oidc"]["issuerUri"].as_str(),
            Some("https://token.actions.githubusercontent.com")
        );
        assert!(body["attributeMapping"].is_object());
        assert!(body["attributeCondition"].is_string());
    }

    #[test]
    fn test_update_provider_request_shape() {
        let svc = WorkloadIdentityRest::unauthenticated();
        let cfg = sample_provider_config();
        let req = svc.update_provider("my-project", "github-pool", "github", &cfg);
        assert!(req.url.contains("providers/github?updateMask="));
        assert_eq!(req.method, HttpMethod::Patch);
        let body = req.body.expect("update provider should include JSON body");
        assert_eq!(
            body["oidc"]["issuerUri"].as_str(),
            Some("https://token.actions.githubusercontent.com")
        );
    }
}
