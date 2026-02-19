//! Cloud Run service interface.
//!
//! Models a focused subset of `run.googleapis.com/v2` for service lifecycle
//! and IAM policy management.

use super::base_urls::RUN;
use super::{GcpRestClient, MethodMeta};
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Cloud Run service interface.
pub trait CloudRunService {
    /// List services in a region.
    fn list_services(&self, project: &str, region: &str) -> RestRequest;

    /// Get one service in a region.
    fn get_service(&self, project: &str, region: &str, service: &str) -> RestRequest;

    /// Create a new service.
    fn create_service(
        &self,
        project: &str,
        region: &str,
        service: &str,
        image: &str,
        service_account: &str,
        env: &[(&str, &str)],
    ) -> RestRequest;

    /// Patch an existing service.
    fn update_service(
        &self,
        project: &str,
        region: &str,
        service: &str,
        image: &str,
        service_account: &str,
        env: &[(&str, &str)],
    ) -> RestRequest;

    /// Get service IAM policy.
    fn get_service_iam_policy(&self, project: &str, region: &str, service: &str) -> RestRequest;

    /// Set service IAM policy.
    fn set_service_iam_policy(
        &self,
        project: &str,
        region: &str,
        service: &str,
        policy: serde_json::Value,
    ) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata
// ---------------------------------------------------------------------------

pub const LIST_SERVICES_META: MethodMeta = MethodMeta {
    endpoint: "/v2/projects/{project}/locations/{region}/services",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["run.services.list"],
    service: "run",
};

pub const GET_SERVICE_META: MethodMeta = MethodMeta {
    endpoint: "/v2/projects/{project}/locations/{region}/services/{service}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["run.services.get"],
    service: "run",
};

pub const CREATE_SERVICE_META: MethodMeta = MethodMeta {
    endpoint: "/v2/projects/{project}/locations/{region}/services",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["run.services.create"],
    service: "run",
};

pub const UPDATE_SERVICE_META: MethodMeta = MethodMeta {
    endpoint: "/v2/projects/{project}/locations/{region}/services/{service}",
    http_method: HttpMethod::Patch,
    idempotent: true,
    read_only: false,
    permissions: &["run.services.update"],
    service: "run",
};

pub const GET_SERVICE_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/v2/projects/{project}/locations/{region}/services/{service}:getIamPolicy",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["run.services.getIamPolicy"],
    service: "run",
};

pub const SET_SERVICE_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/v2/projects/{project}/locations/{region}/services/{service}:setIamPolicy",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["run.services.setIamPolicy"],
    service: "run",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CloudRunRest {
    auth: Option<Credential>,
}

impl CloudRunRest {
    fn service_template_body(
        image: &str,
        service_account: &str,
        env: &[(&str, &str)],
    ) -> serde_json::Value {
        let env_json: Vec<serde_json::Value> = env
            .iter()
            .map(|(name, value)| serde_json::json!({ "name": name, "value": value }))
            .collect();
        serde_json::json!({
            "template": {
                "serviceAccount": service_account,
                "containers": [{
                    "image": image,
                    "env": env_json,
                }]
            }
        })
    }
}

super::impl_gcp_rest_client_constructors!(CloudRunRest);
super::impl_gcp_rest_client!(CloudRunRest, RUN);

impl CloudRunService for CloudRunRest {
    fn list_services(&self, project: &str, region: &str) -> RestRequest {
        self.request_from_meta(
            &LIST_SERVICES_META,
            &[("project", project), ("region", region)],
            &[],
        )
    }

    fn get_service(&self, project: &str, region: &str, service: &str) -> RestRequest {
        self.request_from_meta(
            &GET_SERVICE_META,
            &[
                ("project", project),
                ("region", region),
                ("service", service),
            ],
            &[],
        )
    }

    fn create_service(
        &self,
        project: &str,
        region: &str,
        service: &str,
        image: &str,
        service_account: &str,
        env: &[(&str, &str)],
    ) -> RestRequest {
        self.request_from_meta(
            &CREATE_SERVICE_META,
            &[("project", project), ("region", region)],
            &[("serviceId", service)],
        )
        .json(Self::service_template_body(image, service_account, env))
    }

