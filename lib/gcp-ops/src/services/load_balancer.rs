//! Load Balancer service interface.
//!
//! Models global HTTPS load-balancing resources on `compute.googleapis.com`:
//! backend services, URL maps, HTTPS proxies, and forwarding rules.

use super::base_urls::COMPUTE;
use super::{GcpRestClient, MethodMeta};
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// Global HTTPS load balancer service interface.
pub trait LoadBalancerService {
    /// Get a global backend service.
    fn get_backend_service(&self, project: &str, backend_service: &str) -> RestRequest;

    /// Create a global backend service.
    fn create_backend_service(
        &self,
        project: &str,
        backend_service: &str,
        protocol: &str,
        health_check_url: &str,
    ) -> RestRequest;

    /// Get a global URL map.
    fn get_url_map(&self, project: &str, url_map: &str) -> RestRequest;

    /// Create a global URL map.
    fn create_url_map(
        &self,
        project: &str,
        url_map: &str,
        default_service_url: &str,
    ) -> RestRequest;

    /// Get a global target HTTPS proxy.
    fn get_target_https_proxy(&self, project: &str, proxy: &str) -> RestRequest;

    /// Create a global target HTTPS proxy.
    fn create_target_https_proxy(
        &self,
        project: &str,
        proxy: &str,
        url_map_url: &str,
        certificate_urls: &[&str],
    ) -> RestRequest;

    /// Get a global forwarding rule.
    fn get_global_forwarding_rule(&self, project: &str, rule: &str) -> RestRequest;

    /// Create a global forwarding rule.
    fn create_global_forwarding_rule(
        &self,
        project: &str,
        rule: &str,
        target_proxy_url: &str,
        ip_address: &str,
        port_range: &str,
    ) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata
// ---------------------------------------------------------------------------

pub const GET_BACKEND_SERVICE_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/backendServices/{backend_service}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.backendServices.get"],
    service: "compute",
};

pub const CREATE_BACKEND_SERVICE_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/backendServices",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.backendServices.create"],
    service: "compute",
};

pub const GET_URL_MAP_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/urlMaps/{url_map}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.urlMaps.get"],
    service: "compute",
};

pub const CREATE_URL_MAP_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/urlMaps",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.urlMaps.create"],
    service: "compute",
};

pub const GET_TARGET_HTTPS_PROXY_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/targetHttpsProxies/{proxy}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.targetHttpsProxies.get"],
    service: "compute",
};

pub const CREATE_TARGET_HTTPS_PROXY_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/targetHttpsProxies",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.targetHttpsProxies.create"],
    service: "compute",
};

pub const GET_GLOBAL_FORWARDING_RULE_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/forwardingRules/{rule}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["compute.globalForwardingRules.get"],
    service: "compute",
};

