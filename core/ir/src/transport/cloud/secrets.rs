//! Secret management and federation.
//!
//! This module provides abstractions for:
//! - Secret references (without embedding secret values)
//! - Workload Identity Federation (keyless authentication)
//! - GitHub Actions OIDC integration
//!
//! # Design Principles
//!
//! 1. **No Embedded Secrets**: Secret values are NEVER stored in code or DAG structures.
//!    Only references (environment variable names, secret paths) are stored.
//!
//! 2. **Workload Identity First**: For CI/CD, prefer workload identity federation over
//!    long-lived credentials. This eliminates secret rotation and reduces blast radius.
//!
//! 3. **Declarative Federation**: Workload identity configurations are modeled as data,
//!    not imperative setup scripts.
//!
//! # GitHub Actions OIDC
//!
//! GitHub Actions can authenticate to GCP and AWS using OIDC, without storing credentials:
//!
//! ```text
//! GitHub Actions                         Cloud Provider
//! ┌─────────────────┐                   ┌─────────────────┐
//! │ Workflow Job    │                   │ Workload Pool   │
//! │                 │ ─── OIDC Token ──▶│                 │
//! │ GITHUB_TOKEN    │                   │ Attribute Map   │
//! └─────────────────┘                   └────────┬────────┘
//!                                                │
//!                                                ▼
//!                                       ┌─────────────────┐
//!                                       │ Service Account │
//!                                       │ (impersonated)  │
//!                                       └─────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Secret References
// ============================================================================

/// Reference to a secret value.
///
/// Secrets are never embedded in DAG structures. Instead, we store references
/// that are resolved at execution time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecretRef {
    /// Environment variable containing the secret value
    EnvVar(String),

    /// GitHub Actions secret
    GitHubSecret(String),

    /// GCP Secret Manager reference
    GcpSecretManager {
        project: String,
        secret_id: String,
        version: SecretVersion,
    },

    /// AWS Secrets Manager reference
    AwsSecretsManager {
        secret_id: String,
        region: Option<String>,
        version_stage: Option<String>,
    },

    /// AWS Systems Manager Parameter Store
    AwsParameterStore {
        name: String,
        region: Option<String>,
        with_decryption: bool,
    },

    /// HashiCorp Vault reference
    Vault {
        path: String,
        key: Option<String>,
    },
}

/// Secret version specifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecretVersion {
    /// Latest version
    Latest,
    /// Specific version number
    Version(u32),
    /// Version by alias
    Alias(String),
}

impl Default for SecretVersion {
    fn default() -> Self {
        Self::Latest
    }
}

impl SecretRef {
    /// Create a reference to an environment variable.
    pub fn env(name: impl Into<String>) -> Self {
        Self::EnvVar(name.into())
    }

    /// Create a reference to a GitHub Actions secret.
    pub fn github_secret(name: impl Into<String>) -> Self {
        Self::GitHubSecret(name.into())
    }

    /// Create a reference to a GCP Secret Manager secret.
    pub fn gcp_secret(project: impl Into<String>, secret_id: impl Into<String>) -> Self {
        Self::GcpSecretManager {
            project: project.into(),
            secret_id: secret_id.into(),
            version: SecretVersion::Latest,
        }
    }

    /// Create a reference to an AWS Secrets Manager secret.
    pub fn aws_secret(secret_id: impl Into<String>) -> Self {
        Self::AwsSecretsManager {
            secret_id: secret_id.into(),
            region: None,
            version_stage: None,
        }
    }
}

// ============================================================================
// Secret Source (for DAG modeling)
// ============================================================================

/// Source of secrets for a workflow/DAG.
///
/// This models where secrets come from and how they should be resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretSource {
    /// Name of the secret (for documentation/logging)
    pub name: String,
    /// Reference to the actual secret value
    pub reference: SecretRef,
    /// Description of what this secret is for
    pub description: Option<String>,
    /// Whether this secret is required
    pub required: bool,
}

