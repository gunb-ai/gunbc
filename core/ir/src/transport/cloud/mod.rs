//! Cloud resource management layer.
//!
//! This module provides a unified abstraction for cloud resource provisioning
//! using the DAG upsert pattern. Resources are modeled as typed values that
//! flow through the DAG, with idempotent check → create → resolve semantics.
//!
//! # Architecture
//!
//! ```text
//! cloud/
//! ├── mod.rs          ← Core types (CloudProvider, ResourceDef, Credential)
//! ├── gcp/            ← GCP resources (Service Account, Secret Manager, etc.)
//! ├── aws/            ← AWS resources (IAM Role, Secrets Manager, etc.)
//! └── secrets/        ← Secret management and federation
//! ```
//!
//! # Design Principles
//!
//! 1. **Resources as Typed Values**: Cloud resources flow through DAG edges as
//!    typed values, not as side effects or annotations.
//!
//! 2. **Idempotent by Structure**: All resource operations use the Upsert pattern
//!    (Check → Create → Resolve) ensuring idempotent provisioning.
//!
//! 3. **Credentials via EnvVar**: Cloud credentials are stored as environment
//!    variable references (`EnvVar("GCP_CREDENTIALS")`), resolved at execution time.
//!
//! 4. **Provider Abstraction**: Common operations (list, get, create, delete) are
//!    abstracted across providers, with provider-specific extensions.
//!
//! 5. **CLI and API Support**: Resources can be managed via CLI tools (gcloud, aws)
//!    or native REST APIs, chosen based on context.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::cloud::{gcp, CloudCredential};
//!
//! // Define a service account resource
//! let sa = gcp::ServiceAccountDef {
//!     project_id: "my-project",
//!     account_id: "my-service-account",
//!     display_name: "My Service Account",
//!     roles: &["roles/secretmanager.secretAccessor"],
//! };
//!
//! // Build an upsert node for the service account
//! let upsert_node = sa.upsert_node(CloudCredential::EnvVar("GCP_CREDENTIALS"));
//! ```

pub mod aws;
pub mod gcp;
pub mod gunbc_dag_secrets;
pub mod secrets;
pub mod upsert;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-exports
pub use aws::{AwsCredential, AwsRegion, AwsResourceType};
pub use gcp::{GcpCredential, GcpLocation, GcpResourceType};
pub use secrets::{SecretRef, SecretSource, WorkloadIdentityConfig};
pub use upsert::{CloudResourceOp, CloudResourceUpsertBuilder};

// ============================================================================
// Core Types
// ============================================================================

/// Cloud provider identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    /// Google Cloud Platform
    Gcp,
    /// Amazon Web Services
    Aws,
    /// Microsoft Azure (planned)
    Azure,
}

impl CloudProvider {
    /// Get the provider name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gcp => "gcp",
            Self::Aws => "aws",
            Self::Azure => "azure",
        }
    }

    /// Get the CLI tool name for this provider.
    pub fn cli_tool(&self) -> &'static str {
        match self {
            Self::Gcp => "gcloud",
            Self::Aws => "aws",
            Self::Azure => "az",
        }
    }

    /// Get the environment variable name for default credentials.
    pub fn default_credential_env(&self) -> &'static str {
        match self {
            Self::Gcp => "GOOGLE_APPLICATION_CREDENTIALS",
            Self::Aws => "AWS_ACCESS_KEY_ID",
            Self::Azure => "AZURE_CLIENT_ID",
        }
    }
}

