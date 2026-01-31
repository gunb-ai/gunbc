//! Amazon Web Services resource definitions.
//!
//! This module provides typed definitions for AWS resources that can be
//! managed using the DAG upsert pattern.
//!
//! # Supported Resources
//!
//! | Resource | Type | Description |
//! |----------|------|-------------|
//! | IAM Role | `IamRoleDef` | IAM role for workload identity |
//! | IAM Policy | `IamPolicyDef` | IAM policy document |
//! | Secret | `SecretDef` | Secrets Manager secret |
//! | Parameter | `ParameterDef` | SSM Parameter Store parameter |
//!
//! # Architecture
//!
//! ```text
//! AWS Resource Hierarchy
//! ┌─────────────────────────────────┐
//! │ Account                         │
//! │ ├── IAM                         │
//! │ │   ├── Role ───────────────────┼──▶ Web Identity Trust
//! │ │   ├── Policy                  │
//! │ │   └── OIDC Provider           │
//! │ ├── Secrets Manager             │
//! │ │   └── Secrets                 │
//! │ └── SSM Parameter Store         │
//! │     └── Parameters              │
//! └─────────────────────────────────┘
//! ```

pub mod iam;
pub mod secrets;

use serde::{Deserialize, Serialize};

pub use iam::{IamPolicyDef, IamRoleDef, ManagedPolicies, TrustPolicy, TrustPrincipal, TrustStatement};
pub use secrets::{AwsSecretDef, ParameterDef, ParameterType};

// ============================================================================
// AWS-Specific Types
// ============================================================================

/// AWS credential reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AwsCredential {
    /// Access key credentials (via env vars)
    AccessKey {
        /// Environment variable for access key ID
        access_key_id_env: String,
        /// Environment variable for secret access key
        secret_access_key_env: String,
    },
    /// Use default credential chain
    DefaultChain,
    /// Use AWS CLI profile
    Profile(String),
    /// Use Web Identity (OIDC) token
    WebIdentity(super::secrets::AwsWebIdentity),
    /// Use EC2 instance metadata / ECS task role
    InstanceMetadata,
}

impl Default for AwsCredential {
    fn default() -> Self {
        Self::DefaultChain
    }
}

impl AwsCredential {
    /// Create credentials from environment variables.
    pub fn from_env() -> Self {
        Self::AccessKey {
            access_key_id_env: "AWS_ACCESS_KEY_ID".to_string(),
            secret_access_key_env: "AWS_SECRET_ACCESS_KEY".to_string(),
        }
    }

    /// Create credentials from a specific profile.
    pub fn profile(name: impl Into<String>) -> Self {
        Self::Profile(name.into())
    }
}

/// AWS region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AwsRegion {
    /// US East (N. Virginia)
    UsEast1,
    /// US East (Ohio)
    UsEast2,
    /// US West (N. California)
    UsWest1,
    /// US West (Oregon)
    UsWest2,
    /// EU (Ireland)
    EuWest1,
    /// EU (Frankfurt)
    EuCentral1,
    /// Asia Pacific (Tokyo)
    ApNortheast1,
    /// Asia Pacific (Singapore)
    ApSoutheast1,
    /// Custom region
    Custom(String),
}

impl Default for AwsRegion {
    fn default() -> Self {
        Self::UsEast1
    }
}

impl AwsRegion {
    /// Get the region code string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::UsEast1 => "us-east-1",
            Self::UsEast2 => "us-east-2",
            Self::UsWest1 => "us-west-1",
            Self::UsWest2 => "us-west-2",
            Self::EuWest1 => "eu-west-1",
            Self::EuCentral1 => "eu-central-1",
            Self::ApNortheast1 => "ap-northeast-1",
            Self::ApSoutheast1 => "ap-southeast-1",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Create a custom region.
    pub fn custom(region: impl Into<String>) -> Self {
        Self::Custom(region.into())
    }
}

/// AWS resource type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AwsResourceType {
    /// IAM Role
    IamRole,
    /// IAM Policy
    IamPolicy,
    /// IAM OIDC Provider
    IamOidcProvider,
    /// Secrets Manager Secret
    Secret,
    /// SSM Parameter
    Parameter,
}

impl AwsResourceType {
    /// Get the resource type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IamRole => "iam:role",
            Self::IamPolicy => "iam:policy",
            Self::IamOidcProvider => "iam:oidc-provider",
            Self::Secret => "secretsmanager:secret",
            Self::Parameter => "ssm:parameter",
        }
    }

    /// Get the AWS CLI service for this resource.
    pub fn aws_service(&self) -> &'static str {
        match self {
            Self::IamRole | Self::IamPolicy | Self::IamOidcProvider => "iam",
            Self::Secret => "secretsmanager",
            Self::Parameter => "ssm",
        }
    }
}

