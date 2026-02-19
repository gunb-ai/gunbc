//! Compute Engine service interface.
//!
//! Models a focused subset of `compute.googleapis.com` used by infra
//! bootstrap and rollout DAGs (instance templates, managed instance groups,
//! and health checks).

use super::base_urls::COMPUTE;
use super::{GcpRestClient, MethodMeta};
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Compute Engine service interface.
pub trait ComputeEngineService {
    /// List global instance templates.
    fn list_instance_templates(&self, project: &str) -> RestRequest;

    /// Get one global instance template.
    fn get_instance_template(&self, project: &str, template: &str) -> RestRequest;

    /// Create a global instance template.
    fn create_instance_template(
        &self,
        project: &str,
        template: &str,
        machine_type: &str,
        source_image: &str,
        service_account_email: &str,
    ) -> RestRequest;

    /// Get a zonal managed instance group (MIG).
    fn get_instance_group_manager(&self, project: &str, zone: &str, manager: &str) -> RestRequest;

    /// Create a zonal managed instance group (MIG).
    fn create_instance_group_manager(
        &self,
        project: &str,
        zone: &str,
        manager: &str,
        template: &str,
        target_size: i64,
    ) -> RestRequest;

    /// Resize a zonal managed instance group (MIG).
    fn resize_instance_group_manager(
        &self,
        project: &str,
        zone: &str,
        manager: &str,
        target_size: i64,
    ) -> RestRequest;

    /// Get a global health check.
    fn get_health_check(&self, project: &str, health_check: &str) -> RestRequest;

    /// Create a global HTTP health check.
    fn create_health_check(
        &self,
        project: &str,
        health_check: &str,
        port: i64,
        request_path: &str,
    ) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata
// ---------------------------------------------------------------------------

pub const LIST_INSTANCE_TEMPLATES_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/instanceTemplates",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.instanceTemplates.list"],
    service: "compute",
};

pub const GET_INSTANCE_TEMPLATE_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/instanceTemplates/{template}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.instanceTemplates.get"],
    service: "compute",
};

pub const CREATE_INSTANCE_TEMPLATE_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/instanceTemplates",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.instanceTemplates.create"],
    service: "compute",
};

pub const GET_INSTANCE_GROUP_MANAGER_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/zones/{zone}/instanceGroupManagers/{manager}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.instanceGroupManagers.get"],
    service: "compute",
};

pub const CREATE_INSTANCE_GROUP_MANAGER_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/zones/{zone}/instanceGroupManagers",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.instanceGroupManagers.create"],
    service: "compute",
};

pub const RESIZE_INSTANCE_GROUP_MANAGER_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/zones/{zone}/instanceGroupManagers/{manager}/resize",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.instanceGroupManagers.update"],
    service: "compute",
};

pub const GET_HEALTH_CHECK_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/healthChecks/{health_check}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.healthChecks.get"],
    service: "compute",
};

pub const CREATE_HEALTH_CHECK_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/healthChecks",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.healthChecks.create"],
    service: "compute",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ComputeEngineRest {
    auth: Option<Credential>,
}

super::impl_gcp_rest_client_constructors!(ComputeEngineRest);
super::impl_gcp_rest_client!(ComputeEngineRest, COMPUTE);

