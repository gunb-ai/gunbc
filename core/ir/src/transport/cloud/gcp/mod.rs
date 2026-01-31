//! Google Cloud Platform resource definitions.
//!
//! This module provides typed definitions for GCP resources that can be
//! managed using the DAG upsert pattern.
//!
//! # Supported Resources
//!
//! | Resource | Type | Description |
//! |----------|------|-------------|
//! | Service Account | `ServiceAccountDef` | IAM service account |
//! | Secret | `SecretDef` | Secret Manager secret |
//! | Workload Identity Pool | `WorkloadIdentityPoolDef` | Federation pool |
//! | Workload Identity Provider | `WorkloadIdentityProviderDef` | OIDC provider |
//!
//! # Architecture
//!
//! ```text
//! GCP Resource Hierarchy
//! ┌─────────────────────────────────┐
//! │ Project                         │
//! │ ├── Service Account ────────────┼──▶ Workload Identity binding
//! │ ├── Workload Identity Pool      │
//! │ │   └── Provider (GitHub OIDC)  │
//! │ └── Secret Manager              │
//! │     └── Secrets                 │
//! └─────────────────────────────────┘
//! ```

pub mod iam;
pub mod secret_manager;
pub mod workload_identity;

use serde::{Deserialize, Serialize};

pub use iam::{IamBinding, IamCondition, IamMember, IamResource, RoleDef, ServiceAccountDef};
pub use secret_manager::{SecretDef, SecretVersionDef};
pub use workload_identity::{WorkloadIdentityPoolDef, WorkloadIdentityProviderDef, WorkloadIdentityBinding};

// ============================================================================
// GCP-Specific Types
// ============================================================================

/// GCP credential reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GcpCredential {
    /// Path to service account JSON key file (via env var)
    ServiceAccountKey(String),
    /// Use Application Default Credentials
    ApplicationDefault,
    /// Use gcloud CLI authentication
    GcloudAuth,
    /// Workload Identity Federation
    WorkloadIdentity(super::secrets::GcpWorkloadIdentity),
}

impl Default for GcpCredential {
    fn default() -> Self {
        Self::ApplicationDefault
    }
}

/// GCP location/region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GcpLocation {
    /// Global resource (not region-specific)
    Global,
    /// Single region
    Region(String),
    /// Multi-region
    MultiRegion(GcpMultiRegion),
}

impl Default for GcpLocation {
    fn default() -> Self {
        Self::Global
    }
}

impl GcpLocation {
    /// Create a global location.
    pub fn global() -> Self {
        Self::Global
    }

    /// Create a regional location.
    pub fn region(region: impl Into<String>) -> Self {
        Self::Region(region.into())
    }

    /// Get the location string for API calls.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Global => "global",
            Self::Region(r) => r.as_str(),
            Self::MultiRegion(m) => m.as_str(),
        }
    }
}

/// GCP multi-region locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GcpMultiRegion {
    /// United States
    Us,
    /// Europe
    Eu,
    /// Asia
    Asia,
}

impl GcpMultiRegion {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Us => "us",
            Self::Eu => "eu",
            Self::Asia => "asia",
        }
    }
}

/// GCP resource type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GcpResourceType {
    /// Service Account
    ServiceAccount,
    /// IAM binding
    IamBinding,
    /// Secret Manager secret
    Secret,
    /// Secret Manager secret version
    SecretVersion,
    /// Workload Identity Pool
    WorkloadIdentityPool,
    /// Workload Identity Provider
    WorkloadIdentityProvider,
}

impl GcpResourceType {
    /// Get the resource type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ServiceAccount => "iam.googleapis.com/ServiceAccount",
            Self::IamBinding => "iam.googleapis.com/Binding",
            Self::Secret => "secretmanager.googleapis.com/Secret",
            Self::SecretVersion => "secretmanager.googleapis.com/SecretVersion",
            Self::WorkloadIdentityPool => "iam.googleapis.com/WorkloadIdentityPool",
            Self::WorkloadIdentityProvider => "iam.googleapis.com/WorkloadIdentityPoolProvider",
        }
    }

    /// Get the gcloud command component for this resource.
    pub fn gcloud_component(&self) -> &'static str {
        match self {
            Self::ServiceAccount => "iam service-accounts",
            Self::IamBinding => "projects",
            Self::Secret => "secrets",
            Self::SecretVersion => "secrets versions",
            Self::WorkloadIdentityPool => "iam workload-identity-pools",
            Self::WorkloadIdentityProvider => "iam workload-identity-pools providers",
        }
    }
}

