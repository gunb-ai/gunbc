//! IAM service interface.
//!
//! Models the `iam.googleapis.com/v1` API surface for service account
//! management and the `iamcredentials.googleapis.com/v1` API for
//! token generation (impersonation).

use super::base_urls::{IAM, IAM_CREDENTIALS};
use super::{GcpRestClient, MethodMeta};
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

    /// Create a service account.
    ///
    /// `POST /v1/projects/{project}/serviceAccounts`
    fn create_service_account(
        &self,
        project: &str,
        account_id: &str,
        display_name: &str,
    ) -> RestRequest;

    /// Update a service account display name.
    ///
    /// `PATCH /v1/projects/{project}/serviceAccounts/{email}?updateMask=displayName`
    fn update_service_account(&self, project: &str, email: &str, display_name: &str)
        -> RestRequest;

    /// Delete a service account.
    ///
    /// `DELETE /v1/projects/{project}/serviceAccounts/{email}`
    fn delete_service_account(&self, project: &str, email: &str) -> RestRequest;

    /// Get service-account-level IAM policy (impersonation bindings).
    ///
    /// `POST /v1/projects/{project}/serviceAccounts/{email}:getIamPolicy`
    fn get_service_account_iam_policy(&self, project: &str, email: &str) -> RestRequest;

    /// Set service-account-level IAM policy (impersonation bindings).
    ///
    /// `POST /v1/projects/{project}/serviceAccounts/{email}:setIamPolicy`
    fn set_service_account_iam_policy(
        &self,
        project: &str,
        email: &str,
        policy: serde_json::Value,
    ) -> RestRequest;

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

/// Metadata for `create_service_account`.
pub const CREATE_SERVICE_ACCOUNT_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/serviceAccounts",
    http_method: HttpMethod::Post,
    idempotent: false,
    read_only: false,
    permissions: &["iam.serviceAccounts.create"],
    service: "iam",
};

/// Metadata for `update_service_account`.
pub const UPDATE_SERVICE_ACCOUNT_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/serviceAccounts/{email}",
    http_method: HttpMethod::Patch,
    idempotent: true,
    read_only: false,
    permissions: &["iam.serviceAccounts.update"],
    service: "iam",
};

/// Metadata for `delete_service_account`.
pub const DELETE_SERVICE_ACCOUNT_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/serviceAccounts/{email}",
    http_method: HttpMethod::Delete,
    idempotent: true,
    read_only: false,
    permissions: &["iam.serviceAccounts.delete"],
    service: "iam",
};

/// Metadata for `get_service_account_iam_policy`.
pub const GET_SERVICE_ACCOUNT_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/serviceAccounts/{email}:getIamPolicy",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: true,
    permissions: &["iam.serviceAccounts.getIamPolicy"],
    service: "iam",
};

/// Metadata for `set_service_account_iam_policy`.
pub const SET_SERVICE_ACCOUNT_IAM_POLICY_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/serviceAccounts/{email}:setIamPolicy",
    http_method: HttpMethod::Post,
    idempotent: true,
    read_only: false,
    permissions: &["iam.serviceAccounts.setIamPolicy"],
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

super::impl_gcp_rest_client_constructors!(IamRest);
super::impl_gcp_rest_client!(IamRest, IAM);

