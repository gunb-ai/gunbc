//! Secret reference modeling.
//!
//! Secrets are never stored as values in the DAG — only references to where
//! secrets live. This module provides [`SecretRef`] for referencing secrets
//! from different sources, and [`SecretFederation`] for describing how to
//! obtain temporary credentials without static keys.
//!
//! # Design Principles
//!
//! 1. **No secret values in the DAG.** Only references (`SecretRef`) flow
//!    through nodes. Actual secret resolution happens at the transport boundary.
//!
//! 2. **Federation over static keys.** Prefer OIDC-based federation
//!    (`SecretFederation`) over long-lived credentials wherever possible.
//!
//! 3. **Source tracking.** Every secret reference knows where it comes from
//!    (`SecretSource`), enabling auditing and rotation planning.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::secret::*;
//!
//! // Reference a GitHub Actions secret
//! let token = SecretRef::github_actions("GCP_WIF_PROVIDER");
//!
//! // Reference an environment variable
//! let api_key = SecretRef::env("API_KEY");
//!
//! // Describe GCP federation
//! let fed = SecretFederation::gcp_wif(
//!     "projects/123/locations/global/workloadIdentityPools/pool/providers/gh",
//!     "sa@project.iam.gserviceaccount.com",
//! );
//! ```

use serde::{Deserialize, Serialize};

/// A reference to a secret (never the value itself).
///
/// `SecretRef` is the unit of secret management in the DAG. It carries
/// enough information to resolve the actual value at the transport boundary,
/// but never contains the secret itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretRef {
    /// Human-readable name for this secret reference.
    pub name: String,
    /// Where the secret comes from.
    pub source: SecretSource,
}

impl SecretRef {
    /// Create a secret reference from a GitHub Actions secret.
    ///
    /// In workflows, this resolves to `${{ secrets.NAME }}`.
    pub fn github_actions(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            source: SecretSource::GitHubActionsSecret { name },
        }
    }

    /// Create a secret reference from an environment variable.
    ///
    /// Resolved at execution time from the process environment.
    pub fn env(var_name: impl Into<String>) -> Self {
        let var_name = var_name.into();
        Self {
            name: var_name.clone(),
            source: SecretSource::EnvVar { var_name },
        }
    }

    /// Create a secret reference from GCP Secret Manager.
    pub fn gcp_secret_manager(
        project: impl Into<String>,
        secret_id: impl Into<String>,
    ) -> Self {
        let secret_id = secret_id.into();
        Self {
            name: secret_id.clone(),
            source: SecretSource::GcpSecretManager {
                project: project.into(),
                secret_id,
                version: "latest".to_string(),
            },
        }
    }

    /// Create a secret reference from AWS Secrets Manager.
    pub fn aws_secrets_manager(
        secret_name: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        let secret_name = secret_name.into();
        Self {
            name: secret_name.clone(),
            source: SecretSource::AwsSecretsManager {
                secret_name,
                region: region.into(),
            },
        }
    }

    /// Create a secret reference from OIDC federation (no static secret).
    ///
    /// The "secret" here is a temporary credential obtained via OIDC token
    /// exchange. There is no stored secret — just a federation configuration.
    pub fn federated(name: impl Into<String>, federation: SecretFederation) -> Self {
        Self {
            name: name.into(),
            source: SecretSource::Federation(federation),
        }
    }

    /// Whether this secret requires a static value (vs. federation).
    pub fn is_static(&self) -> bool {
        !matches!(self.source, SecretSource::Federation(_))
    }

    /// Whether this secret uses federation (OIDC-based, no static key).
    pub fn is_federated(&self) -> bool {
        matches!(self.source, SecretSource::Federation(_))
    }

    /// Get the GitHub Actions expression for this secret, if applicable.
    ///
    /// Returns `Some("${{ secrets.NAME }}")` for GitHub Actions secrets,
    /// or `None` for other sources.
    pub fn github_actions_expression(&self) -> Option<String> {
        match &self.source {
            SecretSource::GitHubActionsSecret { name } => {
                Some(format!("${{{{ secrets.{name} }}}}"))
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for SecretRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "secret:{}", self.name)
    }
}

