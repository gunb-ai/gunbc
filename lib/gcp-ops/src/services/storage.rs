//! Cloud Storage service interface.
//!
//! Models the `storage.googleapis.com/storage/v1` API surface
//! for bucket discovery and management.

use super::base_urls::STORAGE;
use super::MethodMeta;
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Cloud Storage service interface.
pub trait StorageService {
    /// List buckets in a project.
    ///
    /// `GET /storage/v1/b?project={project}`
    fn list_buckets(&self, project: &str) -> RestRequest;

    /// Get a specific bucket.
    ///
    /// `GET /storage/v1/b/{bucket}`
    fn get_bucket(&self, bucket: &str) -> RestRequest;

    /// Get the IAM policy for a bucket.
    ///
    /// `GET /storage/v1/b/{bucket}/iam`
    fn get_bucket_iam_policy(&self, bucket: &str) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata
// ---------------------------------------------------------------------------

/// Metadata for `list_buckets`.
pub const LIST_BUCKETS_META: MethodMeta = MethodMeta {
    endpoint: "/storage/v1/b",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["storage.buckets.list"],
    service: "storage",
};

/// Metadata for `get_bucket`.
pub const GET_BUCKET_META: MethodMeta = MethodMeta {
    endpoint: "/storage/v1/b/{bucket}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["storage.buckets.get"],
    service: "storage",
};

/// Metadata for `get_bucket_iam_policy`.
pub const GET_BUCKET_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/storage/v1/b/{bucket}/iam",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["storage.buckets.getIamPolicy"],
    service: "storage",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

/// REST-based implementation of the Cloud Storage service.
#[derive(Debug, Clone)]
pub struct StorageRest {
    auth: Option<Credential>,
}

impl StorageRest {
    /// Create a new REST client with the given auth credential.
    pub fn new(auth: Credential) -> Self {
        Self { auth: Some(auth) }
    }

    /// Create a new REST client without auth (for testing).
    pub fn unauthenticated() -> Self {
        Self { auth: None }
    }

    fn authed_get(&self, path: &str) -> RestRequest {
        let url = format!("{}{}", STORAGE, path);
        let mut req = RestRequest::get(url);
        if let Some(ref auth) = self.auth {
            req = req.credential(auth.clone());
        }
        req
    }
}

impl StorageService for StorageRest {
    fn list_buckets(&self, project: &str) -> RestRequest {
        self.authed_get("/storage/v1/b")
            .query("project", project)
    }

    fn get_bucket(&self, bucket: &str) -> RestRequest {
        let path = format!("/storage/v1/b/{}", bucket);
        self.authed_get(&path)
    }

    fn get_bucket_iam_policy(&self, bucket: &str) -> RestRequest {
        let path = format!("/storage/v1/b/{}/iam", bucket);
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
    fn test_list_buckets_url() {
        let svc = StorageRest::unauthenticated();
        let req = svc.list_buckets("my-project");
        assert!(req.url.contains("/storage/v1/b"));
        assert!(req.query.contains_key("project"));
        assert_eq!(req.query["project"], "my-project");
    }

    #[test]
    fn test_get_bucket_url() {
        let svc = StorageRest::unauthenticated();
        let req = svc.get_bucket("my-bucket");
        assert!(req.url.contains("/storage/v1/b/my-bucket"));
    }

    #[test]
    fn test_get_bucket_iam_policy_url() {
        let svc = StorageRest::unauthenticated();
        let req = svc.get_bucket_iam_policy("my-bucket");
        assert!(req.url.contains("/storage/v1/b/my-bucket/iam"));
    }
}