impl SecretSource {
    /// Create a required secret source.
    pub fn required(name: impl Into<String>, reference: SecretRef) -> Self {
        Self {
            name: name.into(),
            reference,
            description: None,
            required: true,
        }
    }

    /// Create an optional secret source.
    pub fn optional(name: impl Into<String>, reference: SecretRef) -> Self {
        Self {
            name: name.into(),
            reference,
            description: None,
            required: false,
        }
    }

    /// Add a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

// ============================================================================
// Workload Identity Federation
// ============================================================================

/// Workload Identity Federation configuration.
///
/// This enables keyless authentication from external identity providers
/// (like GitHub Actions) to cloud providers (like GCP or AWS).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkloadIdentityConfig {
    /// Target cloud provider
    pub provider: super::CloudProvider,
    /// Provider-specific configuration
    pub config: WorkloadIdentityProvider,
}

/// Provider-specific workload identity configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkloadIdentityProvider {
    /// GCP Workload Identity Federation
    Gcp(GcpWorkloadIdentity),
    /// AWS Web Identity / IRSA
    Aws(AwsWebIdentity),
}

/// GCP Workload Identity Federation configuration.
///
/// This allows GitHub Actions (or other OIDC providers) to authenticate
/// to GCP without storing service account keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GcpWorkloadIdentity {
    /// GCP project ID
    pub project_id: String,

    /// Workload Identity Pool ID
    /// Format: projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}
    pub pool_id: String,

    /// Workload Identity Provider ID
    /// Format: projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}/providers/{provider_id}
    pub provider_id: String,

    /// Service account to impersonate
    /// Format: {account_id}@{project_id}.iam.gserviceaccount.com
    pub service_account: String,

    /// Attribute mapping from OIDC token to GCP attributes
    /// Example: {"google.subject": "assertion.sub", "attribute.repository": "assertion.repository"}
    #[serde(default)]
    pub attribute_mapping: HashMap<String, String>,

    /// Attribute condition CEL expression
    /// Example: "assertion.repository == 'owner/repo'"
    pub attribute_condition: Option<String>,
}

impl GcpWorkloadIdentity {
    /// Create a new GCP Workload Identity configuration.
    pub fn new(
        project_id: impl Into<String>,
        pool_id: impl Into<String>,
        provider_id: impl Into<String>,
        service_account: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            pool_id: pool_id.into(),
            provider_id: provider_id.into(),
            service_account: service_account.into(),
            attribute_mapping: HashMap::new(),
            attribute_condition: None,
        }
    }

    /// Create configuration for GitHub Actions OIDC.
    ///
    /// This sets up the standard attribute mapping for GitHub Actions.
    pub fn for_github_actions(
        project_id: impl Into<String>,
        project_number: u64,
        pool_id: impl Into<String>,
        provider_id: impl Into<String>,
        service_account: impl Into<String>,
        repository: impl Into<String>,
    ) -> Self {
        let project_id = project_id.into();
        let pool_id_str = pool_id.into();
        let provider_id_str = provider_id.into();

        let mut mapping = HashMap::new();
        mapping.insert("google.subject".to_string(), "assertion.sub".to_string());
        mapping.insert(
            "attribute.actor".to_string(),
            "assertion.actor".to_string(),
        );
        mapping.insert(
            "attribute.repository".to_string(),
            "assertion.repository".to_string(),
        );
        mapping.insert(
            "attribute.repository_owner".to_string(),
            "assertion.repository_owner".to_string(),
        );

        Self {
            project_id: project_id.clone(),
            pool_id: format!(
                "projects/{}/locations/global/workloadIdentityPools/{}",
                project_number, pool_id_str
            ),
            provider_id: format!(
                "projects/{}/locations/global/workloadIdentityPools/{}/providers/{}",
                project_number, pool_id_str, provider_id_str
            ),
            service_account: service_account.into(),
            attribute_mapping: mapping,
            attribute_condition: Some(format!(
                "assertion.repository == '{}'",
                repository.into()
            )),
        }
    }

    /// Add an attribute mapping.
    pub fn with_attribute(mut self, gcp_attr: impl Into<String>, oidc_claim: impl Into<String>) -> Self {
        self.attribute_mapping
            .insert(gcp_attr.into(), oidc_claim.into());
        self
    }

    /// Set the attribute condition.
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.attribute_condition = Some(condition.into());
        self
    }

    /// Generate the credential configuration file content (JSON).
    ///
    /// This is the format used by `gcloud auth login --cred-file`.
    pub fn credential_config(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "external_account",
            "audience": format!("//iam.googleapis.com/{}", self.provider_id),
            "subject_token_type": "urn:ietf:params:oauth:token-type:jwt",
            "token_url": "https://sts.googleapis.com/v1/token",
            "credential_source": {
                "file": "/var/run/secrets/tokens/gcp-token",
                "format": {
                    "type": "text"
                }
            },
            "service_account_impersonation_url": format!(
                "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{}:generateAccessToken",
                self.service_account
            )
        })
    }
}