/// Where a secret comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretSource {
    /// Environment variable (resolved at execution time).
    EnvVar {
        /// Variable name (e.g., "GITHUB_TOKEN").
        var_name: String,
    },

    /// GitHub Actions repository/environment secret.
    GitHubActionsSecret {
        /// Secret name as configured in GitHub settings.
        name: String,
    },

    /// GCP Secret Manager.
    GcpSecretManager {
        /// GCP project ID.
        project: String,
        /// Secret ID within the project.
        secret_id: String,
        /// Secret version (typically "latest").
        version: String,
    },

    /// AWS Secrets Manager.
    AwsSecretsManager {
        /// Secret name or ARN.
        secret_name: String,
        /// AWS region.
        region: String,
    },

    /// Federation-based credential (OIDC token exchange).
    /// No static secret — credentials obtained dynamically via identity federation.
    Federation(SecretFederation),
}

impl SecretSource {
    /// Get a short identifier for the source type.
    pub fn source_kind(&self) -> &'static str {
        match self {
            Self::EnvVar { .. } => "env",
            Self::GitHubActionsSecret { .. } => "github_actions",
            Self::GcpSecretManager { .. } => "gcp_secret_manager",
            Self::AwsSecretsManager { .. } => "aws_secrets_manager",
            Self::Federation(_) => "federation",
        }
    }
}

/// Federation configuration for obtaining temporary credentials.
///
/// Federation eliminates the need for static credentials by using identity
/// tokens (e.g., GitHub Actions OIDC) to obtain temporary cloud credentials.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretFederation {
    /// GCP Workload Identity Federation.
    ///
    /// Uses GitHub Actions OIDC token → GCP STS → temporary GCP credentials.
    /// Requires: `id-token: write` permission in GitHub Actions.
    GcpWorkloadIdentity {
        /// Full provider resource name.
        /// e.g., "projects/P/locations/global/workloadIdentityPools/POOL/providers/PROV"
        workload_identity_provider: String,
        /// Service account to impersonate.
        /// e.g., "sa@project.iam.gserviceaccount.com"
        service_account: String,
    },

    /// AWS OIDC federation.
    ///
    /// Uses GitHub Actions OIDC token → AWS STS → temporary AWS credentials.
    /// Requires: `id-token: write` permission in GitHub Actions.
    AwsOidc {
        /// IAM role ARN to assume.
        /// e.g., "arn:aws:iam::123456789012:role/github-actions"
        role_arn: String,
        /// AWS region for STS endpoint.
        region: String,
    },
}

impl SecretFederation {
    /// Create a GCP Workload Identity Federation configuration.
    pub fn gcp_wif(
        workload_identity_provider: impl Into<String>,
        service_account: impl Into<String>,
    ) -> Self {
        Self::GcpWorkloadIdentity {
            workload_identity_provider: workload_identity_provider.into(),
            service_account: service_account.into(),
        }
    }

    /// Create an AWS OIDC federation configuration.
    pub fn aws_oidc(role_arn: impl Into<String>, region: impl Into<String>) -> Self {
        Self::AwsOidc {
            role_arn: role_arn.into(),
            region: region.into(),
        }
    }

    /// Whether this federation requires `id-token: write` in GitHub Actions.
    pub fn requires_id_token(&self) -> bool {
        // Both GCP WIF and AWS OIDC require OIDC token generation
        true
    }

    /// Get the cloud provider for this federation.
    pub fn provider(&self) -> super::cloud::CloudProvider {
        match self {
            Self::GcpWorkloadIdentity { .. } => super::cloud::CloudProvider::Gcp,
            Self::AwsOidc { .. } => super::cloud::CloudProvider::Aws,
        }
    }
}

/// A set of secrets required by a workflow or operation.
///
/// Tracks all secret references needed, enabling:
/// - Pre-flight validation (are all secrets available?)
/// - GitHub Actions secret configuration
/// - Audit of secret dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretRequirements {
    /// All required secret references.
    pub secrets: Vec<SecretRef>,
}

impl SecretRequirements {
    /// Create an empty set of requirements.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a secret requirement.
    pub fn require(&mut self, secret: SecretRef) {
        if !self.secrets.iter().any(|s| s.name == secret.name) {
            self.secrets.push(secret);
        }
    }

    /// Get all GitHub Actions secrets needed.
    pub fn github_actions_secrets(&self) -> Vec<&SecretRef> {
        self.secrets
            .iter()
            .filter(|s| matches!(s.source, SecretSource::GitHubActionsSecret { .. }))
            .collect()
    }

    /// Get all federated credentials needed.
    pub fn federated_secrets(&self) -> Vec<&SecretRef> {
        self.secrets
            .iter()
            .filter(|s| s.is_federated())
            .collect()
    }