impl ComputeEngineService for ComputeEngineRest {
    fn list_instance_templates(&self, project: &str) -> RestRequest {
        let mut req = LIST_INSTANCE_TEMPLATES_META.build_request(
            self.base_url(),
            &[("project", project)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn get_instance_template(&self, project: &str, template: &str) -> RestRequest {
        let mut req = GET_INSTANCE_TEMPLATE_META.build_request(
            self.base_url(),
            &[("project", project), ("template", template)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn create_instance_template(
        &self,
        project: &str,
        template: &str,
        machine_type: &str,
        source_image: &str,
        service_account_email: &str,
    ) -> RestRequest {
        let mut req = CREATE_INSTANCE_TEMPLATE_META.build_request(
            self.base_url(),
            &[("project", project)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req.json(serde_json::json!({
            "name": template,
            "properties": {
                "machineType": machine_type,
                "disks": [{
                    "boot": true,
                    "autoDelete": true,
                    "initializeParams": { "sourceImage": source_image }
                }],
                "serviceAccounts": [{
                    "email": service_account_email,
                    "scopes": ["https://www.googleapis.com/auth/cloud-platform"]
                }]
            }
        }))
    }

    fn get_instance_group_manager(&self, project: &str, zone: &str, manager: &str) -> RestRequest {
        let mut req = GET_INSTANCE_GROUP_MANAGER_META.build_request(
            self.base_url(),
            &[
                ("project", project),
                ("zone", zone),
                ("manager", manager),
            ],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn create_instance_group_manager(
        &self,
        project: &str,
        zone: &str,
        manager: &str,
        template: &str,
        target_size: i64,
    ) -> RestRequest {
        let template_ref = format!("projects/{project}/global/instanceTemplates/{template}");
        let mut req = CREATE_INSTANCE_GROUP_MANAGER_META.build_request(
            self.base_url(),
            &[("project", project), ("zone", zone)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req.json(serde_json::json!({
            "name": manager,
            "baseInstanceName": manager,
            "instanceTemplate": template_ref,
            "targetSize": target_size
        }))
    }

    fn resize_instance_group_manager(
        &self,
        project: &str,
        zone: &str,
        manager: &str,
        target_size: i64,
    ) -> RestRequest {
        let target_size_str = target_size.to_string();
        let mut req = RESIZE_INSTANCE_GROUP_MANAGER_META.build_request(
            self.base_url(),
            &[
                ("project", project),
                ("zone", zone),
                ("manager", manager),
            ],
            &[("size", &target_size_str)],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn get_health_check(&self, project: &str, health_check: &str) -> RestRequest {
        let mut req = GET_HEALTH_CHECK_META.build_request(
            self.base_url(),
            &[("project", project), ("health_check", health_check)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn create_health_check(
        &self,
        project: &str,
        health_check: &str,
        port: i64,
        request_path: &str,
    ) -> RestRequest {
        let mut req = CREATE_HEALTH_CHECK_META.build_request(
            self.base_url(),
            &[("project", project)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req.json(serde_json::json!({
            "name": health_check,
            "type": "HTTP",
            "httpHealthCheck": {
                "port": port,
                "requestPath": request_path
            }
        }))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_instance_template_builds_expected_request() {
        let svc = ComputeEngineRest::unauthenticated();
        let req = svc.create_instance_template(
            "proj",
            "tmpl-a",
            "e2-small",
            "projects/debian-cloud/global/images/family/debian-12",
            "compute@proj.iam.gserviceaccount.com",
        );
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req
            .url
            .contains("/compute/v1/projects/proj/global/instanceTemplates"));
        let body = req.body.expect("request should include json body");
        assert_eq!(body["name"], "tmpl-a");
        assert_eq!(body["properties"]["machineType"], "e2-small");
    }

    #[test]
    fn create_mig_uses_instance_template_self_link() {
        let svc = ComputeEngineRest::unauthenticated();
        let req =
            svc.create_instance_group_manager("proj", "us-central1-a", "web-mig", "tmpl-a", 2);
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.expect("request should include json body");
        assert_eq!(
            body["instanceTemplate"],
            "projects/proj/global/instanceTemplates/tmpl-a"
        );
        assert_eq!(body["targetSize"], 2);
    }

    #[test]
    fn resize_mig_sends_size_query_parameter() {
        let svc = ComputeEngineRest::unauthenticated();
        let req = svc.resize_instance_group_manager("proj", "us-central1-a", "web-mig", 5);
        assert!(req.url.contains("/instanceGroupManagers/web-mig/resize"));
        // `build_request` appends query params to the URL instead of the `RestRequest.query` map.
        assert!(req.url.ends_with("?size=5") || req.url.contains("?size=5&"));
    }

    #[test]
    fn metadata_marks_health_check_create_as_idempotent_write() {
        const { assert!(CREATE_HEALTH_CHECK_META.idempotent) };
        const { assert!(!CREATE_HEALTH_CHECK_META.read_only) };
        assert_eq!(
            CREATE_HEALTH_CHECK_META.permissions,
            &["compute.healthChecks.create"]
        );
    }
}
