//! GCP Workload Identity Federation resource definitions.
//!
//! This module provides types for managing Workload Identity Federation,
//! which enables keyless authentication from external identity providers
//! (like GitHub Actions) to GCP.
//!
//! # Architecture
//!
//! ```text
//! GitHub Actions                  GCP
//! ┌────────────────┐             ┌─────────────────────────────┐
//! │ Workflow       │             │ Workload Identity Pool      │
//! │                │  OIDC       │   └── Provider (GitHub)     │
//! │ GITHUB_TOKEN ──┼─────────────┼──▶ Attribute Mapping        │
//! │                │  Token      │                             │
//! └────────────────┘             │   ┌─────────────────────┐   │
//!                                │   │ Service Account     │   │
//!                                │   │ (impersonated)      │   │
//!                                │   └─────────────────────┘   │
//!                                └─────────────────────────────┘
//! ```

use super::{GcpResourceType, ResourceName};
use crate::transport::cloud::{CloudProvider, ResourceHandle, ResourceState};
use crate::transport::ShellRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Workload Identity Pool
// ============================================================================

/// Workload Identity Pool definition.
///
/// A pool is a container for workload identity providers and defines
/// the trust boundary for external identities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadIdentityPoolDef {
    /// GCP project ID
    pub project_id: String,
    /// GCP project number (required for resource names)
    pub project_number: u64,
    /// Pool ID (unique within project)
    pub pool_id: String,
    /// Display name
    pub display_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Whether the pool is disabled
    pub disabled: bool,
}

impl WorkloadIdentityPoolDef {
    /// Create a new workload identity pool definition.
    pub fn new(
        project_id: impl Into<String>,
        project_number: u64,
        pool_id: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            project_number,
            pool_id: pool_id.into(),
            display_name: None,
            description: None,
            disabled: false,
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

    /// Get the fully qualified resource name.
    pub fn resource_name(&self) -> String {
        ResourceName::workload_identity_pool(self.project_number, &self.pool_id)
    }

    /// Generate the gcloud command to check if this pool exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "iam",
                "workload-identity-pools",
                "describe",
                &self.pool_id,
                "--location=global",
                "--project",
                &self.project_id,
                "--format=json",
            ])
    }

    /// Generate the gcloud command to create this pool.
    pub fn create_command(&self) -> ShellRequest {
        let mut args = vec![
            "iam".to_string(),
            "workload-identity-pools".to_string(),
            "create".to_string(),
            self.pool_id.clone(),
            "--location=global".to_string(),
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

    /// Generate the gcloud command to delete this pool.
    pub fn delete_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "iam",
                "workload-identity-pools",
                "delete",
                &self.pool_id,
                "--location=global",
                "--project",
                &self.project_id,
                "--quiet",
            ])
    }

    /// Create a resource handle for this pool.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Gcp,
            GcpResourceType::WorkloadIdentityPool.as_str(),
            self.resource_name(),
            state,
        )
        .with_metadata("pool_id", serde_json::json!(&self.pool_id))
        .with_metadata("project_id", serde_json::json!(&self.project_id))
    }
}

// ============================================================================
// Workload Identity Provider
// ============================================================================

/// Workload Identity Provider definition.
///
/// A provider defines how to verify and map external identities (like
/// GitHub Actions OIDC tokens) to GCP identities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadIdentityProviderDef {
    /// Parent pool
    pub pool: WorkloadIdentityPoolDef,
    /// Provider ID (unique within pool)
    pub provider_id: String,
    /// Display name
    pub display_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Provider type and configuration
    pub provider_type: ProviderType,
    /// Attribute mapping from OIDC claims to GCP attributes
    #[serde(default)]
    pub attribute_mapping: HashMap<String, String>,
    /// Attribute condition CEL expression
    pub attribute_condition: Option<String>,
}

/// Provider type configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProviderType {
    /// OIDC provider
    Oidc {
        /// OIDC issuer URI
        issuer_uri: String,
        /// Allowed audiences (optional)
        allowed_audiences: Vec<String>,
    },
    /// AWS provider
    Aws {
        /// AWS account ID
        account_id: String,
    },
}

