//! GitHub secrets requirements for gunbc-dag.
//!
//! This module defines the secrets that gunbc-dag workflows need to function,
//! particularly for cloud resource management and workload identity federation.
//!
//! # Secret Categories
//!
//! 1. **GCP Workload Identity** - For keyless authentication to GCP from GitHub Actions
//! 2. **AWS OIDC** - For keyless authentication to AWS from GitHub Actions
//! 3. **Direct Credentials** - Fallback for environments without OIDC support
//!
//! # Usage
//!
//! ```ignore
//! use gunbc_ir::transport::cloud::gunbc_dag_secrets;
//!
//! // Get the requirements for GCP workload identity
//! let gcp_reqs = gunbc_dag_secrets::gcp_workload_identity();
//!
//! // Check if all required secrets are present
//! for secret in &gcp_reqs.required {
//!     println!("Required: {} - {}", secret.name, secret.description);
//! }
//! ```

use super::secrets::{GitHubSecretDef, GitHubSecretsRequirements};

// ============================================================================
// GCP Workload Identity Secrets
// ============================================================================

/// GitHub secrets required for GCP Workload Identity Federation.
///
/// These secrets enable keyless authentication from GitHub Actions to GCP.
pub fn gcp_workload_identity() -> GitHubSecretsRequirements {
    GitHubSecretsRequirements::new()
        .require(
            GitHubSecretDef::repo(
                "GCP_WORKLOAD_IDENTITY_PROVIDER",
                "Full resource name of the Workload Identity Provider",
            )
            .with_example(
                "projects/123456789/locations/global/workloadIdentityPools/github-pool/providers/github",
            ),
        )
        .require(
            GitHubSecretDef::repo(
                "GCP_SERVICE_ACCOUNT",
                "Service account email to impersonate via Workload Identity",
            )
            .with_example("github-actions@my-project.iam.gserviceaccount.com"),
        )
        .optional(
            GitHubSecretDef::repo(
                "GCP_PROJECT_ID",
                "Default GCP project ID for resources",
            )
            .with_example("my-project-12345"),
        )
}

/// GitHub secrets required for GCP Service Account Key authentication.
///
/// This is the fallback method when Workload Identity is not available.
/// Less secure than Workload Identity - keys must be rotated regularly.
pub fn gcp_service_account_key() -> GitHubSecretsRequirements {
    GitHubSecretsRequirements::new()
        .require(
            GitHubSecretDef::repo(
                "GCP_SERVICE_ACCOUNT_KEY",
                "Base64-encoded service account JSON key",
            )
            .with_example("<base64-encoded JSON key>"),
        )
        .optional(
            GitHubSecretDef::repo(
                "GCP_PROJECT_ID",
                "Default GCP project ID for resources",
            )
            .with_example("my-project-12345"),
        )
}

// ============================================================================
// AWS OIDC Secrets
// ============================================================================

/// GitHub secrets required for AWS OIDC authentication.
///
/// These secrets enable keyless authentication from GitHub Actions to AWS.
pub fn aws_oidc() -> GitHubSecretsRequirements {
    GitHubSecretsRequirements::new()
        .require(
            GitHubSecretDef::repo(
                "AWS_ROLE_TO_ASSUME",
                "IAM role ARN to assume via OIDC",
            )
            .with_example("arn:aws:iam::123456789012:role/github-actions-role"),
        )
        .optional(
            GitHubSecretDef::repo(
                "AWS_REGION",
                "Default AWS region for resources",
            )
            .with_example("us-east-1"),
        )
        .optional(
            GitHubSecretDef::repo(
                "AWS_ROLE_SESSION_NAME",
                "Session name for the assumed role",
            )
            .with_example("github-actions-session"),
        )
}

/// GitHub secrets required for AWS access key authentication.
///
/// This is the fallback method when OIDC is not available.
/// Less secure than OIDC - keys must be rotated regularly.
pub fn aws_access_keys() -> GitHubSecretsRequirements {
    GitHubSecretsRequirements::new()
        .require(
            GitHubSecretDef::repo("AWS_ACCESS_KEY_ID", "AWS access key ID"),
        )
        .require(
            GitHubSecretDef::repo("AWS_SECRET_ACCESS_KEY", "AWS secret access key"),
        )
        .optional(
            GitHubSecretDef::repo(
                "AWS_REGION",
                "Default AWS region for resources",
            )
            .with_example("us-east-1"),
        )
}

// ============================================================================
// Combined Requirements for gunbc-dag
// ============================================================================

/// Full GitHub secrets requirements for gunbc-dag cloud operations.
///
/// This combines all the secrets that gunbc-dag might need for cloud
/// resource management.
#[derive(Debug, Clone, Default)]
pub struct GunbcDagSecrets {
    /// GCP secrets (workload identity preferred)
    pub gcp: Option<GitHubSecretsRequirements>,
    /// AWS secrets (OIDC preferred)
    pub aws: Option<GitHubSecretsRequirements>,
    /// Additional custom secrets
    pub custom: Vec<GitHubSecretDef>,
}

impl GunbcDagSecrets {
    /// Create empty requirements.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable GCP Workload Identity.
    pub fn with_gcp_workload_identity(mut self) -> Self {
        self.gcp = Some(gcp_workload_identity());
        self
    }

    /// Enable GCP Service Account Key (fallback).
    pub fn with_gcp_service_account_key(mut self) -> Self {
        self.gcp = Some(gcp_service_account_key());
        self
    }

