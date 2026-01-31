//! GCP IAM resource definitions.
//!
//! This module provides types for managing GCP IAM resources:
//! - Service Accounts
//! - IAM Bindings (role grants)
//! - IAM Policies

use super::{GcpResourceType, ResourceName};
use crate::transport::cloud::{CloudProvider, ResourceHandle, ResourceState};
use crate::transport::{RestRequest, ShellRequest};
use serde::{Deserialize, Serialize};

// ============================================================================
// Service Account
// ============================================================================

/// Service Account definition.
///
/// Service accounts are used for workload authentication in GCP.
/// They can be granted roles and used for workload identity federation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceAccountDef {
    /// GCP project ID
    pub project_id: String,
    /// Service account ID (the part before @)
    pub account_id: String,
    /// Display name for the service account
    pub display_name: Option<String>,
    /// Description
    pub description: Option<String>,
}

impl ServiceAccountDef {
    /// Create a new service account definition.
    pub fn new(project_id: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            account_id: account_id.into(),
            display_name: None,
            description: None,
        }
    }

    /// Set the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Get the full email address of this service account.
    pub fn email(&self) -> String {
        ResourceName::service_account_email(&self.project_id, &self.account_id)
    }

    /// Get the fully qualified resource name.
    pub fn resource_name(&self) -> String {
        ResourceName::service_account(&self.project_id, &self.account_id)
    }

    // ========================================================================
    // CLI Commands (gcloud)
    // ========================================================================

    /// Generate the gcloud command to check if this service account exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "iam",
                "service-accounts",
                "describe",
                &self.email(),
                "--project",
                &self.project_id,
                "--format=json",
            ])
    }

    /// Generate the gcloud command to create this service account.
    pub fn create_command(&self) -> ShellRequest {
        let mut args = vec![
            "iam".to_string(),
            "service-accounts".to_string(),
            "create".to_string(),
            self.account_id.clone(),
            "--project".to_string(),
            self.project_id.clone(),
        ];

        if let Some(ref name) = self.display_name {
            args.push("--display-name".to_string());
            args.push(name.clone());
        }

        if let Some(ref desc) = self.description {
            args.push("--description".to_string());
            args.push(desc.clone());
        }

        args.push("--format=json".to_string());

        ShellRequest::new("gcloud").args(args)
    }

    /// Generate the gcloud command to delete this service account.
    pub fn delete_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "iam",
                "service-accounts",
                "delete",
                &self.email(),
                "--project",
                &self.project_id,
                "--quiet",
            ])
    }

    // ========================================================================
    // REST API Requests
    // ========================================================================

    /// Generate REST API request to check if this service account exists.
    pub fn check_rest_request(&self) -> RestRequest {
        RestRequest::get(format!(
            "https://iam.googleapis.com/v1/{}",
            self.resource_name()
        ))
        .auth_env("GCP_ACCESS_TOKEN")
    }

    /// Generate REST API request to create this service account.
    pub fn create_rest_request(&self) -> RestRequest {
        let body = serde_json::json!({
            "accountId": self.account_id,
            "serviceAccount": {
                "displayName": self.display_name.as_deref().unwrap_or(&self.account_id),
                "description": self.description.as_deref().unwrap_or(""),
            }
        });

        RestRequest::post(format!(
            "https://iam.googleapis.com/v1/projects/{}/serviceAccounts",
            self.project_id
        ))
        .json(body)
        .auth_env("GCP_ACCESS_TOKEN")
    }

    /// Create a resource handle for this service account.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Gcp,
            GcpResourceType::ServiceAccount.as_str(),
            self.resource_name(),
            state,
        )
        .with_metadata("email", serde_json::json!(self.email()))
        .with_metadata("project_id", serde_json::json!(&self.project_id))
    }
}

// ============================================================================
// IAM Binding
// ============================================================================

/// IAM role binding definition.
///
/// Grants a role to a member (user, service account, group).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IamBinding {
    /// Resource to bind the role to (project, folder, organization, or resource)
    pub resource: IamResource,
    /// Role to grant
    pub role: String,
    /// Member to grant the role to
    pub member: IamMember,
    /// Optional condition for the binding
    pub condition: Option<IamCondition>,
}

/// IAM resource types that can have policies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IamResource {
    /// Project-level binding
    Project(String),
    /// Folder-level binding
    Folder(String),
    /// Organization-level binding
    Organization(String),
    /// Service account-level binding
    ServiceAccount { project: String, email: String },
    /// Secret Manager secret
    Secret { project: String, secret_id: String },
}

