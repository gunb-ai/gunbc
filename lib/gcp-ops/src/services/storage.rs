//! Cloud Storage service interface.
//!
//! Models the `storage.googleapis.com/storage/v1` API surface
//! for bucket discovery and management.

use super::base_urls::STORAGE;
use super::{GcpRestClient, MethodMeta};
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

    /// Create a bucket in a project.
    ///
    /// `POST /storage/v1/b?project={project}`
    fn create_bucket(
        &self,
        project: &str,
        bucket: &str,
        location: &str,
        storage_class: &str,
    ) -> RestRequest;

    /// Set the IAM policy for a bucket.
    ///
    /// `PUT /storage/v1/b/{bucket}/iam`
    fn set_bucket_iam_policy(&self, bucket: &str, policy: serde_json::Value) -> RestRequest;
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

/// Metadata for `create_bucket`.
pub const CREATE_BUCKET_META: MethodMeta = MethodMeta {
    endpoint: "/storage/v1/b",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["storage.buckets.create"],
    service: "storage",
};

/// Metadata for `set_bucket_iam_policy`.
pub const SET_BUCKET_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/storage/v1/b/{bucket}/iam",
    http_method: HttpMethod::Put,
    idempotent: true,
    read_only: false,
    permissions: &["storage.buckets.setIamPolicy"],
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

super::impl_gcp_rest_client_constructors!(StorageRest);
super::impl_gcp_rest_client!(StorageRest, STORAGE);

impl StorageService for StorageRest {
    fn list_buckets(&self, project: &str) -> RestRequest {
        self.request_from_meta(&LIST_BUCKETS_META, &[], &[("project", project)])
    }

    fn get_bucket(&self, bucket: &str) -> RestRequest {
        self.request_from_meta(&GET_BUCKET_META, &[("bucket", bucket)], &[])
    }

    fn get_bucket_iam_policy(&self, bucket: &str) -> RestRequest {
        self.request_from_meta(&GET_BUCKET_IAM_POLICY_META, &[("bucket", bucket)], &[])
    }

    fn create_bucket(
        &self,
        project: &str,
        bucket: &str,
        location: &str,
        storage_class: &str,
    ) -> RestRequest {
        self.request_from_meta(&CREATE_BUCKET_META, &[], &[("project", project)])
            .json(serde_json::json!({
                "name": bucket,
                "location": location,
                "storageClass": storage_class
            }))
    }

    fn set_bucket_iam_policy(&self, bucket: &str, policy: serde_json::Value) -> RestRequest {
        self.request_from_meta(&SET_BUCKET_IAM_POLICY_META, &[("bucket", bucket)], &[])
            .json(serde_json::json!({ "policy": policy }))
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
        let expected = meta.build_request(STORAGE, path_params, query_params);
        assert_eq!(req.method, expected.method);
        assert_eq!(req.url, expected.url);
    }

    #[test]
    fn test_list_buckets_url() {
        let svc = StorageRest::unauthenticated();
        let req = svc.list_buckets("my-project");
        assert!(req.url.contains("/storage/v1/b"));
        // `build_request` appends query params to the URL instead of the `RestRequest.query` map.
        assert!(
            req.url.ends_with("?project=my-project") || req.url.contains("?project=my-project&")
        );
        assert_request_matches_meta(&req, &LIST_BUCKETS_META, &[], &[("project", "my-project")]);
    }

    #[test]
    fn test_get_bucket_url() {
        let svc = StorageRest::unauthenticated();
        let req = svc.get_bucket("my-bucket");
        assert!(req.url.contains("/storage/v1/b/my-bucket"));
        assert_request_matches_meta(&req, &GET_BUCKET_META, &[("bucket", "my-bucket")], &[]);
    }

    #[test]
    fn test_get_bucket_iam_policy_url() {
        let svc = StorageRest::unauthenticated();
        let req = svc.get_bucket_iam_policy("my-bucket");
        assert!(req.url.contains("/storage/v1/b/my-bucket/iam"));
        assert_request_matches_meta(
            &req,
            &GET_BUCKET_IAM_POLICY_META,
            &[("bucket", "my-bucket")],
            &[],
        );
    }

    #[test]
    fn test_create_bucket_request() {
        let svc = StorageRest::unauthenticated();
        let req = svc.create_bucket("my-project", "my-bucket", "US", "STANDARD");
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.url.contains("/storage/v1/b"));
        // `build_request` appends query params to the URL instead of the `RestRequest.query` map.
        assert!(
            req.url.ends_with("?project=my-project") || req.url.contains("?project=my-project&")
        );
        let body = req.body.as_ref().expect("request should include json body");
        assert_eq!(body["name"], "my-bucket");
        assert_eq!(body["location"], "US");
        assert_eq!(body["storageClass"], "STANDARD");
        assert_request_matches_meta(&req, &CREATE_BUCKET_META, &[], &[("project", "my-project")]);
    }

    #[test]
    fn test_set_bucket_iam_policy_request() {
        let svc = StorageRest::unauthenticated();
        let req = svc.set_bucket_iam_policy(
            "my-bucket",
            serde_json::json!({
                "bindings": [{
                    "role": "roles/storage.objectViewer",
                    "members": ["allAuthenticatedUsers"]
                }]
            }),
        );
        assert_eq!(req.method, HttpMethod::Put);
        assert!(req.url.contains("/storage/v1/b/my-bucket/iam"));
        let body = req.body.as_ref().expect("request should include json body");
        assert!(body.get("policy").is_some());
        assert_request_matches_meta(
            &req,
            &SET_BUCKET_IAM_POLICY_META,
            &[("bucket", "my-bucket")],
            &[],
        );
    }
}