/// AWS Web Identity configuration for workload identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwsWebIdentity {
    /// IAM role ARN to assume
    /// Format: arn:aws:iam::{account_id}:role/{role_name}
    pub role_arn: String,

    /// Session name for the assumed role
    pub role_session_name: Option<String>,

    /// OIDC provider ARN
    /// For GitHub: arn:aws:iam::{account_id}:oidc-provider/token.actions.githubusercontent.com
    pub web_identity_token_source: WebIdentityTokenSource,

    /// AWS region for STS endpoint
    pub region: Option<String>,
}

/// Source of the web identity token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebIdentityTokenSource {
    /// Token from environment variable
    EnvVar(String),
    /// Token from file
    File(String),
    /// GitHub Actions OIDC token (automatic)
    GitHubActions,
}

impl AwsWebIdentity {
    /// Create configuration for GitHub Actions OIDC.
    pub fn for_github_actions(role_arn: impl Into<String>) -> Self {
        Self {
            role_arn: role_arn.into(),
            role_session_name: Some("github-actions".to_string()),
            web_identity_token_source: WebIdentityTokenSource::GitHubActions,
            region: None,
        }
    }

    /// Set the session name.
    pub fn with_session_name(mut self, name: impl Into<String>) -> Self {
        self.role_session_name = Some(name.into());
        self
    }

    /// Set the AWS region.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }
}

// ============================================================================
// GitHub Secrets Modeling
// ============================================================================

/// Required GitHub secrets for a workflow.
///
/// This models what secrets a workflow needs from GitHub Actions secrets.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GitHubSecretsRequirements {
    /// Required secrets (workflow will fail without these)
    pub required: Vec<GitHubSecretDef>,
    /// Optional secrets (workflow can run without these)
    pub optional: Vec<GitHubSecretDef>,
}

/// Definition of a GitHub Actions secret.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitHubSecretDef {
    /// Secret name (e.g., "GCP_WORKLOAD_IDENTITY_PROVIDER")
    pub name: String,
    /// Description of what this secret contains
    pub description: String,
    /// Example value format (for documentation, NOT the actual value)
    pub example_format: Option<String>,
    /// Whether this is a repository or organization secret
    pub scope: SecretScope,
}

/// Scope of a GitHub secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretScope {
    /// Repository-level secret
    Repository,
    /// Organization-level secret
    Organization,
    /// Environment-specific secret
    Environment,
}

impl GitHubSecretDef {
    /// Create a required repository secret.
    pub fn repo(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            example_format: None,
            scope: SecretScope::Repository,
        }
    }

    /// Create an organization secret.
    pub fn org(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            example_format: None,
            scope: SecretScope::Organization,
        }
    }

    /// Add an example format.
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example_format = Some(example.into());
        self
    }
}

impl GitHubSecretsRequirements {
    /// Create empty requirements.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a required secret.
    pub fn require(mut self, secret: GitHubSecretDef) -> Self {
        self.required.push(secret);
        self
    }

    /// Add an optional secret.
    pub fn optional(mut self, secret: GitHubSecretDef) -> Self {
        self.optional.push(secret);
        self
    }

