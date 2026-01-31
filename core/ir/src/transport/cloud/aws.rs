//! AWS (Amazon Web Services) resource modeling.
//!
//! Models AWS resources that can be upserted: checked for existence,
//! created if missing, and resolved to a handle.
//!
//! # Resources Modeled
//!
//! - **S3 Buckets**: Object storage
//! - **IAM Roles**: Identity for workloads and services
//! - **Secrets Manager Secrets**: Secret storage
//! - **ECR Repositories**: Container image registry
//!
//! # Authentication
//!
//! AWS authentication from GitHub Actions uses OIDC federation via
//! `aws-actions/configure-aws-credentials`. This is modeled in
//! [`AwsOidcConfig`].
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::transport::cloud::aws::*;
//!
//! let account = AwsAccount::new("123456789012", "us-east-1");
//! let bucket = AwsResource::S3Bucket {
//!     name: "my-data-bucket".into(),
//!     region: None, // inherit from account
//! };
//!
//! let check = bucket.check_command(&account);
//! let create = bucket.create_command(&account);
//! ```

use serde::{Deserialize, Serialize};

/// AWS account and region scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AwsAccount {
    /// AWS account ID (12-digit number).
    pub id: String,
    /// Default region (e.g., "us-east-1").
    pub region: String,
}

impl AwsAccount {
    /// Create a new AWS account reference.
    pub fn new(id: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            region: region.into(),
        }
    }
}

impl std::fmt::Display for AwsAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.region)
    }
}

/// AWS resource types that can be upserted.
///
/// Each variant carries the configuration needed to check, create,
/// and resolve the resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AwsResource {
    /// S3 bucket.
    S3Bucket {
        /// Bucket name (globally unique).
        name: String,
        /// Override region (uses account default if None).
        region: Option<String>,
    },

    /// IAM Role.
    IamRole {
        /// Role name.
        name: String,
        /// Trust policy document (JSON string).
        /// Required for creation; describes who can assume this role.
        assume_role_policy: String,
    },

    /// Secrets Manager secret.
    SecretsManagerSecret {
        /// Secret name.
        name: String,
    },

    /// ECR (Elastic Container Registry) repository.
    EcrRepository {
        /// Repository name.
        name: String,
    },
}

impl AwsResource {
    /// Get the resource kind identifier.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::S3Bucket { .. } => "s3_bucket",
            Self::IamRole { .. } => "iam_role",
            Self::SecretsManagerSecret { .. } => "secretsmanager_secret",
            Self::EcrRepository { .. } => "ecr_repository",
        }
    }

    /// Get the resource name.
    pub fn name(&self) -> &str {
        match self {
            Self::S3Bucket { name, .. } => name,
            Self::IamRole { name, .. } => name,
            Self::SecretsManagerSecret { name } => name,
            Self::EcrRepository { name } => name,
        }
    }

    /// Resolve the effective region for this resource.
    fn effective_region<'a>(&'a self, account: &'a AwsAccount) -> &'a str {
        match self {
            Self::S3Bucket { region: Some(r), .. } => r.as_str(),
            _ => account.region.as_str(),
        }
    }

    /// Generate the AWS CLI check command for this resource.
    ///
    /// Returns `(command, args)` that can be used in a `ShellRequest`.
    pub fn check_command(&self, account: &AwsAccount) -> (&'static str, Vec<String>) {
        match self {
            Self::S3Bucket { name, .. } => (
                "aws",
                vec![
                    "s3api".into(),
                    "head-bucket".into(),
                    "--bucket".into(),
                    name.clone(),
                    "--region".into(),
                    self.effective_region(account).to_string(),
                ],
            ),
            Self::IamRole { name, .. } => (
                "aws",
                vec![
                    "iam".into(),
                    "get-role".into(),
                    "--role-name".into(),
                    name.clone(),
                ],
            ),
            Self::SecretsManagerSecret { name } => (
                "aws",
                vec![
                    "secretsmanager".into(),
                    "describe-secret".into(),
                    "--secret-id".into(),
                    name.clone(),
                    "--region".into(),
                    self.effective_region(account).to_string(),
                ],
            ),
            Self::EcrRepository { name } => (
                "aws",
                vec![
                    "ecr".into(),
                    "describe-repositories".into(),
                    "--repository-names".into(),
                    name.clone(),
                    "--region".into(),
                    self.effective_region(account).to_string(),
                ],
            ),
        }
    }

    /// Generate the AWS CLI create command for this resource.
    ///
    /// Returns `(command, args)` that can be used in a `ShellRequest`.
    pub fn create_command(&self, account: &AwsAccount) -> (&'static str, Vec<String>) {
        match self {
            Self::S3Bucket { name, .. } => {
                let region = self.effective_region(account);
                let mut args = vec![
                    "s3api".into(),
                    "create-bucket".into(),
                    "--bucket".into(),
                    name.clone(),
                    "--region".into(),
                    region.to_string(),
                ];
                // LocationConstraint is required for non-us-east-1 regions
                if region != "us-east-1" {
                    args.push("--create-bucket-configuration".into());
                    args.push(format!("LocationConstraint={region}"));
                }
                ("aws", args)
            }
            Self::IamRole {
                name,
                assume_role_policy,
            } => (
                "aws",
                vec![
                    "iam".into(),
                    "create-role".into(),
                    "--role-name".into(),
                    name.clone(),
                    "--assume-role-policy-document".into(),
                    assume_role_policy.clone(),
                ],
            ),
            Self::SecretsManagerSecret { name } => (
                "aws",
                vec![
                    "secretsmanager".into(),
                    "create-secret".into(),
                    "--name".into(),
                    name.clone(),
                    "--region".into(),
                    self.effective_region(account).to_string(),
                ],
            ),
            Self::EcrRepository { name } => (
                "aws",
                vec![
                    "ecr".into(),
                    "create-repository".into(),
                    "--repository-name".into(),
                    name.clone(),
                    "--region".into(),
                    self.effective_region(account).to_string(),
                ],
            ),
        }
    }

    /// Get the IAM role ARN for an IamRole resource.
    ///
    /// Returns `None` for non-IamRole resources.
    pub fn role_arn(&self, account: &AwsAccount) -> Option<String> {
        match self {
            Self::IamRole { name, .. } => {
                Some(format!("arn:aws:iam::{}:role/{}", account.id, name))
            }
            _ => None,
        }
    }
}