pub const CREATE_GLOBAL_FORWARDING_RULE_META: MethodMeta = MethodMeta {
    endpoint: "/compute/v1/projects/{project}/global/forwardingRules",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["compute.globalForwardingRules.create"],
    service: "compute",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LoadBalancerRest {
    auth: Option<Credential>,
}

super::impl_gcp_rest_client_constructors!(LoadBalancerRest);
super::impl_gcp_rest_client!(LoadBalancerRest, COMPUTE);

impl LoadBalancerService for LoadBalancerRest {
    fn get_backend_service(&self, project: &str, backend_service: &str) -> RestRequest {
        self.request_from_meta(
            &GET_BACKEND_SERVICE_META,
            &[("project", project), ("backend_service", backend_service)],
            &[],
        )
    }

    fn create_backend_service(
        &self,
        project: &str,
        backend_service: &str,
        protocol: &str,
        health_check_url: &str,
    ) -> RestRequest {
        self.request_from_meta(&CREATE_BACKEND_SERVICE_META, &[("project", project)], &[])
            .json(serde_json::json!({
                "name": backend_service,
                "protocol": protocol,
                "loadBalancingScheme": "EXTERNAL_MANAGED",
                "healthChecks": [health_check_url]
            }))
    }

    fn get_url_map(&self, project: &str, url_map: &str) -> RestRequest {
        self.request_from_meta(
            &GET_URL_MAP_META,
            &[("project", project), ("url_map", url_map)],
            &[],
        )
    }

    fn create_url_map(
        &self,
        project: &str,
        url_map: &str,
        default_service_url: &str,
    ) -> RestRequest {
        self.request_from_meta(&CREATE_URL_MAP_META, &[("project", project)], &[])
            .json(serde_json::json!({
                "name": url_map,
                "defaultService": default_service_url
            }))
    }

    fn get_target_https_proxy(&self, project: &str, proxy: &str) -> RestRequest {
        self.request_from_meta(
            &GET_TARGET_HTTPS_PROXY_META,
            &[("project", project), ("proxy", proxy)],
            &[],
        )
    }

    fn create_target_https_proxy(
        &self,
        project: &str,
        proxy: &str,
        url_map_url: &str,
        certificate_urls: &[&str],
    ) -> RestRequest {
        self.request_from_meta(
            &CREATE_TARGET_HTTPS_PROXY_META,
            &[("project", project)],
            &[],
        )
        .json(serde_json::json!({
            "name": proxy,
            "urlMap": url_map_url,
            "sslCertificates": certificate_urls
        }))
    }

    fn get_global_forwarding_rule(&self, project: &str, rule: &str) -> RestRequest {
        self.request_from_meta(
            &GET_GLOBAL_FORWARDING_RULE_META,
            &[("project", project), ("rule", rule)],
            &[],
        )
    }

    fn create_global_forwarding_rule(
        &self,
        project: &str,
        rule: &str,
        target_proxy_url: &str,
        ip_address: &str,
        port_range: &str,
    ) -> RestRequest {
        self.request_from_meta(
            &CREATE_GLOBAL_FORWARDING_RULE_META,
            &[("project", project)],
            &[],
        )
        .json(serde_json::json!({
            "name": rule,
            "target": target_proxy_url,
            "IPAddress": ip_address,
            "IPProtocol": "TCP",
            "portRange": port_range,
            "loadBalancingScheme": "EXTERNAL_MANAGED"
        }))
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
        let expected = meta.build_request(COMPUTE, path_params, query_params);
        assert_eq!(req.method, expected.method);
        assert_eq!(req.url, expected.url);
    }

    #[test]
    fn create_backend_service_includes_health_check_reference() {
        let svc = LoadBalancerRest::unauthenticated();
        let req = svc.create_backend_service(
            "proj",
            "web-backend",
            "HTTP",
            "projects/proj/global/healthChecks/web-hc",
        );
        assert_eq!(req.method, HttpMethod::Post);
        assert!(req.url.contains("/global/backendServices"));
        let body = req.body.as_ref().expect("request should include json body");
        assert_eq!(body["name"], "web-backend");
        assert_eq!(
            body["healthChecks"][0],
            "projects/proj/global/healthChecks/web-hc"
        );
        assert_request_matches_meta(
            &req,
            &CREATE_BACKEND_SERVICE_META,
            &[("project", "proj")],
            &[],
        );
    }

    #[test]
    fn create_target_https_proxy_sets_url_map_and_certs() {
        let svc = LoadBalancerRest::unauthenticated();
        let req = svc.create_target_https_proxy(
            "proj",
            "https-proxy",
            "projects/proj/global/urlMaps/web",
            &[
                "projects/proj/global/sslCertificates/cert-a",
                "projects/proj/global/sslCertificates/cert-b",
            ],
        );
        let body = req.body.as_ref().expect("request should include json body");
        assert_eq!(body["name"], "https-proxy");
        assert_eq!(body["sslCertificates"].as_array().map(|a| a.len()), Some(2));
        assert_request_matches_meta(
            &req,
            &CREATE_TARGET_HTTPS_PROXY_META,
            &[("project", "proj")],
            &[],
        );
    }

    #[test]
    fn create_forwarding_rule_sets_target_and_ip() {
        let svc = LoadBalancerRest::unauthenticated();
        let req = svc.create_global_forwarding_rule(
            "proj",
            "fr-web",
            "projects/proj/global/targetHttpsProxies/https-proxy",
            "34.120.1.2",
            "443",
        );
        assert!(req.url.contains("/global/forwardingRules"));
        let body = req.body.as_ref().expect("request should include json body");
        assert_eq!(
            body["target"],
            "projects/proj/global/targetHttpsProxies/https-proxy"
        );
        assert_eq!(body["IPAddress"], "34.120.1.2");
        assert_request_matches_meta(
            &req,
            &CREATE_GLOBAL_FORWARDING_RULE_META,
            &[("project", "proj")],
            &[],
        );
    }

    #[test]
    fn metadata_for_get_url_map_is_read_only() {
        const { assert!(GET_URL_MAP_META.idempotent) };
        const { assert!(GET_URL_MAP_META.read_only) };
        assert_eq!(GET_URL_MAP_META.permissions, &["compute.urlMaps.get"]);
    }

    #[test]
    fn read_requests_match_method_metadata_paths() {
        let svc = LoadBalancerRest::unauthenticated();

        let get_backend = svc.get_backend_service("proj", "backend");
        assert_request_matches_meta(
            &get_backend,
            &GET_BACKEND_SERVICE_META,
            &[("project", "proj"), ("backend_service", "backend")],
            &[],
        );

        let get_url_map = svc.get_url_map("proj", "web-map");
        assert_request_matches_meta(
            &get_url_map,
            &GET_URL_MAP_META,
            &[("project", "proj"), ("url_map", "web-map")],
            &[],
        );

        let get_proxy = svc.get_target_https_proxy("proj", "https-proxy");
        assert_request_matches_meta(
            &get_proxy,
            &GET_TARGET_HTTPS_PROXY_META,
            &[("project", "proj"), ("proxy", "https-proxy")],
            &[],
        );

        let get_rule = svc.get_global_forwarding_rule("proj", "fr-web");
        assert_request_matches_meta(
            &get_rule,
            &GET_GLOBAL_FORWARDING_RULE_META,
            &[("project", "proj"), ("rule", "fr-web")],
            &[],
        );
    }

    #[test]
    fn create_requests_match_method_metadata_paths() {
        let svc = LoadBalancerRest::unauthenticated();

        let create_url_map = svc.create_url_map("proj", "web-map", "backend-url");
        assert_request_matches_meta(
            &create_url_map,
            &CREATE_URL_MAP_META,
            &[("project", "proj")],
            &[],
        );
    }
}