    fn update_service(
        &self,
        project: &str,
        region: &str,
        service: &str,
        image: &str,
        service_account: &str,
        env: &[(&str, &str)],
    ) -> RestRequest {
        self.request_from_meta(
            &UPDATE_SERVICE_META,
            &[
                ("project", project),
                ("region", region),
                ("service", service),
            ],
            &[("updateMask", "template,labels")],
        )
        .json(Self::service_template_body(image, service_account, env))
    }

    fn get_service_iam_policy(&self, project: &str, region: &str, service: &str) -> RestRequest {
        self.request_from_meta(
            &GET_SERVICE_IAM_POLICY_META,
            &[
                ("project", project),
                ("region", region),
                ("service", service),
            ],
            &[],
        )
    }

    fn set_service_iam_policy(
        &self,
        project: &str,
        region: &str,
        service: &str,
        policy: serde_json::Value,
    ) -> RestRequest {
        self.request_from_meta(
            &SET_SERVICE_IAM_POLICY_META,
            &[
                ("project", project),
                ("region", region),
                ("service", service),
            ],
            &[],
        )
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
        let expected = meta.build_request(RUN, path_params, query_params);
        assert_eq!(req.method, expected.method);
        assert_eq!(req.url, expected.url);
    }

    #[test]
    fn create_service_sets_service_id_and_template_body() {
        let svc = CloudRunRest::unauthenticated();
        let req = svc.create_service(
            "proj",
            "us-central1",
            "api",
            "us-docker.pkg.dev/proj/repo/api:latest",
            "run@proj.iam.gserviceaccount.com",
            &[("ENV", "dev"), ("PORT", "8080")],
        );
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req
            .url
            .contains("/v2/projects/proj/locations/us-central1/services"));
        // `build_request` appends query params to the URL instead of the `RestRequest.query` map.
        assert!(req.url.ends_with("?serviceId=api") || req.url.contains("?serviceId=api&"));
        let body = req.body.as_ref().expect("request should include json body");
        assert_eq!(
            body["template"]["containers"][0]["image"],
            "us-docker.pkg.dev/proj/repo/api:latest"
        );
        assert_request_matches_meta(
            &req,
            &CREATE_SERVICE_META,
            &[("project", "proj"), ("region", "us-central1")],
            &[("serviceId", "api")],
        );
    }

    #[test]
    fn update_service_uses_patch_with_update_mask() {
        let svc = CloudRunRest::unauthenticated();
        let req = svc.update_service(
            "proj",
            "us-central1",
            "api",
            "img",
            "run@proj.iam.gserviceaccount.com",
            &[],
        );
        assert_eq!(req.method, HttpMethod::Patch);
        // `build_request` appends query params to the URL instead of the `RestRequest.query` map.
        assert!(
            req.url.ends_with("?updateMask=template%2Clabels")
                || req.url.contains("?updateMask=template%2Clabels&")
        );
        assert_request_matches_meta(
            &req,
            &UPDATE_SERVICE_META,
            &[
                ("project", "proj"),
                ("region", "us-central1"),
                ("service", "api"),
            ],
            &[("updateMask", "template,labels")],
        );
    }

    #[test]
    fn metadata_marks_service_get_iam_as_read_only() {
        const { assert!(GET_SERVICE_IAM_POLICY_META.read_only) };
        const { assert!(GET_SERVICE_IAM_POLICY_META.idempotent) };
        assert_eq!(
            GET_SERVICE_IAM_POLICY_META.permissions,
            &["run.services.getIamPolicy"]
        );
    }

    #[test]
    fn read_requests_match_method_metadata_paths() {
        let svc = CloudRunRest::unauthenticated();

        let list = svc.list_services("proj", "us-central1");
        assert_request_matches_meta(
            &list,
            &LIST_SERVICES_META,
            &[("project", "proj"), ("region", "us-central1")],
            &[],
        );

        let get = svc.get_service("proj", "us-central1", "api");
        assert_request_matches_meta(
            &get,
            &GET_SERVICE_META,
            &[
                ("project", "proj"),
                ("region", "us-central1"),
                ("service", "api"),
            ],
            &[],
        );

        let get_iam = svc.get_service_iam_policy("proj", "us-central1", "api");
        assert_request_matches_meta(
            &get_iam,
            &GET_SERVICE_IAM_POLICY_META,
            &[
                ("project", "proj"),
                ("region", "us-central1"),
                ("service", "api"),
            ],
            &[],
        );
    }
}