/// Cloud credential reference.
///
/// Credentials are stored as references, not values. They are resolved at
/// execution time from the environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CloudCredential {
    /// Reference to an environment variable containing credentials.
    ///
    /// For GCP: Path to service account JSON file (GOOGLE_APPLICATION_CREDENTIALS)
    /// For AWS: Access key ID (AWS_ACCESS_KEY_ID)
    EnvVar(String),

    /// Use the default credential chain for the provider.
    ///
    /// For GCP: Application Default Credentials (ADC)
    /// For AWS: Default credential provider chain
    Default,

    /// Use workload identity federation (no long-lived credentials).
    ///
    /// For GCP: Workload Identity Federation with OIDC
    /// For AWS: IAM Roles for Service Accounts (IRSA) / Web Identity
    WorkloadIdentity(WorkloadIdentityConfig),

    /// Service account impersonation (GCP-specific).
    ///
    /// Use one service account to impersonate another.
    Impersonate {
        /// The service account to impersonate
        target_service_account: String,
        /// Credential to use for impersonation
        source_credential: Box<CloudCredential>,
    },
}

impl CloudCredential {
    /// Create a credential from an environment variable.
    pub fn env(name: impl Into<String>) -> Self {
        Self::EnvVar(name.into())
    }

    /// Use default credentials.
    pub fn default() -> Self {
        Self::Default
    }

    /// Use workload identity federation.
    pub fn workload_identity(config: WorkloadIdentityConfig) -> Self {
        Self::WorkloadIdentity(config)
    }
}

// ============================================================================
// Resource Definition
// ============================================================================

/// State of a cloud resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    /// Resource does not exist
    NotFound,
    /// Resource exists and is active
    Active,
    /// Resource is being created
    Creating,
    /// Resource is being deleted
    Deleting,
    /// Resource exists but is in an error state
    Error,
    /// Resource state is unknown (check failed)
    Unknown,
}

impl ResourceState {
    /// Returns true if the resource exists (Active, Creating, Error).
    pub fn exists(&self) -> bool {
        matches!(self, Self::Active | Self::Creating | Self::Error)
    }

    /// Returns true if the resource is ready for use.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// Generic cloud resource handle.
///
/// This is the typed value that flows through DAG edges after a resource
/// is created or resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceHandle {
    /// Cloud provider
    pub provider: CloudProvider,
    /// Resource type identifier
    pub resource_type: String,
    /// Fully qualified resource name/ARN/ID
    pub resource_id: String,
    /// Resource state
    pub state: ResourceState,
    /// Provider-specific metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ResourceHandle {
    /// Create a new resource handle.
    pub fn new(
        provider: CloudProvider,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        state: ResourceState,
    ) -> Self {
        Self {
            provider,
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            state,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the handle.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Get a metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Get a metadata value as a string.
    pub fn get_metadata_str(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).and_then(|v| v.as_str())
    }
}

// ============================================================================
// Resource Operations
// ============================================================================

/// Operation type for cloud resources.
///
/// These map to the upsert pattern phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceOp {
    /// Check if resource exists (read-only)
    Check,
    /// Create resource if missing
    Create,
    /// Update resource configuration
    Update,
    /// Delete resource
    Delete,
    /// Resolve/verify resource state
    Resolve,
}

/// Result of a resource check operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    /// Whether the resource exists
    pub exists: bool,
    /// Current state if exists
    pub state: ResourceState,
    /// Resource handle if exists
    pub handle: Option<ResourceHandle>,
}

impl CheckResult {
    /// Resource does not exist.
    pub fn not_found() -> Self {
        Self {
            exists: false,
            state: ResourceState::NotFound,
            handle: None,
        }
    }

    /// Resource exists with the given handle.
    pub fn found(handle: ResourceHandle) -> Self {
        Self {
            exists: true,
            state: handle.state,
            handle: Some(handle),
        }
    }
}

/// Result of a resource create operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateResult {
    /// Whether creation succeeded
    pub success: bool,
    /// Resource handle if created
    pub handle: Option<ResourceHandle>,
    /// Error message if failed
    pub error: Option<String>,
}

impl CreateResult {
    /// Successful creation.
    pub fn success(handle: ResourceHandle) -> Self {
        Self {
            success: true,
            handle: Some(handle),
            error: None,
        }
    }

    /// Failed creation.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            handle: None,
            error: Some(error.into()),
        }
    }
}

// ============================================================================
// CLI Tool Definitions for Cloud Providers
// ============================================================================

