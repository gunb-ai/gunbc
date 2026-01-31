//! Cloud provider abstraction for resource upserts.
//!
//! This module models cloud resources (GCP, AWS) as typed identifiers that
//! can be upserted using the standard Check → Create → Resolve pattern.
//!
//! # Design
//!
//! Cloud resources are modeled as data — provider, project/account, resource
//! kind, and name. Authentication is handled separately through [`SecretRef`]
//! and federation patterns (e.g., GCP Workload Identity Federation).
//!
//! The key abstraction is [`CloudResource`]: a fully-qualified identifier for
//! a cloud resource that can be checked, created, and resolved.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::cloud::*;
//! use gunbc_ir::transport::cloud::gcp::*;
//!
//! let bucket = CloudResource::gcp(
//!     GcpProject::new("my-project"),
//!     GcpResource::StorageBucket { name: "my-bucket".into(), location: "us-central1".into() },
//! );
//!
//! // Use with UpsertBuilder for idempotent provisioning
//! ```

pub mod aws;
pub mod gcp;

use serde::{Deserialize, Serialize};

/// Cloud provider identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudProvider {
    /// Google Cloud Platform
    Gcp,
    /// Amazon Web Services
    Aws,
}

impl CloudProvider {
    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Gcp => "GCP",
            Self::Aws => "AWS",
        }
    }

    /// Get the CLI tool name for this provider.
    pub fn cli_tool(&self) -> &'static str {
        match self {
            Self::Gcp => "gcloud",
            Self::Aws => "aws",
        }
    }
}

impl std::fmt::Display for CloudProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A fully-qualified cloud resource identifier.
///
/// Combines provider, project/account scope, and resource-specific identity.
/// This is the unit of upsert — one `CloudResource` maps to one
/// Check → Create → Resolve cycle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CloudResource {
    /// Which cloud provider owns this resource.
    pub provider: CloudProvider,
    /// Provider-specific project or account scope.
    pub scope: String,
    /// Resource kind (e.g., "storage_bucket", "iam_role").
    pub kind: String,
    /// Resource name within its scope.
    pub name: String,
}

impl CloudResource {
    /// Create a GCP cloud resource.
    pub fn gcp(project: &gcp::GcpProject, resource: &gcp::GcpResource) -> Self {
        Self {
            provider: CloudProvider::Gcp,
            scope: project.id.clone(),
            kind: resource.kind().to_string(),
            name: resource.name().to_string(),
        }
    }

    /// Create an AWS cloud resource.
    pub fn aws(account: &aws::AwsAccount, resource: &aws::AwsResource) -> Self {
        Self {
            provider: CloudProvider::Aws,
            scope: account.id.clone(),
            kind: resource.kind().to_string(),
            name: resource.name().to_string(),
        }
    }

    /// Get the canonical resource ID string.
    ///
    /// Format: `cloud:{provider}:{scope}:{kind}/{name}`
    pub fn resource_id(&self) -> String {
        format!(
            "cloud:{}:{}:{}/{}",
            self.provider.name().to_lowercase(),
            self.scope,
            self.kind,
            self.name,
        )
    }
}

impl std::fmt::Display for CloudResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.resource_id())
    }
}

/// State of a cloud resource after a check operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudResourceState {
    /// Resource exists and is ready.
    Exists,
    /// Resource does not exist (can be created).
    NotFound,
    /// Resource exists but is in a degraded/error state.
    Degraded { reason: String },
}

impl CloudResourceState {
    /// Whether the resource exists (regardless of state).
    pub fn exists(&self) -> bool {
        matches!(self, Self::Exists | Self::Degraded { .. })
    }

    /// Whether the resource needs to be created.
    pub fn needs_create(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

/// Result of a cloud resource upsert operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudUpsertResult {
    /// The resource that was upserted.
    pub resource: CloudResource,
    /// Whether the resource was created (true) or already existed (false).
    pub was_created: bool,
    /// Final state after upsert.
    pub state: CloudResourceState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_provider_display() {
        assert_eq!(CloudProvider::Gcp.name(), "GCP");
        assert_eq!(CloudProvider::Aws.name(), "AWS");
    }

    #[test]
    fn test_cloud_provider_cli_tool() {
        assert_eq!(CloudProvider::Gcp.cli_tool(), "gcloud");
        assert_eq!(CloudProvider::Aws.cli_tool(), "aws");
    }

    #[test]
    fn test_cloud_resource_id_format() {
        let resource = CloudResource {
            provider: CloudProvider::Gcp,
            scope: "my-project".to_string(),
            kind: "storage_bucket".to_string(),
            name: "my-bucket".to_string(),
        };
        assert_eq!(
            resource.resource_id(),
            "cloud:gcp:my-project:storage_bucket/my-bucket"
        );
    }

    #[test]
    fn test_cloud_resource_state() {
        assert!(CloudResourceState::Exists.exists());
        assert!(!CloudResourceState::NotFound.exists());
        assert!(CloudResourceState::Degraded { reason: "err".into() }.exists());

        assert!(!CloudResourceState::Exists.needs_create());
        assert!(CloudResourceState::NotFound.needs_create());
    }
}