impl IamService for IamRest {
    fn list_service_accounts(&self, project: &str) -> RestRequest {
        let mut req = LIST_SERVICE_ACCOUNTS_META.build_request(
            self.base_url(),
            &[("project", project)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn get_service_account(&self, project: &str, email: &str) -> RestRequest {
        let mut req = GET_SERVICE_ACCOUNT_META.build_request(
            self.base_url(),
            &[("project", project), ("email", email)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn create_service_account(
        &self,
        project: &str,
        account_id: &str,
        display_name: &str,
    ) -> RestRequest {
        let mut req = CREATE_SERVICE_ACCOUNT_META.build_request(
            self.base_url(),
            &[("project", project)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req.json(serde_json::json!({
            "accountId": account_id,
            "serviceAccount": {
                "displayName": display_name
            }
        }))
    }

    fn update_service_account(
        &self,
        project: &str,
        email: &str,
        display_name: &str,
    ) -> RestRequest {
        let mut req = UPDATE_SERVICE_ACCOUNT_META.build_request(
            self.base_url(),
            &[("project", project), ("email", email)],
            &[("updateMask", "displayName")],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req.json(serde_json::json!({ "displayName": display_name }))
    }

    fn delete_service_account(&self, project: &str, email: &str) -> RestRequest {
        let mut req = DELETE_SERVICE_ACCOUNT_META.build_request(
            self.base_url(),
            &[("project", project), ("email", email)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn get_service_account_iam_policy(&self, project: &str, email: &str) -> RestRequest {
        let mut req = GET_SERVICE_ACCOUNT_IAM_POLICY_META.build_request(
            self.base_url(),
            &[("project", project), ("email", email)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req
    }

    fn set_service_account_iam_policy(
        &self,
        project: &str,
        email: &str,
        policy: serde_json::Value,
    ) -> RestRequest {
        let mut req = SET_SERVICE_ACCOUNT_IAM_POLICY_META.build_request(
            self.base_url(),
            &[("project", project), ("email", email)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        req.json(serde_json::json!({ "policy": policy }))
    }

    fn generate_access_token(
        &self,
        service_account_email: &str,
        scopes: &[&str],
        lifetime_seconds: Option<i64>,
    ) -> RestRequest {
        let mut req = GENERATE_ACCESS_TOKEN_META.build_request(
            IAM_CREDENTIALS,
            &[("email", service_account_email)],
            &[],
        );
        if let Some(auth) = self.credential() {
            req = req.credential(auth.clone());
        }
        let mut body = serde_json::json!({
            "scope": scopes,
        });
        if let Some(lifetime) = lifetime_seconds {
            body["lifetime"] = serde_json::json!(format!("{}s", lifetime));
        }
        req.json(body)
    }

    fn exchange_token(
        &self,
        audience: &str,
        subject_token: &str,
        subject_token_type: &str,
    ) -> RestRequest {
        let req = EXCHANGE_TOKEN_META.build_request(
            super::base_urls::STS,
            &[],
            &[],
        );
        // exchange_token doesn't use standard auth headers
        req.json(serde_json::json!({
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
    fn test_create_service_account_request_shape() {
        let svc = IamRest::unauthenticated();
        let req = svc.create_service_account("my-project", "new-sa", "New SA");
        assert!(req.url.contains("/v1/projects/my-project/serviceAccounts"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.expect("create request should include json body");
        assert_eq!(body["accountId"].as_str(), Some("new-sa"));
        assert_eq!(
            body["serviceAccount"]["displayName"].as_str(),
            Some("New SA")
        );
    }

    #[test]
    fn test_update_service_account_request_shape() {
        let svc = IamRest::unauthenticated();
        let req = svc.update_service_account(
            "my-project",
            "sa@project.iam.gserviceaccount.com",
            "Updated SA",
        );
        assert!(req
            .url
            .contains("serviceAccounts/sa@project.iam.gserviceaccount.com?updateMask=displayName"));
        assert_eq!(req.method, HttpMethod::Patch);
        let body = req.body.expect("update request should include json body");
        assert_eq!(body["displayName"].as_str(), Some("Updated SA"));
    }

    #[test]
    fn test_delete_service_account_request_shape() {
        let svc = IamRest::unauthenticated();
        let req = svc.delete_service_account("my-project", "sa@project.iam.gserviceaccount.com");
        assert!(req
            .url
            .contains("serviceAccounts/sa@project.iam.gserviceaccount.com"));
        assert_eq!(req.method, HttpMethod::Delete);
        assert!(req.body.is_none(), "delete request should not include body");
    }

    #[test]
    fn test_get_service_account_iam_policy_request_shape() {
        let svc = IamRest::unauthenticated();
        let req =
            svc.get_service_account_iam_policy("my-project", "sa@project.iam.gserviceaccount.com");
        assert!(req.url.contains(":getIamPolicy"));
        assert_eq!(req.method, HttpMethod::Post);
    }

    #[test]
    fn test_set_service_account_iam_policy_request_shape() {
        let svc = IamRest::unauthenticated();
        let req = svc.set_service_account_iam_policy(
            "my-project",
            "sa@project.iam.gserviceaccount.com",
            serde_json::json!({
                "bindings": [
                    {
                        "role": "roles/iam.workloadIdentityUser",
                        "members": ["principalSet://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/pool/attribute.repository/gunb-ai/gunbc"]
                    }
                ]
            }),
        );
        assert!(req.url.contains(":setIamPolicy"));
        assert_eq!(req.method, HttpMethod::Post);
        let body = req.body.expect("setIamPolicy should include body");
        assert!(body.get("policy").is_some());
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