// ============================================================================
// ARN Formatting
// ============================================================================

/// AWS ARN (Amazon Resource Name) builder.
pub struct Arn;

impl Arn {
    /// Format an IAM role ARN.
    ///
    /// Format: `arn:aws:iam::{account_id}:role/{role_name}`
    pub fn iam_role(account_id: &str, role_name: &str) -> String {
        format!("arn:aws:iam::{}:role/{}", account_id, role_name)
    }

    /// Format an IAM policy ARN.
    ///
    /// Format: `arn:aws:iam::{account_id}:policy/{policy_name}`
    pub fn iam_policy(account_id: &str, policy_name: &str) -> String {
        format!("arn:aws:iam::{}:policy/{}", account_id, policy_name)
    }

    /// Format an IAM OIDC provider ARN.
    ///
    /// Format: `arn:aws:iam::{account_id}:oidc-provider/{provider_url}`
    pub fn iam_oidc_provider(account_id: &str, provider_url: &str) -> String {
        // Remove https:// prefix if present
        let url = provider_url
            .strip_prefix("https://")
            .unwrap_or(provider_url);
        format!("arn:aws:iam::{}:oidc-provider/{}", account_id, url)
    }

    /// Format a Secrets Manager secret ARN.
    ///
    /// Format: `arn:aws:secretsmanager:{region}:{account_id}:secret:{secret_id}`
    pub fn secret(region: &str, account_id: &str, secret_id: &str) -> String {
        format!(
            "arn:aws:secretsmanager:{}:{}:secret:{}",
            region, account_id, secret_id
        )
    }

    /// Format an SSM parameter ARN.
    ///
    /// Format: `arn:aws:ssm:{region}:{account_id}:parameter/{name}`
    pub fn parameter(region: &str, account_id: &str, name: &str) -> String {
        // Remove leading slash if present
        let param_name = name.strip_prefix('/').unwrap_or(name);
        format!(
            "arn:aws:ssm:{}:{}:parameter/{}",
            region, account_id, param_name
        )
    }

    /// Format a GitHub Actions OIDC provider ARN.
    pub fn github_oidc_provider(account_id: &str) -> String {
        Self::iam_oidc_provider(account_id, "token.actions.githubusercontent.com")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_region_as_str() {
        assert_eq!(AwsRegion::UsEast1.as_str(), "us-east-1");
        assert_eq!(AwsRegion::EuCentral1.as_str(), "eu-central-1");
        assert_eq!(AwsRegion::custom("ap-south-1").as_str(), "ap-south-1");
    }

    #[test]
    fn test_resource_type_as_str() {
        assert_eq!(AwsResourceType::IamRole.as_str(), "iam:role");
        assert_eq!(AwsResourceType::Secret.as_str(), "secretsmanager:secret");
    }

    #[test]
    fn test_arn_iam_role() {
        let arn = Arn::iam_role("123456789012", "my-role");
        assert_eq!(arn, "arn:aws:iam::123456789012:role/my-role");
    }

    #[test]
    fn test_arn_iam_policy() {
        let arn = Arn::iam_policy("123456789012", "my-policy");
        assert_eq!(arn, "arn:aws:iam::123456789012:policy/my-policy");
    }

    #[test]
    fn test_arn_oidc_provider() {
        let arn = Arn::iam_oidc_provider(
            "123456789012",
            "https://token.actions.githubusercontent.com",
        );
        assert_eq!(
            arn,
            "arn:aws:iam::123456789012:oidc-provider/token.actions.githubusercontent.com"
        );
    }

    #[test]
    fn test_arn_github_oidc() {
        let arn = Arn::github_oidc_provider("123456789012");
        assert!(arn.contains("token.actions.githubusercontent.com"));
    }

    #[test]
    fn test_arn_secret() {
        let arn = Arn::secret("us-east-1", "123456789012", "my-secret");
        assert_eq!(
            arn,
            "arn:aws:secretsmanager:us-east-1:123456789012:secret:my-secret"
        );
    }

    #[test]
    fn test_arn_parameter() {
        let arn = Arn::parameter("us-east-1", "123456789012", "/my/param");
        assert_eq!(
            arn,
            "arn:aws:ssm:us-east-1:123456789012:parameter/my/param"
        );
    }

    #[test]
    fn test_aws_credential_from_env() {
        let cred = AwsCredential::from_env();
        match cred {
            AwsCredential::AccessKey {
                access_key_id_env,
                secret_access_key_env,
            } => {
                assert_eq!(access_key_id_env, "AWS_ACCESS_KEY_ID");
                assert_eq!(secret_access_key_env, "AWS_SECRET_ACCESS_KEY");
            }
            _ => panic!("Expected AccessKey"),
        }
    }
}