impl IamResource {
    /// Get the resource identifier for gcloud commands.
    pub fn resource_id(&self) -> String {
        match self {
            Self::Project(id) => id.clone(),
            Self::Folder(id) => id.clone(),
            Self::Organization(id) => id.clone(),
            Self::ServiceAccount { email, .. } => email.clone(),
            Self::Secret { project, secret_id } => {
                format!("projects/{}/secrets/{}", project, secret_id)
            }
        }
    }

    /// Get the gcloud command prefix for this resource type.
    pub fn gcloud_prefix(&self) -> Vec<&str> {
        match self {
            Self::Project(_) => vec!["projects"],
            Self::Folder(_) => vec!["resource-manager", "folders"],
            Self::Organization(_) => vec!["organizations"],
            Self::ServiceAccount { .. } => vec!["iam", "service-accounts"],
            Self::Secret { .. } => vec!["secrets"],
        }
    }
}

/// IAM member types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IamMember {
    /// User email
    User(String),
    /// Service account email
    ServiceAccount(String),
    /// Google group email
    Group(String),
    /// Domain
    Domain(String),
    /// All authenticated users
    AllAuthenticatedUsers,
    /// All users (public)
    AllUsers,
    /// Principal set (for workload identity)
    PrincipalSet {
        pool: String,
        attribute: String,
        value: String,
    },
}

impl IamMember {
    /// Format as the IAM member string.
    pub fn as_str(&self) -> String {
        match self {
            Self::User(email) => format!("user:{}", email),
            Self::ServiceAccount(email) => format!("serviceAccount:{}", email),
            Self::Group(email) => format!("group:{}", email),
            Self::Domain(domain) => format!("domain:{}", domain),
            Self::AllAuthenticatedUsers => "allAuthenticatedUsers".to_string(),
            Self::AllUsers => "allUsers".to_string(),
            Self::PrincipalSet {
                pool,
                attribute,
                value,
            } => format!("principalSet://iam.googleapis.com/{}/attribute.{}/{}", pool, attribute, value),
        }
    }

    /// Create a service account member.
    pub fn service_account(email: impl Into<String>) -> Self {
        Self::ServiceAccount(email.into())
    }

    /// Create a principal set for workload identity federation.
    pub fn principal_set(
        pool: impl Into<String>,
        attribute: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::PrincipalSet {
            pool: pool.into(),
            attribute: attribute.into(),
            value: value.into(),
        }
    }
}

/// IAM condition for conditional role bindings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IamCondition {
    /// Condition title
    pub title: String,
    /// Condition description
    pub description: Option<String>,
    /// CEL expression
    pub expression: String,
}

impl IamBinding {
    /// Create a new IAM binding.
    pub fn new(resource: IamResource, role: impl Into<String>, member: IamMember) -> Self {
        Self {
            resource,
            role: role.into(),
            member,
            condition: None,
        }
    }

    /// Add a condition to the binding.
    pub fn with_condition(mut self, title: impl Into<String>, expression: impl Into<String>) -> Self {
        self.condition = Some(IamCondition {
            title: title.into(),
            description: None,
            expression: expression.into(),
        });
        self
    }

    /// Generate the gcloud command to add this IAM binding.
    pub fn add_command(&self) -> ShellRequest {
        let mut args: Vec<String> = self.resource.gcloud_prefix().iter().map(|s| s.to_string()).collect();

        args.push("add-iam-policy-binding".to_string());
        args.push(self.resource.resource_id());
        args.push("--member".to_string());
        args.push(self.member.as_str());
        args.push("--role".to_string());
        args.push(self.role.clone());

        if let Some(ref cond) = self.condition {
            args.push("--condition".to_string());
            args.push(format!(
                "title={},expression={}",
                cond.title, cond.expression
            ));
        }

        ShellRequest::new("gcloud").args(args)
    }

    /// Generate the gcloud command to remove this IAM binding.
    pub fn remove_command(&self) -> ShellRequest {
        let mut args: Vec<String> = self.resource.gcloud_prefix().iter().map(|s| s.to_string()).collect();

        args.push("remove-iam-policy-binding".to_string());
        args.push(self.resource.resource_id());
        args.push("--member".to_string());
        args.push(self.member.as_str());
        args.push("--role".to_string());
        args.push(self.role.clone());

        ShellRequest::new("gcloud").args(args)
    }
}

// ============================================================================
// Role Definitions
// ============================================================================

/// Common GCP IAM roles.
pub struct RoleDef;

impl RoleDef {
    // Secret Manager roles
    pub const SECRET_ACCESSOR: &'static str = "roles/secretmanager.secretAccessor";
    pub const SECRET_ADMIN: &'static str = "roles/secretmanager.admin";
    pub const SECRET_VIEWER: &'static str = "roles/secretmanager.viewer";

