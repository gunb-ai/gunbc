//! Provider-agnostic infrastructure scope abstraction.
//!
//! Following the-gunbai's pattern: abstract access requirements resolve
//! to provider-specific grants (GCP IAM roles, AWS IAM policies, etc.).
//!
//! This enables the credential lifecycle to express "I need secret read access"
//! without hardwiring to a specific cloud provider's IAM system.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Abstract scope types
// ---------------------------------------------------------------------------

/// Type of infrastructure resource being accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InfraScopeType {
    /// Secret management (e.g., Secret Manager, AWS Secrets Manager, Key Vault).
    Secret,
    /// Identity / service account operations.
    Identity,
    /// Workload Identity Federation / keyless auth.
    Federation,
    /// Object storage (e.g., GCS, S3, Azure Blob).
    Storage,
    /// Compute resources (VMs, containers, functions).
    Compute,
    /// API gateway / endpoint access.
    Api,
}

/// Level of access required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InfraAccessLevel {
    /// Read-only access (e.g., secretmanager.versions.access).
    Read,
    /// Read-write access (e.g., secretmanager.secrets.create).
    Write,
    /// Full administrative access (e.g., secretmanager.secrets.setIamPolicy).
    Admin,
}

/// Provider-agnostic infrastructure access requirement.
///
/// Declares WHAT access is needed without specifying HOW to get it.
/// Resolved to provider-specific grants via `resolve_gcp()`, `resolve_aws()`, etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfraScope {
    /// What kind of resource is being accessed.
    pub scope_type: InfraScopeType,
    /// What level of access is needed.
    pub access_level: InfraAccessLevel,
    /// Resource pattern (e.g., "projects/*/secrets/*" or "buckets/my-bucket").
    pub resource_pattern: String,
}

// ---------------------------------------------------------------------------
// GCP-specific resolution
// ---------------------------------------------------------------------------

/// GCP-specific resolution of an abstract InfraScope.
///
/// Maps an abstract access requirement to concrete GCP IAM constructs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpScope {
    /// Concrete GCP resource path (e.g., "projects/my-project/secrets/my-secret").
    pub resource_path: String,
    /// IAM role granting the required access (e.g., "roles/secretmanager.secretAccessor").
    pub iam_role: String,
    /// Individual IAM permissions required (e.g., ["secretmanager.versions.access"]).
    pub permissions: Vec<String>,
}

impl InfraScope {
    /// Create a new infrastructure scope.
    pub fn new(
        scope_type: InfraScopeType,
        access_level: InfraAccessLevel,
        resource_pattern: impl Into<String>,
    ) -> Self {
        Self {
            scope_type,
            access_level,
            resource_pattern: resource_pattern.into(),
        }
    }

    /// Resolve this abstract scope to GCP-specific IAM grants.
    ///
    /// The `project` parameter is used to expand resource patterns
    /// (e.g., "secrets/*" -> "projects/{project}/secrets/*").
    pub fn resolve_gcp(&self, project: &str) -> GcpScope {
        match self.scope_type {
            InfraScopeType::Secret => self.resolve_gcp_secret(project),
            InfraScopeType::Identity => self.resolve_gcp_identity(project),
            InfraScopeType::Federation => self.resolve_gcp_federation(project),
            InfraScopeType::Storage => self.resolve_gcp_storage(project),
            InfraScopeType::Compute => self.resolve_gcp_compute(project),
            InfraScopeType::Api => self.resolve_gcp_api(project),
        }
    }