    /// Get all static secrets (non-federated).
    pub fn static_secrets(&self) -> Vec<&SecretRef> {
        self.secrets.iter().filter(|s| s.is_static()).collect()
    }

    /// Check if any federation requires `id-token: write` permission.
    pub fn requires_id_token_permission(&self) -> bool {
        self.secrets.iter().any(|s| {
            matches!(
                &s.source,
                SecretSource::Federation(fed) if fed.requires_id_token()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_ref_github_actions() {
        let secret = SecretRef::github_actions("MY_TOKEN");
        assert_eq!(secret.name, "MY_TOKEN");
        assert!(secret.is_static());
        assert!(!secret.is_federated());
        assert_eq!(
            secret.github_actions_expression(),
            Some("${{ secrets.MY_TOKEN }}".to_string()),
        );
    }

    #[test]
    fn test_secret_ref_env() {
        let secret = SecretRef::env("API_KEY");
        assert_eq!(secret.name, "API_KEY");
        assert!(secret.is_static());
        assert_eq!(secret.github_actions_expression(), None);
    }

    #[test]
    fn test_secret_ref_gcp_secret_manager() {
        let secret = SecretRef::gcp_secret_manager("my-project", "db-password");
        assert_eq!(secret.name, "db-password");
        assert!(secret.is_static());
        match &secret.source {
            SecretSource::GcpSecretManager {
                project,
                secret_id,
                version,
            } => {
                assert_eq!(project, "my-project");
                assert_eq!(secret_id, "db-password");
                assert_eq!(version, "latest");
            }
            _ => panic!("Expected GcpSecretManager"),
        }
    }

    #[test]
    fn test_secret_ref_aws_secrets_manager() {
        let secret = SecretRef::aws_secrets_manager("db-password", "us-east-1");
        assert_eq!(secret.name, "db-password");
        assert!(secret.is_static());
    }

    #[test]
    fn test_secret_ref_federated() {
        let fed = SecretFederation::gcp_wif(
            "projects/p/locations/global/workloadIdentityPools/pool/providers/prov",
            "sa@p.iam.gserviceaccount.com",
        );
        let secret = SecretRef::federated("gcp-access", fed);
        assert!(!secret.is_static());
        assert!(secret.is_federated());
        assert_eq!(secret.github_actions_expression(), None);
    }

    #[test]
    fn test_secret_source_kind() {
        assert_eq!(
            SecretSource::EnvVar { var_name: "X".into() }.source_kind(),
            "env",
        );
        assert_eq!(
            SecretSource::GitHubActionsSecret { name: "X".into() }.source_kind(),
            "github_actions",
        );
    }

    #[test]
    fn test_secret_federation_provider() {
        let gcp = SecretFederation::gcp_wif("p", "sa");
        assert_eq!(gcp.provider(), super::super::cloud::CloudProvider::Gcp);

        let aws = SecretFederation::aws_oidc("arn", "us-east-1");
        assert_eq!(aws.provider(), super::super::cloud::CloudProvider::Aws);
    }

    #[test]
    fn test_secret_federation_requires_id_token() {
        let gcp = SecretFederation::gcp_wif("p", "sa");
        assert!(gcp.requires_id_token());

        let aws = SecretFederation::aws_oidc("arn", "us-east-1");
        assert!(aws.requires_id_token());
    }

    #[test]
    fn test_secret_requirements() {
        let mut reqs = SecretRequirements::new();
        reqs.require(SecretRef::github_actions("TOKEN_A"));
        reqs.require(SecretRef::env("LOCAL_KEY"));
        reqs.require(SecretRef::federated(
            "gcp",
            SecretFederation::gcp_wif("p", "sa"),
        ));

        assert_eq!(reqs.secrets.len(), 3);
        assert_eq!(reqs.github_actions_secrets().len(), 1);
        assert_eq!(reqs.federated_secrets().len(), 1);
        assert_eq!(reqs.static_secrets().len(), 2);
        assert!(reqs.requires_id_token_permission());
    }

    #[test]
    fn test_secret_requirements_dedup() {
        let mut reqs = SecretRequirements::new();
        reqs.require(SecretRef::github_actions("TOKEN_A"));
        reqs.require(SecretRef::github_actions("TOKEN_A")); // duplicate
        assert_eq!(reqs.secrets.len(), 1);
    }

    #[test]
    fn test_secret_ref_display() {
        let secret = SecretRef::github_actions("MY_TOKEN");
        assert_eq!(secret.to_string(), "secret:MY_TOKEN");
    }
}