/// AWS OIDC federation configuration for GitHub Actions.
///
/// Enables keyless authentication from GitHub Actions to AWS using OIDC.
/// The GitHub Actions runner requests an OIDC token, which AWS validates
/// against the IAM OIDC provider to grant temporary credentials via STS.
///
/// # Setup Requirements
///
/// 1. An IAM OIDC Provider for `token.actions.githubusercontent.com`
/// 2. An IAM Role with a trust policy allowing the OIDC provider
/// 3. The role must have permissions for the resources you want to access
///
/// # Example
///
/// ```ignore
/// let oidc = AwsOidcConfig::new(
///     AwsAccount::new("123456789012", "us-east-1"),
///     "github-actions-role",
///     "myorg/myrepo",
/// );
///
/// let role_arn = oidc.role_arn();
/// // "arn:aws:iam::123456789012:role/github-actions-role"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AwsOidcConfig {
    /// AWS account for the OIDC role.
    pub account: AwsAccount,
    /// IAM role name to assume via OIDC.
    pub role_name: String,
    /// GitHub repository filter (e.g., "myorg/myrepo").
    /// Used in the trust policy to restrict which repos can assume the role.
    pub github_repo: String,
}

impl AwsOidcConfig {
    /// Create a new AWS OIDC configuration.
    pub fn new(
        account: AwsAccount,
        role_name: impl Into<String>,
        github_repo: impl Into<String>,
    ) -> Self {
        Self {
            account,
            role_name: role_name.into(),
            github_repo: github_repo.into(),
        }
    }

    /// Get the IAM role ARN for the OIDC role.
    pub fn role_arn(&self) -> String {
        format!(
            "arn:aws:iam::{}:role/{}",
            self.account.id, self.role_name,
        )
    }

