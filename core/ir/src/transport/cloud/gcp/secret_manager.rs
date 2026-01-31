//! GCP Secret Manager resource definitions.
//!
//! This module provides types for managing secrets in GCP Secret Manager.

use super::{GcpResourceType, ResourceName};
use crate::transport::cloud::{CloudProvider, ResourceHandle, ResourceState};
use crate::transport::ShellRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Secret Definition
// ============================================================================

/// Secret Manager secret definition.
///
/// A secret is a container for secret versions. It defines replication,
/// labels, and access policies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretDef {
    /// GCP project ID
    pub project_id: String,
    /// Secret ID (unique within project)
    pub secret_id: String,
    /// Replication policy
    pub replication: ReplicationPolicy,
    /// Labels for organization
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Secret replication policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReplicationPolicy {
    /// Automatic replication across all regions
    Automatic,
    /// User-managed replication with specific regions
    UserManaged(Vec<String>),
}

impl Default for ReplicationPolicy {
    fn default() -> Self {
        Self::Automatic
    }
}

impl SecretDef {
    /// Create a new secret definition with automatic replication.
    pub fn new(project_id: impl Into<String>, secret_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            secret_id: secret_id.into(),
            replication: ReplicationPolicy::Automatic,
            labels: HashMap::new(),
        }
    }

    /// Set user-managed replication with specific regions.
    pub fn with_regions(mut self, regions: Vec<String>) -> Self {
        self.replication = ReplicationPolicy::UserManaged(regions);
        self
    }

    /// Add a label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Get the fully qualified resource name.
    pub fn resource_name(&self) -> String {
        ResourceName::secret(&self.project_id, &self.secret_id)
    }

    // ========================================================================
    // CLI Commands
    // ========================================================================

    /// Generate the gcloud command to check if this secret exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "secrets",
                "describe",
                &self.secret_id,
                "--project",
                &self.project_id,
                "--format=json",
            ])
    }

    /// Generate the gcloud command to create this secret.
    pub fn create_command(&self) -> ShellRequest {
        let mut args = vec![
            "secrets".to_string(),
            "create".to_string(),
            self.secret_id.clone(),
            "--project".to_string(),
            self.project_id.clone(),
        ];

        match &self.replication {
            ReplicationPolicy::Automatic => {
                args.push("--replication-policy=automatic".to_string());
            }
            ReplicationPolicy::UserManaged(regions) => {
                args.push("--replication-policy=user-managed".to_string());
                args.push("--locations".to_string());
                args.push(regions.join(","));
            }
        }

        for (key, value) in &self.labels {
            args.push("--labels".to_string());
            args.push(format!("{}={}", key, value));
        }

        args.push("--format=json".to_string());

        ShellRequest::new("gcloud").args(args)
    }

    /// Generate the gcloud command to delete this secret.
    pub fn delete_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "secrets",
                "delete",
                &self.secret_id,
                "--project",
                &self.project_id,
                "--quiet",
            ])
    }

    /// Create a resource handle for this secret.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Gcp,
            GcpResourceType::Secret.as_str(),
            self.resource_name(),
            state,
        )
        .with_metadata("secret_id", serde_json::json!(&self.secret_id))
        .with_metadata("project_id", serde_json::json!(&self.project_id))
    }
}

// ============================================================================
// Secret Version Definition
// ============================================================================

/// Secret version definition.
///
/// A secret version contains the actual secret data. Versions are immutable
/// once created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretVersionDef {
    /// Parent secret definition
    pub secret: SecretDef,
    /// Version number (None for latest)
    pub version: Option<u32>,
}

impl SecretVersionDef {
    /// Create a reference to the latest version.
    pub fn latest(secret: SecretDef) -> Self {
        Self {
            secret,
            version: None,
        }
    }

    /// Create a reference to a specific version.
    pub fn specific(secret: SecretDef, version: u32) -> Self {
        Self {
            secret,
            version: Some(version),
        }
    }

    /// Get the version string.
    pub fn version_str(&self) -> String {
        self.version
            .map(|v| v.to_string())
            .unwrap_or_else(|| "latest".to_string())
    }

    /// Get the fully qualified resource name.
    pub fn resource_name(&self) -> String {
        ResourceName::secret_version(
            &self.secret.project_id,
            &self.secret.secret_id,
            &self.version_str(),
        )
    }