use crate::transport::tool::{InstallInputs, InstallOption, ToolDef};

/// gcloud CLI tool (Google Cloud SDK).
pub static GCLOUD: ToolDef = ToolDef {
    id: "gcloud",
    command: "gcloud",
    verify: "gcloud --version",
    install_options: &[
        InstallOption {
            via: "brew",
            inputs: InstallInputs::packages(&["google-cloud-sdk"]),
        },
        // apt via google's package repository (requires setup)
        InstallOption {
            via: "apt",
            inputs: InstallInputs::packages(&["google-cloud-cli"]),
        },
    ],
    depends_on: &[],
};

/// AWS CLI tool.
pub static AWS_CLI: ToolDef = ToolDef {
    id: "aws",
    command: "aws",
    verify: "aws --version",
    install_options: &[
        InstallOption {
            via: "brew",
            inputs: InstallInputs::packages(&["awscli"]),
        },
        InstallOption {
            via: "apt",
            inputs: InstallInputs::packages(&["awscli"]),
        },
    ],
    depends_on: &[],
};

/// Azure CLI tool.
pub static AZ_CLI: ToolDef = ToolDef {
    id: "az",
    command: "az",
    verify: "az --version",
    install_options: &[
        InstallOption {
            via: "brew",
            inputs: InstallInputs::packages(&["azure-cli"]),
        },
        InstallOption {
            via: "apt",
            inputs: InstallInputs::packages(&["azure-cli"]),
        },
    ],
    depends_on: &[],
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_provider() {
        assert_eq!(CloudProvider::Gcp.as_str(), "gcp");
        assert_eq!(CloudProvider::Gcp.cli_tool(), "gcloud");
        assert_eq!(
            CloudProvider::Gcp.default_credential_env(),
            "GOOGLE_APPLICATION_CREDENTIALS"
        );

        assert_eq!(CloudProvider::Aws.as_str(), "aws");
        assert_eq!(CloudProvider::Aws.cli_tool(), "aws");
        assert_eq!(CloudProvider::Aws.default_credential_env(), "AWS_ACCESS_KEY_ID");
    }

    #[test]
    fn test_resource_state() {
        assert!(!ResourceState::NotFound.exists());
        assert!(ResourceState::Active.exists());
        assert!(ResourceState::Creating.exists());
        assert!(ResourceState::Error.exists());
        assert!(!ResourceState::Deleting.exists());

        assert!(ResourceState::Active.is_ready());
        assert!(!ResourceState::Creating.is_ready());
    }

    #[test]
    fn test_resource_handle() {
        let handle = ResourceHandle::new(
            CloudProvider::Gcp,
            "serviceAccount",
            "projects/my-project/serviceAccounts/my-sa@my-project.iam.gserviceaccount.com",
            ResourceState::Active,
        )
        .with_metadata("email", serde_json::json!("my-sa@my-project.iam.gserviceaccount.com"));

        assert_eq!(handle.provider, CloudProvider::Gcp);
        assert!(handle.state.is_ready());
        assert_eq!(
            handle.get_metadata_str("email"),
            Some("my-sa@my-project.iam.gserviceaccount.com")
        );
    }

    #[test]
    fn test_check_result() {
        let not_found = CheckResult::not_found();
        assert!(!not_found.exists);
        assert!(not_found.handle.is_none());

        let handle = ResourceHandle::new(
            CloudProvider::Aws,
            "iam:role",
            "arn:aws:iam::123456789012:role/my-role",
            ResourceState::Active,
        );
        let found = CheckResult::found(handle);
        assert!(found.exists);
        assert!(found.handle.is_some());
    }

    #[test]
    fn test_credential_types() {
        let env_cred = CloudCredential::env("GCP_CREDENTIALS");
        assert!(matches!(env_cred, CloudCredential::EnvVar(_)));

        let default_cred = CloudCredential::default();
        assert!(matches!(default_cred, CloudCredential::Default));
    }
}
