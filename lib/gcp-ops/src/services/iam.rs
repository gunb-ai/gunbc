//! IAM service interface.
//!
//! Models the `iam.googleapis.com/v1` API surface for service account
//! management and the `iamcredentials.googleapis.com/v1` API for
//! token generation (impersonation).

use super::base_urls::{IAM, IAM_CREDENTIALS};
use super::MethodMeta;
use gunbc_ir::transport::credential::Credential;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::rest::RestRequest;

// ---------------------------------------------------------------------------
// Service trait
// ---------------------------------------------------------------------------

/// IAM service interface for service account and token operations.
pub trait IamService {
    /// List service accounts in a project.
    ///
    /// `GET /v1/projects/{project}/serviceAccounts`
    fn list_service_accounts(&self, project: &str) -> RestRequest;

    /// Get a specific service account.
    ///
    /// `GET /v1/projects/{project}/serviceAccounts/{email}`
    fn get_service_account(&self, project: &str, email: &str) -> RestRequest;

    /// Generate an access token for a service account (impersonation).
    ///
    /// `POST /v1/projects/-/serviceAccounts/{email}:generateAccessToken`
    fn generate_access_token(
        &self,
        service_account_email: &str,
        scopes: &[&str],
        lifetime_seconds: Option<i64>,
    ) -> RestRequest;

    /// Exchange a subject token for a federated access token via STS.
    ///
    /// `POST /v1/token` on sts.googleapis.com
    fn exchange_token(
        &self,
        audience: &str,
        subject_token: &str,
        subject_token_type: &str,
    ) -> RestRequest;
}

// ---------------------------------------------------------------------------
// Method metadata
// ---------------------------------------------------------------------------

/// Metadata for `list_service_accounts`.
pub const LIST_SERVICE_ACCOUNTS_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/serviceAccounts",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["iam.serviceAccounts.list"],
    service: "iam",
};

/// Metadata for `get_service_account`.
pub const GET_SERVICE_ACCOUNT_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/serviceAccounts/{email}",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["iam.serviceAccounts.get"],
    service: "iam",
};

/// Metadata for `generate_access_token`.
pub const GENERATE_ACCESS_TOKEN_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/-/serviceAccounts/{email}:generateAccessToken",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["iam.serviceAccounts.getAccessToken"],
    service: "iamcredentials",
};

/// Metadata for `exchange_token`.
pub const EXCHANGE_TOKEN_META: MethodMeta = MethodMeta {
    endpoint: "/v1/token",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &[],
    service: "sts",
};

// ---------------------------------------------------------------------------
// REST implementation
// ---------------------------------------------------------------------------

/// REST-based implementation of the IAM service.
#[derive(Debug, Clone)]
pub struct IamRest {
    auth: Option<Credential>,
}

impl IamRest {
    /// Create a new REST client with the given auth credential.
    pub fn new(auth: Credential) -> Self {
        Self { auth: Some(auth) }
    }

    /// Create a new REST client without auth (for testing).
    pub fn unauthenticated() -> Self {
        Self { auth: None }
    }

    fn authed_get(&self, base: &str, path: &str) -> RestRequest {
        let url = format!("{}{}", base, path);
        let mut req = RestRequest::get(url);
        if let Some(ref auth) = self.auth {
            req = req.credential(auth.clone());
        }
        req
    }

    fn authed_post(&self, base: &str, path: &str) -> RestRequest {
        let url = format!("{}{}", base, path);
        let mut req = RestRequest::post(url);
        if let Some(ref auth) = self.auth {
            req = req.credential(auth.clone());
        }
        req
    }
}

impl IamService for IamRest {
    fn list_service_accounts(&self, project: &str) -> RestRequest {
        let path = format!("/v1/projects/{}/serviceAccounts", project);
        self.authed_get(IAM, &path)
    }

    fn get_service_account(&self, project: &str, email: &str) -> RestRequest {
        let path = format!("/v1/projects/{}/serviceAccounts/{}", project, email);
        self.authed_get(IAM, &path)
    }

    fn generate_access_token(
        &self,
        service_account_email: &str,
        scopes: &[&str],
        lifetime_seconds: Option<i64>,
    ) -> RestRequest {
        let path = format!(
            "/v1/projects/-/serviceAccounts/{}:generateAccessToken",
            service_account_email
        );
        let mut body = serde_json::json!({
            "scope": scopes,
        });
        if let Some(lifetime) = lifetime_seconds {
            body["lifetime"] = serde_json::json!(format!("{}s", lifetime));
        }
        self.authed_post(IAM_CREDENTIALS, &path).json(body)
    }

    fn exchange_token(
        &self,
        audience: &str,
        subject_token: &str,
        subject_token_type: &str,
    ) -> RestRequest {
        let url = format!("{}/v1/token", super::base_urls::STS);
        RestRequest::post(url).json(serde_json::json!({
            "grantType": "urn:ietf:params:oauth:grant-type:token-exchange",
            "audience": format!("//iam.googleapis.com/{}", audience),
            "scope": "https://www.googleapis.com/auth/cloud-platform",
            "requestedTokenType": "urn:ietf:params:oauth:token-type:access_token",
            "subjectToken": subject_token,
            "subjectTokenType": subject_token_type,
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
    fn test_list_service_accounts_url() {
        let svc = IamRest::unauthenticated();
        let req = svc.list_service_accounts("my-project");
        assert!(req.url.contains("/v1/projects/my-project/serviceAccounts"));
        assert_eq!(req.method, HttpMethod::Get);
    }

    #[test]
    fn test_get_service_account_url() {
        let svc = IamRest::unauthenticated();
        let req = svc.get_service_account("my-project", "sa@project.iam.gserviceaccount.com");
        assert!(req
            .url
            .contains("serviceAccounts/sa@project.iam.gserviceaccount.com"));
    }

    #[test]
    fn test_generate_access_token_body() {
        let svc = IamRest::unauthenticated();
        let req = svc.generate_access_token(
            "sa@project.iam.gserviceaccount.com",
            &["https://www.googleapis.com/auth/cloud-platform"],
            Some(3600),
        );
        assert!(req.url.contains(":generateAccessToken"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.unwrap();
        assert!(body["lifetime"].as_str().unwrap().contains("3600s"));
    }

    #[test]
    fn test_exchange_token_url() {
        let svc = IamRest::unauthenticated();
        let req = svc.exchange_token(
            "projects/123/locations/global/workloadIdentityPools/pool/providers/prov",
            "subject-token",
            "urn:ietf:params:oauth:token-type:jwt",
        );
        assert!(req.url.contains("sts.googleapis.com/v1/token"));
        assert_eq!(req.method, HttpMethod::Post);
    }
}