    /// Generate the gcloud command to add a new version.
    ///
    /// Note: The secret value should be piped via stdin.
    pub fn add_version_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "secrets",
                "versions",
                "add",
                &self.secret.secret_id,
                "--project",
                &self.secret.project_id,
                "--data-file=-", // Read from stdin
            ])
    }

    /// Generate the gcloud command to access (read) this version.
    pub fn access_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "secrets",
                "versions",
                "access",
                &self.version_str(),
                "--secret",
                &self.secret.secret_id,
                "--project",
                &self.secret.project_id,
            ])
    }

    /// Generate the gcloud command to disable this version.
    pub fn disable_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "secrets",
                "versions",
                "disable",
                &self.version_str(),
                "--secret",
                &self.secret.secret_id,
                "--project",
                &self.secret.project_id,
            ])
    }

    /// Generate the gcloud command to destroy this version.
    pub fn destroy_command(&self) -> ShellRequest {
        ShellRequest::new("gcloud")
            .args([
                "secrets",
                "versions",
                "destroy",
                &self.version_str(),
                "--secret",
                &self.secret.secret_id,
                "--project",
                &self.secret.project_id,
                "--quiet",
            ])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_def_new() {
        let secret = SecretDef::new("my-project", "api-key");
        assert_eq!(secret.project_id, "my-project");
        assert_eq!(secret.secret_id, "api-key");
        assert!(matches!(secret.replication, ReplicationPolicy::Automatic));
    }

    #[test]
    fn test_secret_def_resource_name() {
        let secret = SecretDef::new("my-project", "api-key");
        assert_eq!(secret.resource_name(), "projects/my-project/secrets/api-key");
    }

    #[test]
    fn test_secret_def_with_regions() {
        let secret = SecretDef::new("my-project", "regional-secret")
            .with_regions(vec!["us-central1".to_string(), "us-east1".to_string()]);

        match secret.replication {
            ReplicationPolicy::UserManaged(regions) => {
                assert_eq!(regions.len(), 2);
                assert!(regions.contains(&"us-central1".to_string()));
            }
            _ => panic!("Expected UserManaged replication"),
        }
    }

    #[test]
    fn test_secret_check_command() {
        let secret = SecretDef::new("my-project", "api-key");
        let cmd = secret.check_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"describe".to_string()));
        assert!(cmd.args.contains(&"api-key".to_string()));
        assert!(cmd.args.contains(&"--project".to_string()));
    }

    #[test]
    fn test_secret_create_command_automatic() {
        let secret = SecretDef::new("my-project", "api-key");
        let cmd = secret.create_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"create".to_string()));
        assert!(cmd.args.contains(&"--replication-policy=automatic".to_string()));
    }

    #[test]
    fn test_secret_create_command_user_managed() {
        let secret = SecretDef::new("my-project", "api-key")
            .with_regions(vec!["us-central1".to_string()]);
        let cmd = secret.create_command();

        assert!(cmd.args.contains(&"--replication-policy=user-managed".to_string()));
        assert!(cmd.args.contains(&"--locations".to_string()));
    }

    #[test]
    fn test_secret_version_latest() {
        let secret = SecretDef::new("my-project", "api-key");
        let version = SecretVersionDef::latest(secret);

        assert_eq!(version.version_str(), "latest");
        assert!(version.resource_name().ends_with("/versions/latest"));
    }

    #[test]
    fn test_secret_version_specific() {
        let secret = SecretDef::new("my-project", "api-key");
        let version = SecretVersionDef::specific(secret, 3);

        assert_eq!(version.version_str(), "3");
        assert!(version.resource_name().ends_with("/versions/3"));
    }

    #[test]
    fn test_secret_version_access_command() {
        let secret = SecretDef::new("my-project", "api-key");
        let version = SecretVersionDef::latest(secret);
        let cmd = version.access_command();

        assert_eq!(cmd.command, "gcloud");
        assert!(cmd.args.contains(&"access".to_string()));
        assert!(cmd.args.contains(&"latest".to_string()));
    }

    #[test]
    fn test_secret_handle() {
        let secret = SecretDef::new("my-project", "api-key");
        let handle = secret.to_handle(ResourceState::Active);

        assert_eq!(handle.provider, CloudProvider::Gcp);
        assert_eq!(
            handle.resource_type,
            GcpResourceType::Secret.as_str()
        );
        assert!(handle.state.is_ready());
    }
}