impl WorkloadIdentityProviderDef {
    /// Create a new OIDC provider definition.
    pub fn oidc(
        pool: WorkloadIdentityPoolDef,
        provider_id: impl Into<String>,
        issuer_uri: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            provider_id: provider_id.into(),
            display_name: None,
            description: None,
            provider_type: ProviderType::Oidc {
                issuer_uri: issuer_uri.into(),
                allowed_audiences: Vec::new(),
            },
            attribute_mapping: HashMap::new(),
            attribute_condition: None,
        }
    }

    /// Create a provider for GitHub Actions OIDC.
    ///
    /// This sets up the standard configuration for GitHub Actions.
    pub fn github_actions(pool: WorkloadIdentityPoolDef, provider_id: impl Into<String>) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert("google.subject".to_string(), "assertion.sub".to_string());
        mapping.insert("attribute.actor".to_string(), "assertion.actor".to_string());
        mapping.insert(
            "attribute.repository".to_string(),
            "assertion.repository".to_string(),
        );
        mapping.insert(
            "attribute.repository_owner".to_string(),
            "assertion.repository_owner".to_string(),
        );
        mapping.insert("attribute.ref".to_string(), "assertion.ref".to_string());

        Self {
            pool,
            provider_id: provider_id.into(),
            display_name: Some("GitHub Actions".to_string()),
            description: Some("OIDC provider for GitHub Actions workflows".to_string()),
            provider_type: ProviderType::Oidc {
                issuer_uri: "https://token.actions.githubusercontent.com".to_string(),
                allowed_audiences: Vec::new(),
            },
            attribute_mapping: mapping,
            attribute_condition: None,
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

    /// Add an attribute mapping.
    pub fn with_attribute(mut self, gcp_attr: impl Into<String>, oidc_claim: impl Into<String>) -> Self {
        self.attribute_mapping.insert(gcp_attr.into(), oidc_claim.into());
        self
    }

    /// Set the attribute condition.
    ///
    /// Common conditions:
    /// - `assertion.repository == 'owner/repo'` - restrict to specific repo
    /// - `assertion.ref == 'refs/heads/main'` - restrict to specific branch
    /// - `assertion.repository_owner == 'my-org'` - restrict to organization
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.attribute_condition = Some(condition.into());
        self
    }

    /// Restrict to a specific repository.
    pub fn for_repository(mut self, repository: impl Into<String>) -> Self {
        self.attribute_condition = Some(format!("assertion.repository == '{}'", repository.into()));
        self
    }

    /// Restrict to repositories in an organization.
    pub fn for_organization(mut self, org: impl Into<String>) -> Self {
        self.attribute_condition = Some(format!("assertion.repository_owner == '{}'", org.into()));
        self
    }

    /// Get the fully qualified resource name.
    pub fn resource_name(&self) -> String {
        ResourceName::workload_identity_provider(
            self.pool.project_number,
            &self.pool.pool_id,
            &self.provider_id,
        )
    }

    /// Generate the gcloud command to check if this provider exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "iam",
                "workload-identity-pools",
                "providers",
                "describe",
                &self.provider_id,
                "--workload-identity-pool",
                &self.pool.pool_id,
                "--location=global",
                "--project",
                &self.pool.project_id,
                "--format=json",
            ])
    }

    /// Generate the gcloud command to create this provider.
    pub fn create_command(&self) -> ShellRequest {
        let mut args = vec![
            "iam".to_string(),
            "workload-identity-pools".to_string(),
            "providers".to_string(),
            "create-oidc".to_string(),
            self.provider_id.clone(),
            "--workload-identity-pool".to_string(),
            self.pool.pool_id.clone(),
            "--location=global".to_string(),
            "--project".to_string(),
            self.pool.project_id.clone(),
        ];

        // Add issuer URI
        if let ProviderType::Oidc { ref issuer_uri, ref allowed_audiences } = self.provider_type {
            args.push("--issuer-uri".to_string());
            args.push(issuer_uri.clone());

            if !allowed_audiences.is_empty() {
                args.push("--allowed-audiences".to_string());
                args.push(allowed_audiences.join(","));
            }
        }

        // Add display name
        if let Some(ref name) = self.display_name {
            args.push("--display-name".to_string());
            args.push(name.clone());
        }

        // Add attribute mapping
        if !self.attribute_mapping.is_empty() {
            args.push("--attribute-mapping".to_string());
            let mapping: Vec<String> = self
                .attribute_mapping
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            args.push(mapping.join(","));
        }

        // Add attribute condition
        if let Some(ref condition) = self.attribute_condition {
            args.push("--attribute-condition".to_string());
            args.push(condition.clone());
        }

        args.push("--format=json".to_string());

        ShellRequest::new("gcloud").args(args)
    }

    /// Generate the gcloud command to delete this provider.
    pub fn delete_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "iam",
                "workload-identity-pools",
                "providers",
                "delete",
                &self.provider_id,
                "--workload-identity-pool",
                &self.pool.pool_id,
                "--location=global",
                "--project",
                &self.pool.project_id,
                "--quiet",
            ])
    }

    /// Create a resource handle for this provider.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Gcp,
            GcpResourceType::WorkloadIdentityProvider.as_str(),
            self.resource_name(),
            state,
        )
        .with_metadata("provider_id", serde_json::json!(&self.provider_id))
        .with_metadata("pool_id", serde_json::json!(&self.pool.pool_id))
    }
}