// ============================================================================
// Resource Name Formatting
// ============================================================================

/// GCP resource name builder.
///
/// Builds fully qualified resource names in GCP's standard format.
pub struct ResourceName;

impl ResourceName {
    /// Format a service account resource name.
    ///
    /// Format: `projects/{project}/serviceAccounts/{email}`
    pub fn service_account(project: &str, account_id: &str) -> String {
        let email = Self::service_account_email(project, account_id);
        format!("projects/{}/serviceAccounts/{}", project, email)
    }

    /// Format a service account email.
    ///
    /// Format: `{account_id}@{project}.iam.gserviceaccount.com`
    pub fn service_account_email(project: &str, account_id: &str) -> String {
        format!("{}@{}.iam.gserviceaccount.com", account_id, project)
    }

    /// Format a secret resource name.
    ///
    /// Format: `projects/{project}/secrets/{secret_id}`
    pub fn secret(project: &str, secret_id: &str) -> String {
        format!("projects/{}/secrets/{}", project, secret_id)
    }

    /// Format a secret version resource name.
    ///
    /// Format: `projects/{project}/secrets/{secret_id}/versions/{version}`
    pub fn secret_version(project: &str, secret_id: &str, version: &str) -> String {
        format!(
            "projects/{}/secrets/{}/versions/{}",
            project, secret_id, version
        )
    }

    /// Format a workload identity pool resource name.
    ///
    /// Format: `projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}`
    pub fn workload_identity_pool(project_number: u64, pool_id: &str) -> String {
        format!(
            "projects/{}/locations/global/workloadIdentityPools/{}",
            project_number, pool_id
        )
    }

    /// Format a workload identity provider resource name.
    ///
    /// Format: `projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}/providers/{provider_id}`
    pub fn workload_identity_provider(
        project_number: u64,
        pool_id: &str,
        provider_id: &str,
    ) -> String {
        format!(
            "projects/{}/locations/global/workloadIdentityPools/{}/providers/{}",
            project_number, pool_id, provider_id
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcp_location() {
        assert_eq!(GcpLocation::global().as_str(), "global");
        assert_eq!(GcpLocation::region("us-central1").as_str(), "us-central1");
        assert_eq!(GcpLocation::MultiRegion(GcpMultiRegion::Us).as_str(), "us");
    }

    #[test]
    fn test_resource_type_as_str() {
        assert_eq!(
            GcpResourceType::ServiceAccount.as_str(),
            "iam.googleapis.com/ServiceAccount"
        );
        assert_eq!(
            GcpResourceType::Secret.as_str(),
            "secretmanager.googleapis.com/Secret"
        );
    }

    #[test]
    fn test_service_account_email() {
        let email = ResourceName::service_account_email("my-project", "my-sa");
        assert_eq!(email, "my-sa@my-project.iam.gserviceaccount.com");
    }

    #[test]
    fn test_service_account_resource_name() {
        let name = ResourceName::service_account("my-project", "my-sa");
        assert_eq!(
            name,
            "projects/my-project/serviceAccounts/my-sa@my-project.iam.gserviceaccount.com"
        );
    }

    #[test]
    fn test_secret_resource_name() {
        let name = ResourceName::secret("my-project", "api-key");
        assert_eq!(name, "projects/my-project/secrets/api-key");
    }

    #[test]
    fn test_workload_identity_pool_name() {
        let name = ResourceName::workload_identity_pool(123456789, "github-pool");
        assert_eq!(
            name,
            "projects/123456789/locations/global/workloadIdentityPools/github-pool"
        );
    }

    #[test]
    fn test_workload_identity_provider_name() {
        let name =
            ResourceName::workload_identity_provider(123456789, "github-pool", "github-provider");
        assert_eq!(
            name,
            "projects/123456789/locations/global/workloadIdentityPools/github-pool/providers/github-provider"
        );
    }
}