    fn resolve_gcp_secret(&self, project: &str) -> GcpScope {
        let resource_path = format!("projects/{}/secrets/{}", project, self.resource_pattern);
        let (iam_role, permissions) = match self.access_level {
            InfraAccessLevel::Read => (
                "roles/secretmanager.secretAccessor",
                vec!["secretmanager.versions.access".to_string()],
            ),
            InfraAccessLevel::Write => (
                "roles/secretmanager.secretVersionAdder",
                vec![
                    "secretmanager.secrets.create".to_string(),
                    "secretmanager.versions.add".to_string(),
                ],
            ),
            InfraAccessLevel::Admin => (
                "roles/secretmanager.admin",
                vec![
                    "secretmanager.secrets.create".to_string(),
                    "secretmanager.secrets.delete".to_string(),
                    "secretmanager.secrets.setIamPolicy".to_string(),
                    "secretmanager.versions.add".to_string(),
                    "secretmanager.versions.access".to_string(),
                ],
            ),
        };
        GcpScope {
            resource_path,
            iam_role: iam_role.to_string(),
            permissions,
        }
    }

    fn resolve_gcp_identity(&self, project: &str) -> GcpScope {
        let resource_path = format!(
            "projects/{}/serviceAccounts/{}",
            project, self.resource_pattern
        );
        let (iam_role, permissions) = match self.access_level {
            InfraAccessLevel::Read => (
                "roles/iam.serviceAccountUser",
                vec!["iam.serviceAccounts.actAs".to_string()],
            ),
            InfraAccessLevel::Write => (
                "roles/iam.serviceAccountTokenCreator",
                vec![
                    "iam.serviceAccounts.actAs".to_string(),
                    "iam.serviceAccounts.getAccessToken".to_string(),
                    "iam.serviceAccounts.signBlob".to_string(),
                ],
            ),
            InfraAccessLevel::Admin => (
                "roles/iam.serviceAccountAdmin",
                vec![
                    "iam.serviceAccounts.create".to_string(),
                    "iam.serviceAccounts.delete".to_string(),
                    "iam.serviceAccounts.setIamPolicy".to_string(),
                ],
            ),
        };
        GcpScope {
            resource_path,
            iam_role: iam_role.to_string(),
            permissions,
        }
    }

    fn resolve_gcp_federation(&self, project: &str) -> GcpScope {
        let resource_path = format!(
            "projects/{}/locations/global/workloadIdentityPools/{}",
            project, self.resource_pattern
        );
        let (iam_role, permissions) = match self.access_level {
            InfraAccessLevel::Read => (
                "roles/iam.workloadIdentityPoolViewer",
                vec!["iam.workloadIdentityPools.get".to_string()],
            ),
            InfraAccessLevel::Write | InfraAccessLevel::Admin => (
                "roles/iam.workloadIdentityPoolAdmin",
                vec![
                    "iam.workloadIdentityPools.create".to_string(),
                    "iam.workloadIdentityPools.delete".to_string(),
                    "iam.workloadIdentityPoolProviders.create".to_string(),
                ],
            ),
        };
        GcpScope {
            resource_path,
            iam_role: iam_role.to_string(),
            permissions,
        }
    }

    fn resolve_gcp_storage(&self, _project: &str) -> GcpScope {
        let resource_path = format!("b/{}", self.resource_pattern);
        let (iam_role, permissions) = match self.access_level {
            InfraAccessLevel::Read => (
                "roles/storage.objectViewer",
                vec!["storage.objects.get".to_string()],
            ),
            InfraAccessLevel::Write => (
                "roles/storage.objectUser",
                vec![
                    "storage.objects.get".to_string(),
                    "storage.objects.create".to_string(),
                    "storage.objects.delete".to_string(),
                ],
            ),
            InfraAccessLevel::Admin => (
                "roles/storage.admin",
                vec![
                    "storage.buckets.create".to_string(),
                    "storage.buckets.delete".to_string(),
                    "storage.objects.get".to_string(),
                    "storage.objects.create".to_string(),
                ],
            ),
        };
        GcpScope {
            resource_path,
            iam_role: iam_role.to_string(),
            permissions,
        }
    }

