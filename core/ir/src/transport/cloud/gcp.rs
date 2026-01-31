//! GCP (Google Cloud Platform) resource modeling.
//!
//! Models GCP resources that can be upserted: checked for existence,
//! created if missing, and resolved to a handle.
//!
//! # Resources Modeled
//!
//! - **Storage Buckets** (GCS): Object storage
//! - **Service Accounts**: IAM identity for workloads
//! - **Secrets**: Secret Manager secrets
//! - **Pub/Sub Topics**: Messaging topics
//! - **Artifact Registry Repos**: Container/package registry
//!
//! # Authentication
//!
//! GCP authentication uses Workload Identity Federation (WIF) for keyless
//! access from GitHub Actions. This is modeled in [`GcpWifConfig`] and
//! integrates with the [`super::super::secret::SecretRef`] system.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::cloud::gcp::*;
//!
//! let project = GcpProject::new("my-project-123");
//! let bucket = GcpResource::StorageBucket {
//!     name: "my-data-bucket".into(),
//!     location: "us-central1".into(),
//! };
//!
//! // Check command: gsutil ls gs://my-data-bucket
//! let check = bucket.check_command(&project);
//! // Create command: gsutil mb -l us-central1 gs://my-data-bucket
//! let create = bucket.create_command(&project);
//! ```

use serde::{Deserialize, Serialize};

/// GCP project identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GcpProject {
    /// Project ID (e.g., "my-project-123").
    pub id: String,
}

impl GcpProject {
    /// Create a new GCP project reference.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl std::fmt::Display for GcpProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// GCP resource types that can be upserted.
///
/// Each variant carries the configuration needed to check, create,
/// and resolve the resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GcpResource {
    /// Google Cloud Storage bucket.
    StorageBucket {
        /// Bucket name (globally unique).
        name: String,
        /// Bucket location (e.g., "us-central1", "US").
        location: String,
    },

    /// IAM Service Account.
    ServiceAccount {
        /// Service account ID (e.g., "my-sa").
        /// Full email will be `{id}@{project}.iam.gserviceaccount.com`.
        id: String,
        /// Display name.
        display_name: String,
    },

    /// Secret Manager secret.
    Secret {
        /// Secret ID.
        id: String,
    },

    /// Pub/Sub topic.
    PubSubTopic {
        /// Topic name.
        name: String,
    },

    /// Artifact Registry repository.
    ArtifactRegistryRepo {
        /// Repository name.
        name: String,
        /// Repository location (e.g., "us-central1").
        location: String,
        /// Repository format (e.g., "docker", "npm", "python").
        format: String,
    },
}