    /// Define standard GCP workload identity secrets.
    pub fn gcp_workload_identity() -> Self {
        Self::new()
            .require(
                GitHubSecretDef::repo(
                    "GCP_WORKLOAD_IDENTITY_PROVIDER",
                    "Full provider ID for workload identity federation",
                )
                .with_example(
                    "projects/123456789/locations/global/workloadIdentityPools/my-pool/providers/github",
                ),
            )
            .require(
                GitHubSecretDef::repo(
                    "GCP_SERVICE_ACCOUNT",
                    "Service account email to impersonate",
                )
                .with_example("my-sa@my-project.iam.gserviceaccount.com"),
            )
    }

    /// Define standard AWS OIDC secrets.
    pub fn aws_oidc() -> Self {
        Self::new().require(
            GitHubSecretDef::repo("AWS_ROLE_TO_ASSUME", "IAM role ARN to assume via OIDC")
                .with_example("arn:aws:iam::123456789012:role/github-actions-role"),
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
    fn test_secret_ref_env() {
        let secret = SecretRef::env("MY_SECRET");
        assert!(matches!(secret, SecretRef::EnvVar(s) if s == "MY_SECRET"));
    }

    #[test]
    fn test_secret_ref_github() {
        let secret = SecretRef::github_secret("GCP_CREDENTIALS");
        assert!(matches!(secret, SecretRef::GitHubSecret(s) if s == "GCP_CREDENTIALS"));
    }

    #[test]
    fn test_secret_ref_gcp() {
        let secret = SecretRef::gcp_secret("my-project", "api-key");
        match secret {
            SecretRef::GcpSecretManager {
                project,
                secret_id,
                version,
            } => {
                assert_eq!(project, "my-project");
                assert_eq!(secret_id, "api-key");
                assert!(matches!(version, SecretVersion::Latest));
            }
            _ => panic!("Expected GcpSecretManager"),
        }
    }

    #[test]
    fn test_gcp_workload_identity_for_github() {
        let config = GcpWorkloadIdentity::for_github_actions(
            "my-project",
            123456789,
            "github-pool",
            "github-provider",
            "github-sa@my-project.iam.gserviceaccount.com",
            "owner/repo",
        );

        assert_eq!(config.project_id, "my-project");
        assert!(config.pool_id.contains("github-pool"));
        assert!(config.provider_id.contains("github-provider"));
        assert!(config.attribute_mapping.contains_key("google.subject"));
        assert!(config
            .attribute_condition
            .as_ref()
            .unwrap()
            .contains("owner/repo"));
    }

    #[test]
    fn test_gcp_credential_config() {
        let config = GcpWorkloadIdentity::new(
            "my-project",
            "projects/123/locations/global/workloadIdentityPools/my-pool",
            "projects/123/locations/global/workloadIdentityPools/my-pool/providers/github",
            "sa@my-project.iam.gserviceaccount.com",
        );

        let cred_config = config.credential_config();
        assert_eq!(cred_config["type"], "external_account");
        assert!(cred_config["audience"]
            .as_str()
            .unwrap()
            .contains("my-pool"));
    }

    #[test]
    fn test_aws_web_identity_for_github() {
        let config = AwsWebIdentity::for_github_actions("arn:aws:iam::123456789012:role/my-role")
            .with_region("us-east-1");

        assert_eq!(config.role_arn, "arn:aws:iam::123456789012:role/my-role");
        assert_eq!(config.region, Some("us-east-1".to_string()));
        assert!(matches!(
            config.web_identity_token_source,
            WebIdentityTokenSource::GitHubActions
        ));
    }

    #[test]
    fn test_github_secrets_requirements() {
        let reqs = GitHubSecretsRequirements::gcp_workload_identity();
        assert_eq!(reqs.required.len(), 2);
        assert!(reqs
            .required
            .iter()
            .any(|s| s.name == "GCP_WORKLOAD_IDENTITY_PROVIDER"));
        assert!(reqs
            .required
            .iter()
            .any(|s| s.name == "GCP_SERVICE_ACCOUNT"));
    }
}