// ============================================================================
// Service Account Binding for Workload Identity
// ============================================================================

/// Binding configuration for allowing workload identity to impersonate a service account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadIdentityBinding {
    /// Service account email to impersonate
    pub service_account_email: String,
    /// Project ID
    pub project_id: String,
    /// Pool resource name
    pub pool_name: String,
    /// Attribute to match (e.g., "repository")
    pub attribute: String,
    /// Value to match (e.g., "owner/repo")
    pub value: String,
}

impl WorkloadIdentityBinding {
    /// Create a binding for a specific repository.
    pub fn for_repository(
        service_account_email: impl Into<String>,
        project_id: impl Into<String>,
        pool: &WorkloadIdentityPoolDef,
        repository: impl Into<String>,
    ) -> Self {
        Self {
            service_account_email: service_account_email.into(),
            project_id: project_id.into(),
            pool_name: pool.resource_name(),
            attribute: "repository".to_string(),
            value: repository.into(),
        }
    }

    /// Get the principal set member string for IAM binding.
    pub fn principal_set(&self) -> String {
        format!(
            "principalSet://iam.googleapis.com/{}/attribute.{}/{}",
            self.pool_name, self.attribute, self.value
        )
    }

    /// Generate the gcloud command to add this binding.
    pub fn add_binding_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "iam",
                "service-accounts",
                "add-iam-policy-binding",
                &self.service_account_email,
                "--project",
                &self.project_id,
                "--role",
                "roles/iam.workloadIdentityUser",
                "--member",
                &self.principal_set(),
            ])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> WorkloadIdentityPoolDef {
        WorkloadIdentityPoolDef::new("my-project", 123456789, "github-pool")
    }

    #[test]
    fn test_pool_resource_name() {
        let pool = test_pool();
        assert_eq!(
            pool.resource_name(),
            "projects/123456789/locations/global/workloadIdentityPools/github-pool"
        );
    }

    #[test]
    fn test_pool_check_command() {
        let pool = test_pool();
        let cmd = pool.check_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"describe".to_string()));
        assert!(cmd.args.contains(&"github-pool".to_string()));
    }

    #[test]
    fn test_pool_create_command() {
        let pool = test_pool().with_display_name("GitHub Pool");
        let cmd = pool.create_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"create".to_string()));
        assert!(cmd.args.contains(&"--display-name".to_string()));
    }

    #[test]
    fn test_provider_github_actions() {
        let pool = test_pool();
        let provider = WorkloadIdentityProviderDef::github_actions(pool, "github-provider");

        assert_eq!(provider.provider_id, "github-provider");
        assert!(provider.attribute_mapping.contains_key("google.subject"));
        assert!(provider.attribute_mapping.contains_key("attribute.repository"));
    }

    #[test]
    fn test_provider_resource_name() {
        let pool = test_pool();
        let provider = WorkloadIdentityProviderDef::github_actions(pool, "github-provider");

        assert_eq!(
            provider.resource_name(),
            "projects/123456789/locations/global/workloadIdentityPools/github-pool/providers/github-provider"
        );
    }

    #[test]
    fn test_provider_for_repository() {
        let pool = test_pool();
        let provider = WorkloadIdentityProviderDef::github_actions(pool, "github-provider")
            .for_repository("owner/repo");

        assert_eq!(
            provider.attribute_condition,
            Some("assertion.repository == 'owner/repo'".to_string())
        );
    }

    #[test]
    fn test_provider_create_command() {
        let pool = test_pool();
        let provider = WorkloadIdentityProviderDef::github_actions(pool, "github-provider")
            .for_repository("owner/repo");
        let cmd = provider.create_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"create-oidc".to_string()));
        assert!(cmd.args.contains(&"--issuer-uri".to_string()));
        assert!(cmd.args.contains(&"--attribute-mapping".to_string()));
        assert!(cmd.args.contains(&"--attribute-condition".to_string()));
    }

    #[test]
    fn test_workload_identity_binding() {
        let pool = test_pool();
        let binding = WorkloadIdentityBinding::for_repository(
            "sa@my-project.iam.gserviceaccount.com",
            "my-project",
            &pool,
            "owner/repo",
        );

        let principal = binding.principal_set();
        assert!(principal.contains("github-pool"));
        assert!(principal.contains("repository"));
        assert!(principal.contains("owner/repo"));
    }

    #[test]
    fn test_workload_identity_binding_command() {
        let pool = test_pool();
        let binding = WorkloadIdentityBinding::for_repository(
            "sa@my-project.iam.gserviceaccount.com",
            "my-project",
            &pool,
            "owner/repo",
        );
        let cmd = binding.add_binding_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"add-iam-policy-binding".to_string()));
        assert!(cmd
            .args
            .contains(&"roles/iam.workloadIdentityUser".to_string()));
    }
}