    fn resolve_gcp_compute(&self, project: &str) -> GcpScope {
        let resource_path = format!("projects/{}/{}", project, self.resource_pattern);
        let (iam_role, permissions) = match self.access_level {
            InfraAccessLevel::Read => (
                "roles/compute.viewer",
                vec!["compute.instances.list".to_string()],
            ),
            InfraAccessLevel::Write => (
                "roles/compute.instanceAdmin.v1",
                vec![
                    "compute.instances.create".to_string(),
                    "compute.instances.delete".to_string(),
                ],
            ),
            InfraAccessLevel::Admin => (
                "roles/compute.admin",
                vec![
                    "compute.instances.create".to_string(),
                    "compute.instances.delete".to_string(),
                    "compute.networks.create".to_string(),
                ],
            ),
        };
        GcpScope {
            resource_path,
            iam_role: iam_role.to_string(),
            permissions,
        }
    }

    fn resolve_gcp_api(&self, project: &str) -> GcpScope {
        let resource_path = format!("projects/{}/services/{}", project, self.resource_pattern);
        let (iam_role, permissions) = match self.access_level {
            InfraAccessLevel::Read | InfraAccessLevel::Write | InfraAccessLevel::Admin => (
                "roles/serviceusage.serviceUsageConsumer",
                vec!["serviceusage.services.use".to_string()],
            ),
        };
        GcpScope {
            resource_path,
            iam_role: iam_role.to_string(),
            permissions,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_read_scope() {
        let scope = InfraScope::new(InfraScopeType::Secret, InfraAccessLevel::Read, "my-secret");
        let gcp = scope.resolve_gcp("my-project");
        assert_eq!(gcp.resource_path, "projects/my-project/secrets/my-secret");
        assert_eq!(gcp.iam_role, "roles/secretmanager.secretAccessor");
        assert!(gcp
            .permissions
            .contains(&"secretmanager.versions.access".to_string()));
    }

    #[test]
    fn test_secret_write_scope() {
        let scope = InfraScope::new(InfraScopeType::Secret, InfraAccessLevel::Write, "*");
        let gcp = scope.resolve_gcp("my-project");
        assert_eq!(gcp.iam_role, "roles/secretmanager.secretVersionAdder");
        assert!(gcp
            .permissions
            .contains(&"secretmanager.secrets.create".to_string()));
    }

    #[test]
    fn test_identity_scope() {
        let scope = InfraScope::new(
            InfraScopeType::Identity,
            InfraAccessLevel::Write,
            "sa@project.iam.gserviceaccount.com",
        );
        let gcp = scope.resolve_gcp("my-project");
        assert_eq!(gcp.iam_role, "roles/iam.serviceAccountTokenCreator");
        assert!(gcp
            .permissions
            .contains(&"iam.serviceAccounts.getAccessToken".to_string()));
    }

    #[test]
    fn test_federation_scope() {
        let scope = InfraScope::new(
            InfraScopeType::Federation,
            InfraAccessLevel::Read,
            "github-pool",
        );
        let gcp = scope.resolve_gcp("my-project");
        assert!(gcp
            .resource_path
            .contains("workloadIdentityPools/github-pool"));
        assert_eq!(gcp.iam_role, "roles/iam.workloadIdentityPoolViewer");
    }

    #[test]
    fn test_storage_scope() {
        let scope = InfraScope::new(
            InfraScopeType::Storage,
            InfraAccessLevel::Write,
            "my-bucket",
        );
        let gcp = scope.resolve_gcp("my-project");
        assert_eq!(gcp.resource_path, "b/my-bucket");
        assert_eq!(gcp.iam_role, "roles/storage.objectUser");
    }

    #[test]
    fn test_infra_scope_serde_roundtrip() {
        let scope = InfraScope::new(InfraScopeType::Secret, InfraAccessLevel::Read, "*");
        let json = serde_json::to_string(&scope).unwrap();
        let parsed: InfraScope = serde_json::from_str(&json).unwrap();
        assert_eq!(scope, parsed);
    }
}