    // Service Account roles
    pub const SERVICE_ACCOUNT_USER: &'static str = "roles/iam.serviceAccountUser";
    pub const SERVICE_ACCOUNT_TOKEN_CREATOR: &'static str = "roles/iam.serviceAccountTokenCreator";
    pub const WORKLOAD_IDENTITY_USER: &'static str = "roles/iam.workloadIdentityUser";

    // Storage roles
    pub const STORAGE_ADMIN: &'static str = "roles/storage.admin";
    pub const STORAGE_OBJECT_VIEWER: &'static str = "roles/storage.objectViewer";
    pub const STORAGE_OBJECT_CREATOR: &'static str = "roles/storage.objectCreator";

    // Compute roles
    pub const COMPUTE_VIEWER: &'static str = "roles/compute.viewer";
    pub const COMPUTE_ADMIN: &'static str = "roles/compute.admin";

    // Project roles
    pub const VIEWER: &'static str = "roles/viewer";
    pub const EDITOR: &'static str = "roles/editor";
    pub const OWNER: &'static str = "roles/owner";
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_account_def() {
        let sa = ServiceAccountDef::new("my-project", "github-actions")
            .with_display_name("GitHub Actions SA")
            .with_description("Service account for GitHub Actions CI/CD");

        assert_eq!(sa.email(), "github-actions@my-project.iam.gserviceaccount.com");
        assert_eq!(
            sa.resource_name(),
            "projects/my-project/serviceAccounts/github-actions@my-project.iam.gserviceaccount.com"
        );
    }

    #[test]
    fn test_service_account_check_command() {
        let sa = ServiceAccountDef::new("my-project", "my-sa");
        let cmd = sa.check_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"describe".to_string()));
        assert!(cmd.args.contains(&"my-sa@my-project.iam.gserviceaccount.com".to_string()));
    }

    #[test]
    fn test_service_account_create_command() {
        let sa = ServiceAccountDef::new("my-project", "my-sa")
            .with_display_name("My SA");
        let cmd = sa.create_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"create".to_string()));
        assert!(cmd.args.contains(&"my-sa".to_string()));
        assert!(cmd.args.contains(&"--display-name".to_string()));
    }

    #[test]
    fn test_iam_member_format() {
        assert_eq!(
            IamMember::User("user@example.com".to_string()).as_str(),
            "user:user@example.com"
        );
        assert_eq!(
            IamMember::ServiceAccount("sa@project.iam.gserviceaccount.com".to_string()).as_str(),
            "serviceAccount:sa@project.iam.gserviceaccount.com"
        );
        assert_eq!(
            IamMember::AllAuthenticatedUsers.as_str(),
            "allAuthenticatedUsers"
        );
    }

    #[test]
    fn test_principal_set() {
        let member = IamMember::principal_set(
            "projects/123/locations/global/workloadIdentityPools/github-pool",
            "repository",
            "owner/repo",
        );

        let member_str = member.as_str();
        assert!(member_str.starts_with("principalSet://"));
        assert!(member_str.contains("github-pool"));
        assert!(member_str.contains("repository"));
        assert!(member_str.contains("owner/repo"));
    }

    #[test]
    fn test_iam_binding() {
        let binding = IamBinding::new(
            IamResource::Project("my-project".to_string()),
            RoleDef::SECRET_ACCESSOR,
            IamMember::service_account("sa@my-project.iam.gserviceaccount.com"),
        );

        let cmd = binding.add_command();
        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"add-iam-policy-binding".to_string()));
        assert!(cmd.args.contains(&"my-project".to_string()));
        assert!(cmd.args.contains(&RoleDef::SECRET_ACCESSOR.to_string()));
    }

    #[test]
    fn test_iam_binding_with_condition() {
        let binding = IamBinding::new(
            IamResource::Project("my-project".to_string()),
            RoleDef::SECRET_ACCESSOR,
            IamMember::service_account("sa@project.iam.gserviceaccount.com"),
        )
        .with_condition("time-limited", "request.time < timestamp('2025-01-01T00:00:00Z')");

        assert!(binding.condition.is_some());
        let cmd = binding.add_command();
        assert!(cmd.args.contains(&"--condition".to_string()));
    }

    #[test]
    fn test_service_account_handle() {
        let sa = ServiceAccountDef::new("my-project", "my-sa");
        let handle = sa.to_handle(ResourceState::Active);

        assert_eq!(handle.provider, CloudProvider::Gcp);
        assert!(handle.state.is_ready());
        assert_eq!(
            handle.get_metadata_str("email"),
            Some("my-sa@my-project.iam.gserviceaccount.com")
        );
    }
}
