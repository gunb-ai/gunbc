//! Cloud Resource Manager service interface.
//!
//! Models the `cloudresourcemanager.googleapis.com/v1` API surface
//! for project management and project-level IAM policies.

use super::base_urls::RESOURCE_MANAGER;
use super::MethodMeta;
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Cloud Resource Manager service interface.
pub trait ResourceManagerService {
    /// List projects accessible to the authenticated identity.
    ///
    /// `GET /v1/projects`
    fn list_projects(&self) -> RestRequest;

    /// Get a specific project by ID.
    ///
    /// `GET /v1/projects/{project}`
    fn get_project(&self, project: &str) -> RestRequest;

    /// Get the IAM policy for a project.
    ///
    /// `POST /v1/projects/{project}:getIamPolicy`
    fn get_iam_policy(&self, project: &str) -> RestRequest;

    /// Set the IAM policy for a project (read-modify-write with etag).
    ///
    /// `POST /v1/projects/{project}:setIamPolicy`
    fn set_iam_policy(
        &self,
        project: &str,
        policy: serde_json::Value,
    ) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata
// ---------------------------------------------------------------------------

/// Metadata for `list_projects`.
pub const LIST_PROJECTS_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["resourcemanager.projects.list"],
    service: "cloudresourcemanager",
};

/// Metadata for `get_project`.
pub const GET_PROJECT_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["resourcemanager.projects.get"],
    service: "cloudresourcemanager",
};

/// Metadata for `get_iam_policy`.
pub const GET_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}:getIamPolicy",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: true,
    permissions: &["resourcemanager.projects.getIamPolicy"],
    service: "cloudresourcemanager",
};

/// Metadata for `set_iam_policy`.
pub const SET_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}:setIamPolicy",
    http_method: HttpMethod::Post,
    idempotent: false,
    read_only: false,
    permissions: &["resourcemanager.projects.setIamPolicy"],
    service: "cloudresourcemanager",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

/// REST-based implementation of the Resource Manager service.
#[derive(Debug, Clone)]
pub struct ResourceManagerRest {
    auth: Option<Credential>,
}

impl ResourceManagerRest {
    /// Create a new REST client with the given auth credential.
    pub fn new(auth: Credential) -> Self {
        Self { auth: Some(auth) }
    }

    /// Create a new REST client without auth (for testing).
    pub fn unauthenticated() -> Self {
        Self { auth: None }
    }

    fn authed_get(&self, path: &str) -> RestRequest {
        let url = format!("{}{}", RESOURCE_MANAGER, path);
        let mut req = RestRequest::get(url);
        if let Some(ref auth) = self.auth {
            req = req.credential(auth.clone());
        }
        req
    }

    fn authed_post(&self, path: &str) -> RestRequest {
        let url = format!("{}{}", RESOURCE_MANAGER, path);
        let mut req = RestRequest::post(url);
        if let Some(ref auth) = self.auth {
            req = req.credential(auth.clone());
        }
        req
    }
}

impl ResourceManagerService for ResourceManagerRest {
    fn list_projects(&self) -> RestRequest {
        self.authed_get("/v1/projects")
    }

    fn get_project(&self, project: &str) -> RestRequest {
        let path = format!("/v1/projects/{}", project);
        self.authed_get(&path)
    }

    fn get_iam_policy(&self, project: &str) -> RestRequest {
        let path = format!("/v1/projects/{}:getIamPolicy", project);
        self.authed_post(&path)
            .json(serde_json::json!({}))
    }

    fn set_iam_policy(
        &self,
        project: &str,
        policy: serde_json::Value,
    ) -> RestRequest {
        let path = format!("/v1/projects/{}:setIamPolicy", project);
        self.authed_post(&path)
            .json(serde_json::json!({ "policy": policy }))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_projects_url() {
        let svc = ResourceManagerRest::unauthenticated();
        let req = svc.list_projects();
        assert!(req.url.contains("/v1/projects"));
        assert_eq!(req.method, HttpMethod::Get);
    }

    #[test]
    fn test_get_project_url() {
        let svc = ResourceManagerRest::unauthenticated();
        let req = svc.get_project("my-project");
        assert!(req.url.contains("/v1/projects/my-project"));
        assert_eq!(req.method, HttpMethod::Get);
    }

    #[test]
    fn test_get_iam_policy_url() {
        let svc = ResourceManagerRest::unauthenticated();
        let req = svc.get_iam_policy("my-project");
        assert!(req.url.contains(":getIamPolicy"));
        assert_eq!(req.method, HttpMethod::Post);
    }

    #[test]
    fn test_set_iam_policy_url_and_body() {
        let svc = ResourceManagerRest::unauthenticated();
        let policy = serde_json::json!({
            "bindings": [{"role": "roles/viewer", "members": ["user:test@example.com"]}],
            "etag": "abc123"
        });
        let req = svc.set_iam_policy("my-project", policy);
        assert!(req.url.contains(":setIamPolicy"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.unwrap();
        assert!(body["policy"]["bindings"].is_array());
        assert_eq!(body["policy"]["etag"], "abc123");
    }
}