    /// Enable AWS OIDC.
    pub fn with_aws_oidc(mut self) -> Self {
        self.aws = Some(aws_oidc());
        self
    }

    /// Enable AWS Access Keys (fallback).
    pub fn with_aws_access_keys(mut self) -> Self {
        self.aws = Some(aws_access_keys());
        self
    }

    /// Add a custom secret requirement.
    pub fn with_secret(mut self, secret: GitHubSecretDef) -> Self {
        self.custom.push(secret);
        self
    }

    /// Get all required secrets.
    pub fn all_required(&self) -> Vec<&GitHubSecretDef> {
        let mut required = Vec::new();

        if let Some(ref gcp) = self.gcp {
            required.extend(gcp.required.iter());
        }

        if let Some(ref aws) = self.aws {
            required.extend(aws.required.iter());
        }

        required.extend(self.custom.iter());

        required
    }

    /// Get all optional secrets.
    pub fn all_optional(&self) -> Vec<&GitHubSecretDef> {
        let mut optional = Vec::new();

        if let Some(ref gcp) = self.gcp {
            optional.extend(gcp.optional.iter());
        }

        if let Some(ref aws) = self.aws {
            optional.extend(aws.optional.iter());
        }

        optional
    }

    /// Generate a markdown documentation of required secrets.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# GitHub Secrets Requirements\n\n");

        let required = self.all_required();
        if !required.is_empty() {
            md.push_str("## Required Secrets\n\n");
            md.push_str("| Secret | Description | Example |\n");
            md.push_str("|--------|-------------|--------|\n");
            for secret in required {
                md.push_str(&format!(
                    "| `{}` | {} | {} |\n",
                    secret.name,
                    secret.description,
                    secret.example_format.as_deref().unwrap_or("-")
                ));
            }
            md.push_str("\n");
        }

        let optional = self.all_optional();
        if !optional.is_empty() {
            md.push_str("## Optional Secrets\n\n");
            md.push_str("| Secret | Description | Example |\n");
            md.push_str("|--------|-------------|--------|\n");
            for secret in optional {
                md.push_str(&format!(
                    "| `{}` | {} | {} |\n",
                    secret.name,
                    secret.description,
                    secret.example_format.as_deref().unwrap_or("-")
                ));
            }
            md.push_str("\n");
        }

        md
    }
}

/// Standard gunbc-dag secrets for full cloud support.
///
/// This is the recommended configuration for gunbc-dag repositories
/// that need to manage cloud resources.
pub fn standard() -> GunbcDagSecrets {
    GunbcDagSecrets::new()
        .with_gcp_workload_identity()
        .with_aws_oidc()
}

/// Minimal gunbc-dag secrets (GCP only with workload identity).
pub fn gcp_only() -> GunbcDagSecrets {
    GunbcDagSecrets::new().with_gcp_workload_identity()
}

/// Minimal gunbc-dag secrets (AWS only with OIDC).
pub fn aws_only() -> GunbcDagSecrets {
    GunbcDagSecrets::new().with_aws_oidc()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcp_workload_identity_secrets() {
        let reqs = gcp_workload_identity();
        assert_eq!(reqs.required.len(), 2);
        assert!(reqs
            .required
            .iter()
            .any(|s| s.name == "GCP_WORKLOAD_IDENTITY_PROVIDER"));
        assert!(reqs.required.iter().any(|s| s.name == "GCP_SERVICE_ACCOUNT"));
    }

    #[test]
    fn test_aws_oidc_secrets() {
        let reqs = aws_oidc();
        assert_eq!(reqs.required.len(), 1);
        assert!(reqs.required.iter().any(|s| s.name == "AWS_ROLE_TO_ASSUME"));
    }

    #[test]
    fn test_gunbc_dag_secrets_standard() {
        let secrets = standard();
        assert!(secrets.gcp.is_some());
        assert!(secrets.aws.is_some());

        let required = secrets.all_required();
        assert!(required.len() >= 3); // 2 GCP + 1 AWS minimum
    }

    #[test]
    fn test_gunbc_dag_secrets_to_markdown() {
        let secrets = standard();
        let md = secrets.to_markdown();

        assert!(md.contains("# GitHub Secrets Requirements"));
        assert!(md.contains("GCP_WORKLOAD_IDENTITY_PROVIDER"));
        assert!(md.contains("AWS_ROLE_TO_ASSUME"));
    }

    #[test]
    fn test_gunbc_dag_secrets_with_custom() {
        let secrets = GunbcDagSecrets::new()
            .with_gcp_workload_identity()
            .with_secret(GitHubSecretDef::repo("CUSTOM_SECRET", "A custom secret"));

        assert_eq!(secrets.custom.len(), 1);
        assert!(secrets.custom.iter().any(|s| s.name == "CUSTOM_SECRET"));
    }

    #[test]
    fn test_fallback_secrets() {
        let gcp = gcp_service_account_key();
        assert!(gcp
            .required
            .iter()
            .any(|s| s.name == "GCP_SERVICE_ACCOUNT_KEY"));

        let aws = aws_access_keys();
        assert!(aws.required.iter().any(|s| s.name == "AWS_ACCESS_KEY_ID"));
        assert!(aws
            .required
            .iter()
            .any(|s| s.name == "AWS_SECRET_ACCESS_KEY"));
    }
}