impl GcpResource {
    /// Get the resource kind identifier.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::StorageBucket { .. } => "storage_bucket",
            Self::ServiceAccount { .. } => "service_account",
            Self::Secret { .. } => "secret",
            Self::PubSubTopic { .. } => "pubsub_topic",
            Self::ArtifactRegistryRepo { .. } => "artifact_registry_repo",
        }
    }

    /// Get the resource name.
    pub fn name(&self) -> &str {
        match self {
            Self::StorageBucket { name, .. } => name,
            Self::ServiceAccount { id, .. } => id,
            Self::Secret { id } => id,
            Self::PubSubTopic { name } => name,
            Self::ArtifactRegistryRepo { name, .. } => name,
        }
    }

    /// Generate the gcloud/gsutil check command for this resource.
    ///
    /// Returns `(command, args)` that can be used in a `ShellRequest`.
    pub fn check_command(&self, project: &GcpProject) -> (&'static str, Vec<String>) {
        match self {
            Self::StorageBucket { name, .. } => (
                "gcloud",
                vec![
                    "storage".into(),
                    "buckets".into(),
                    "describe".into(),
                    format!("gs://{name}"),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::ServiceAccount { id, .. } => (
                "gcloud",
                vec![
                    "iam".into(),
                    "service-accounts".into(),
                    "describe".into(),
                    format!("{id}@{}.iam.gserviceaccount.com", project.id),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::Secret { id } => (
                "gcloud",
                vec![
                    "secrets".into(),
                    "describe".into(),
                    id.clone(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::PubSubTopic { name } => (
                "gcloud",
                vec![
                    "pubsub".into(),
                    "topics".into(),
                    "describe".into(),
                    name.clone(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::ArtifactRegistryRepo { name, location, .. } => (
                "gcloud",
                vec![
                    "artifacts".into(),
                    "repositories".into(),
                    "describe".into(),
                    name.clone(),
                    "--location".into(),
                    location.clone(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
        }
    }

    /// Generate the gcloud/gsutil create command for this resource.
    ///
    /// Returns `(command, args)` that can be used in a `ShellRequest`.
    pub fn create_command(&self, project: &GcpProject) -> (&'static str, Vec<String>) {
        match self {
            Self::StorageBucket { name, location } => (
                "gcloud",
                vec![
                    "storage".into(),
                    "buckets".into(),
                    "create".into(),
                    format!("gs://{name}"),
                    "--location".into(),
                    location.clone(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::ServiceAccount { id, display_name } => (
                "gcloud",
                vec![
                    "iam".into(),
                    "service-accounts".into(),
                    "create".into(),
                    id.clone(),
                    "--display-name".into(),
                    display_name.clone(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::Secret { id } => (
                "gcloud",
                vec![
                    "secrets".into(),
                    "create".into(),
                    id.clone(),
                    "--replication-policy=automatic".into(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::PubSubTopic { name } => (
                "gcloud",
                vec![
                    "pubsub".into(),
                    "topics".into(),
                    "create".into(),
                    name.clone(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
            Self::ArtifactRegistryRepo {
                name,
                location,
                format,
            } => (
                "gcloud",
                vec![
                    "artifacts".into(),
                    "repositories".into(),
                    "create".into(),
                    name.clone(),
                    "--location".into(),
                    location.clone(),
                    "--repository-format".into(),
                    format.clone(),
                    "--project".into(),
                    project.id.clone(),
                    "--format=json".into(),
                ],
            ),
        }
    }

    /// Get the service account email for a ServiceAccount resource.
    ///
    /// Returns `None` for non-ServiceAccount resources.
    pub fn service_account_email(&self, project: &GcpProject) -> Option<String> {
        match self {
            Self::ServiceAccount { id, .. } => {
                Some(format!("{id}@{}.iam.gserviceaccount.com", project.id))
            }
            _ => None,
        }
    }
}

/// GCP Workload Identity Federation configuration.
///
/// Enables keyless authentication from GitHub Actions to GCP using OIDC.
/// The GitHub Actions runner requests an OIDC token, which GCP validates
/// against the workload identity pool to grant temporary credentials.
///
/// # Setup Requirements
///
/// 1. A Workload Identity Pool in the GCP project
/// 2. A Workload Identity Provider configured for GitHub OIDC
/// 3. A Service Account with IAM bindings to the pool
///
/// # Example
///
/// ```ignore
/// let wif = GcpWifConfig::new(
///     GcpProject::new("my-project"),
///     "my-pool",
///     "github-provider",
///     "ci-sa",
/// );
///
/// // Generates the auth action config for GitHub Actions
/// let provider_name = wif.provider_full_name();
/// // "projects/my-project/locations/global/workloadIdentityPools/my-pool/providers/github-provider"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpWifConfig {
    /// GCP project containing the workload identity pool.
    pub project: GcpProject,
    /// Workload Identity Pool ID.
    pub pool_id: String,
    /// Workload Identity Provider ID (configured for GitHub OIDC).
    pub provider_id: String,
    /// Service account ID to impersonate.
    pub service_account_id: String,
}

impl GcpWifConfig {
    /// Create a new WIF configuration.
    pub fn new(
        project: GcpProject,
        pool_id: impl Into<String>,
        provider_id: impl Into<String>,
        service_account_id: impl Into<String>,
    ) -> Self {
        Self {
            project,
            pool_id: pool_id.into(),
            provider_id: provider_id.into(),
            service_account_id: service_account_id.into(),
        }
    }

    /// Get the full Workload Identity Provider resource name.
    ///
    /// This is used in the `google-github-actions/auth@v2` action's
    /// `workload_identity_provider` input.
    pub fn provider_full_name(&self) -> String {
        format!(
            "projects/{}/locations/global/workloadIdentityPools/{}/providers/{}",
            self.project.id, self.pool_id, self.provider_id,
        )
    }

    /// Get the service account email for impersonation.
    pub fn service_account_email(&self) -> String {
        format!(
            "{}@{}.iam.gserviceaccount.com",
            self.service_account_id, self.project.id,
        )
    }

    /// List the GCP resources that must exist for WIF to work.
    ///
    /// These should be upserted before any workflow that uses WIF.
    pub fn required_resources(&self) -> Vec<GcpResource> {
        vec![
            GcpResource::ServiceAccount {
                id: self.service_account_id.clone(),
                display_name: format!("WIF SA for {}", self.pool_id),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_project() -> GcpProject {
        GcpProject::new("test-project-123")
    }

    #[test]
    fn test_gcp_project_display() {
        let project = test_project();
        assert_eq!(project.to_string(), "test-project-123");
    }

    #[test]
    fn test_gcp_resource_kind() {
        let bucket = GcpResource::StorageBucket {
            name: "b".into(),
            location: "us".into(),
        };
        assert_eq!(bucket.kind(), "storage_bucket");

        let sa = GcpResource::ServiceAccount {
            id: "sa".into(),
            display_name: "SA".into(),
        };
        assert_eq!(sa.kind(), "service_account");

        let secret = GcpResource::Secret { id: "s".into() };
        assert_eq!(secret.kind(), "secret");
    }

    #[test]
    fn test_gcp_resource_name() {
        let bucket = GcpResource::StorageBucket {
            name: "my-bucket".into(),
            location: "us-central1".into(),
        };
        assert_eq!(bucket.name(), "my-bucket");
    }

    #[test]
    fn test_bucket_check_command() {
        let project = test_project();
        let bucket = GcpResource::StorageBucket {
            name: "my-bucket".into(),
            location: "us-central1".into(),
        };
        let (cmd, args) = bucket.check_command(&project);
        assert_eq!(cmd, "gcloud");
        assert!(args.contains(&"describe".to_string()));
        assert!(args.contains(&"gs://my-bucket".to_string()));
    }

    #[test]
    fn test_bucket_create_command() {
        let project = test_project();
        let bucket = GcpResource::StorageBucket {
            name: "my-bucket".into(),
            location: "us-central1".into(),
        };
        let (cmd, args) = bucket.create_command(&project);
        assert_eq!(cmd, "gcloud");
        assert!(args.contains(&"create".to_string()));
        assert!(args.contains(&"gs://my-bucket".to_string()));
        assert!(args.contains(&"us-central1".to_string()));
    }

    #[test]
    fn test_service_account_email() {
        let project = test_project();
        let sa = GcpResource::ServiceAccount {
            id: "my-sa".into(),
            display_name: "My SA".into(),
        };
        assert_eq!(
            sa.service_account_email(&project),
            Some("my-sa@test-project-123.iam.gserviceaccount.com".to_string()),
        );
    }

    #[test]
    fn test_non_sa_has_no_email() {
        let project = test_project();
        let bucket = GcpResource::StorageBucket {
            name: "b".into(),
            location: "us".into(),
        };
        assert_eq!(bucket.service_account_email(&project), None);
    }

    #[test]
    fn test_wif_config() {
        let wif = GcpWifConfig::new(
            test_project(),
            "my-pool",
            "github-provider",
            "ci-sa",
        );

        assert_eq!(
            wif.provider_full_name(),
            "projects/test-project-123/locations/global/workloadIdentityPools/my-pool/providers/github-provider",
        );
        assert_eq!(
            wif.service_account_email(),
            "ci-sa@test-project-123.iam.gserviceaccount.com",
        );
    }

    #[test]
    fn test_wif_required_resources() {
        let wif = GcpWifConfig::new(
            test_project(),
            "my-pool",
            "github-provider",
            "ci-sa",
        );
        let resources = wif.required_resources();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].kind(), "service_account");
        assert_eq!(resources[0].name(), "ci-sa");
    }

    #[test]
    fn test_cloud_resource_from_gcp() {
        use super::super::CloudResource;

        let project = test_project();
        let bucket = GcpResource::StorageBucket {
            name: "my-bucket".into(),
            location: "us-central1".into(),
        };

        let cloud_resource = CloudResource::gcp(&project, &bucket);
        assert_eq!(
            cloud_resource.resource_id(),
            "cloud:gcp:test-project-123:storage_bucket/my-bucket",
        );
    }
}