    /// Generate the trust policy document for this OIDC configuration.
    ///
    /// This policy allows GitHub Actions from the specified repository
    /// to assume this role via OIDC federation.
    pub fn trust_policy(&self) -> String {
        serde_json::json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {
                    "Federated": format!(
                        "arn:aws:iam::{}:oidc-provider/token.actions.githubusercontent.com",
                        self.account.id,
                    )
                },
                "Action": "sts:AssumeRoleWithWebIdentity",
                "Condition": {
                    "StringEquals": {
                        "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
                    },
                    "StringLike": {
                        "token.actions.githubusercontent.com:sub": format!(
                            "repo:{}:*",
                            self.github_repo,
                        )
                    }
                }
            }]
        })
        .to_string()
    }

    /// List the AWS resources that must exist for OIDC to work.
    ///
    /// These should be upserted before any workflow that uses OIDC auth.
    pub fn required_resources(&self) -> Vec<AwsResource> {
        vec![AwsResource::IamRole {
            name: self.role_name.clone(),
            assume_role_policy: self.trust_policy(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_account() -> AwsAccount {
        AwsAccount::new("123456789012", "us-east-1")
    }

    #[test]
    fn test_aws_account_display() {
        let account = test_account();
        assert_eq!(account.to_string(), "123456789012:us-east-1");
    }

    #[test]
    fn test_aws_resource_kind() {
        let bucket = AwsResource::S3Bucket {
            name: "b".into(),
            region: None,
        };
        assert_eq!(bucket.kind(), "s3_bucket");

        let role = AwsResource::IamRole {
            name: "r".into(),
            assume_role_policy: "{}".into(),
        };
        assert_eq!(role.kind(), "iam_role");

        let secret = AwsResource::SecretsManagerSecret {
            name: "s".into(),
        };
        assert_eq!(secret.kind(), "secretsmanager_secret");

        let ecr = AwsResource::EcrRepository {
            name: "e".into(),
        };
        assert_eq!(ecr.kind(), "ecr_repository");
    }

    #[test]
    fn test_s3_bucket_check_command() {
        let account = test_account();
        let bucket = AwsResource::S3Bucket {
            name: "my-bucket".into(),
            region: None,
        };
        let (cmd, args) = bucket.check_command(&account);
        assert_eq!(cmd, "aws");
        assert!(args.contains(&"head-bucket".to_string()));
        assert!(args.contains(&"my-bucket".to_string()));
    }

    #[test]
    fn test_s3_bucket_create_with_location_constraint() {
        let account = AwsAccount::new("123456789012", "eu-west-1");
        let bucket = AwsResource::S3Bucket {
            name: "my-bucket".into(),
            region: None,
        };
        let (_, args) = bucket.create_command(&account);
        assert!(args.contains(&"--create-bucket-configuration".to_string()));
        assert!(args.contains(&"LocationConstraint=eu-west-1".to_string()));
    }

    #[test]
    fn test_s3_bucket_create_us_east_1_no_constraint() {
        let account = test_account(); // us-east-1
        let bucket = AwsResource::S3Bucket {
            name: "my-bucket".into(),
            region: None,
        };
        let (_, args) = bucket.create_command(&account);
        assert!(!args.contains(&"--create-bucket-configuration".to_string()));
    }

    #[test]
    fn test_s3_bucket_region_override() {
        let account = test_account();
        let bucket = AwsResource::S3Bucket {
            name: "my-bucket".into(),
            region: Some("ap-southeast-1".into()),
        };
        let (_, args) = bucket.check_command(&account);
        assert!(args.contains(&"ap-southeast-1".to_string()));
    }

    #[test]
    fn test_iam_role_arn() {
        let account = test_account();
        let role = AwsResource::IamRole {
            name: "my-role".into(),
            assume_role_policy: "{}".into(),
        };
        assert_eq!(
            role.role_arn(&account),
            Some("arn:aws:iam::123456789012:role/my-role".to_string()),
        );
    }

    #[test]
    fn test_non_role_has_no_arn() {
        let account = test_account();
        let bucket = AwsResource::S3Bucket {
            name: "b".into(),
            region: None,
        };
        assert_eq!(bucket.role_arn(&account), None);
    }

    #[test]
    fn test_aws_oidc_config() {
        let oidc = AwsOidcConfig::new(
            test_account(),
            "github-actions-role",
            "myorg/myrepo",
        );

        assert_eq!(
            oidc.role_arn(),
            "arn:aws:iam::123456789012:role/github-actions-role",
        );
    }

    #[test]
    fn test_aws_oidc_trust_policy() {
        let oidc = AwsOidcConfig::new(
            test_account(),
            "github-actions-role",
            "myorg/myrepo",
        );

        let policy = oidc.trust_policy();
        let parsed: serde_json::Value = serde_json::from_str(&policy).unwrap();

        assert_eq!(parsed["Version"], "2012-10-17");
        let statement = &parsed["Statement"][0];
        assert_eq!(statement["Effect"], "Allow");
        assert_eq!(statement["Action"], "sts:AssumeRoleWithWebIdentity");
    }

    #[test]
    fn test_aws_oidc_required_resources() {
        let oidc = AwsOidcConfig::new(
            test_account(),
            "github-actions-role",
            "myorg/myrepo",
        );
        let resources = oidc.required_resources();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].kind(), "iam_role");
        assert_eq!(resources[0].name(), "github-actions-role");
    }

    #[test]
    fn test_cloud_resource_from_aws() {
        use super::super::CloudResource;

        let account = test_account();
        let bucket = AwsResource::S3Bucket {
            name: "my-bucket".into(),
            region: None,
        };

        let cloud_resource = CloudResource::aws(&account, &bucket);
        assert_eq!(
            cloud_resource.resource_id(),
            "cloud:aws:123456789012:s3_bucket/my-bucket",
        );
    }
}
