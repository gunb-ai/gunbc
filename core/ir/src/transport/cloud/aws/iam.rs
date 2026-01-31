//! AWS IAM resource definitions.
//!
//! This module provides types for managing AWS IAM resources:
//! - Roles (with trust policies for OIDC)
//! - Policies
//! - OIDC Providers

use super::{Arn, AwsResourceType};
use crate::transport::cloud::{CloudProvider, ResourceHandle, ResourceState};
use crate::transport::ShellRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// IAM Role
// ============================================================================

/// IAM Role definition.
///
/// Roles are used for workload identity and cross-service authentication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IamRoleDef {
    /// AWS account ID
    pub account_id: String,
    /// Role name
    pub role_name: String,
    /// Role description
    pub description: Option<String>,
    /// Trust policy (who can assume this role)
    pub trust_policy: TrustPolicy,
    /// Managed policy ARNs to attach
    #[serde(default)]
    pub managed_policies: Vec<String>,
    /// Inline policies
    #[serde(default)]
    pub inline_policies: HashMap<String, serde_json::Value>,
    /// Path (default: /)
    pub path: String,
    /// Tags
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

/// Trust policy for IAM role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Policy statements
    pub statements: Vec<TrustStatement>,
}

/// Trust policy statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustStatement {
    /// Effect (Allow/Deny)
    pub effect: String,
    /// Principal
    pub principal: TrustPrincipal,
    /// Action (typically sts:AssumeRoleWithWebIdentity or sts:AssumeRole)
    pub action: String,
    /// Conditions
    #[serde(default)]
    pub condition: HashMap<String, HashMap<String, String>>,
}

/// Trust policy principal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrustPrincipal {
    /// AWS account or role
    Aws(String),
    /// Service principal
    Service(String),
    /// Federated (OIDC) identity
    Federated(String),
}

impl IamRoleDef {
    /// Create a new IAM role definition.
    pub fn new(
        account_id: impl Into<String>,
        role_name: impl Into<String>,
        trust_policy: TrustPolicy,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            role_name: role_name.into(),
            description: None,
            trust_policy,
            managed_policies: Vec::new(),
            inline_policies: HashMap::new(),
            path: "/".to_string(),
            tags: HashMap::new(),
        }
    }

    /// Create a role for GitHub Actions OIDC.
    pub fn for_github_actions(
        account_id: impl Into<String>,
        role_name: impl Into<String>,
        repository: impl Into<String>,
    ) -> Self {
        let account_id = account_id.into();
        let repo = repository.into();

        let oidc_arn = Arn::github_oidc_provider(&account_id);

        let condition = {
            let mut cond = HashMap::new();
            let mut string_like = HashMap::new();
            string_like.insert(
                "token.actions.githubusercontent.com:sub".to_string(),
                format!("repo:{}:*", repo),
            );
            cond.insert("StringLike".to_string(), string_like);
            cond
        };

        let trust_policy = TrustPolicy {
            statements: vec![TrustStatement {
                effect: "Allow".to_string(),
                principal: TrustPrincipal::Federated(oidc_arn),
                action: "sts:AssumeRoleWithWebIdentity".to_string(),
                condition,
            }],
        };

        Self::new(account_id, role_name, trust_policy)
            .with_description(format!("Role for GitHub Actions from {}", repo))
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Attach a managed policy.
    pub fn with_managed_policy(mut self, policy_arn: impl Into<String>) -> Self {
        self.managed_policies.push(policy_arn.into());
        self
    }

    /// Add an inline policy.
    pub fn with_inline_policy(mut self, name: impl Into<String>, policy: serde_json::Value) -> Self {
        self.inline_policies.insert(name.into(), policy);
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Get the role ARN.
    pub fn arn(&self) -> String {
        Arn::iam_role(&self.account_id, &self.role_name)
    }

    /// Generate the trust policy as JSON.
    pub fn trust_policy_json(&self) -> serde_json::Value {
        let statements: Vec<serde_json::Value> = self
            .trust_policy
            .statements
            .iter()
            .map(|s| {
                let principal = match &s.principal {
                    TrustPrincipal::Aws(arn) => serde_json::json!({ "AWS": arn }),
                    TrustPrincipal::Service(svc) => serde_json::json!({ "Service": svc }),
                    TrustPrincipal::Federated(arn) => serde_json::json!({ "Federated": arn }),
                };

                let mut stmt = serde_json::json!({
                    "Effect": s.effect,
                    "Principal": principal,
                    "Action": s.action,
                });

                if !s.condition.is_empty() {
                    stmt["Condition"] = serde_json::json!(s.condition);
                }

                stmt
            })
            .collect();

        serde_json::json!({
            "Version": "2012-10-17",
            "Statement": statements
        })
    }

    // ========================================================================
    // CLI Commands
    // ========================================================================

    /// Generate the AWS CLI command to check if this role exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "iam",
                "get-role",
                "--role-name",
                &self.role_name,
                "--output",
                "json",
            ])
    }

    /// Generate the AWS CLI command to create this role.
    pub fn create_command(&self) -> ShellRequest {
        let trust_policy = serde_json::to_string(&self.trust_policy_json()).unwrap();

        let mut args = vec![
            "iam".to_string(),
            "create-role".to_string(),
            "--role-name".to_string(),
            self.role_name.clone(),
            "--assume-role-policy-document".to_string(),
            trust_policy,
        ];

        if let Some(ref desc) = self.description {
            args.push("--description".to_string());
            args.push(desc.clone());
        }

        if self.path != "/" {
            args.push("--path".to_string());
            args.push(self.path.clone());
        }

        if !self.tags.is_empty() {
            args.push("--tags".to_string());
            let tags: Vec<String> = self
                .tags
                .iter()
                .map(|(k, v)| format!("Key={},Value={}", k, v))
                .collect();
            args.push(tags.join(" "));
        }

        args.push("--output".to_string());
        args.push("json".to_string());

        ShellRequest::new("aws").args(args)
    }

    /// Generate the AWS CLI command to attach a managed policy.
    pub fn attach_policy_command(&self, policy_arn: &str) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "iam",
                "attach-role-policy",
                "--role-name",
                &self.role_name,
                "--policy-arn",
                policy_arn,
            ])
    }

    /// Generate the AWS CLI command to delete this role.
    pub fn delete_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "iam",
                "delete-role",
                "--role-name",
                &self.role_name,
            ])
    }

    /// Create a resource handle for this role.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Aws,
            AwsResourceType::IamRole.as_str(),
            self.arn(),
            state,
        )
        .with_metadata("role_name", serde_json::json!(&self.role_name))
        .with_metadata("account_id", serde_json::json!(&self.account_id))
    }
}

// ============================================================================
// IAM Policy
// ============================================================================

/// IAM Policy definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IamPolicyDef {
    /// AWS account ID
    pub account_id: String,
    /// Policy name
    pub policy_name: String,
    /// Policy description
    pub description: Option<String>,
    /// Policy document (JSON)
    pub policy_document: serde_json::Value,
    /// Path (default: /)
    pub path: String,
    /// Tags
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl IamPolicyDef {
    /// Create a new policy definition.
    pub fn new(
        account_id: impl Into<String>,
        policy_name: impl Into<String>,
        policy_document: serde_json::Value,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            policy_name: policy_name.into(),
            description: None,
            policy_document,
            path: "/".to_string(),
            tags: HashMap::new(),
        }
    }

    /// Create a policy that allows reading from Secrets Manager.
    pub fn secrets_reader(
        account_id: impl Into<String>,
        policy_name: impl Into<String>,
        secret_arns: Vec<String>,
    ) -> Self {
        let policy = serde_json::json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": [
                    "secretsmanager:GetSecretValue",
                    "secretsmanager:DescribeSecret"
                ],
                "Resource": secret_arns
            }]
        });

        Self::new(account_id, policy_name, policy)
            .with_description("Read access to specified secrets")
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Get the policy ARN.
    pub fn arn(&self) -> String {
        Arn::iam_policy(&self.account_id, &self.policy_name)
    }

    /// Generate the AWS CLI command to check if this policy exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "iam",
                "get-policy",
                "--policy-arn",
                &self.arn(),
                "--output",
                "json",
            ])
    }

    /// Generate the AWS CLI command to create this policy.
    pub fn create_command(&self) -> ShellRequest {
        let policy_doc = serde_json::to_string(&self.policy_document).unwrap();

        let mut args = vec![
            "iam".to_string(),
            "create-policy".to_string(),
            "--policy-name".to_string(),
            self.policy_name.clone(),
            "--policy-document".to_string(),
            policy_doc,
        ];

        if let Some(ref desc) = self.description {
            args.push("--description".to_string());
            args.push(desc.clone());
        }

        args.push("--output".to_string());
        args.push("json".to_string());

        ShellRequest::new("aws").args(args)
    }

    /// Create a resource handle for this policy.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Aws,
            AwsResourceType::IamPolicy.as_str(),
            self.arn(),
            state,
        )
        .with_metadata("policy_name", serde_json::json!(&self.policy_name))
    }
}

// ============================================================================
// Common Managed Policies
// ============================================================================

/// Common AWS managed policy ARNs.
pub struct ManagedPolicies;

impl ManagedPolicies {
    // General
    pub const ADMINISTRATOR_ACCESS: &'static str = "arn:aws:iam::aws:policy/AdministratorAccess";
    pub const POWER_USER_ACCESS: &'static str = "arn:aws:iam::aws:policy/PowerUserAccess";
    pub const READ_ONLY_ACCESS: &'static str = "arn:aws:iam::aws:policy/ReadOnlyAccess";

    // Secrets Manager
    pub const SECRETS_MANAGER_READ_WRITE: &'static str =
        "arn:aws:iam::aws:policy/SecretsManagerReadWrite";

    // S3
    pub const S3_FULL_ACCESS: &'static str = "arn:aws:iam::aws:policy/AmazonS3FullAccess";
    pub const S3_READ_ONLY_ACCESS: &'static str = "arn:aws:iam::aws:policy/AmazonS3ReadOnlyAccess";

    // EC2
    pub const EC2_FULL_ACCESS: &'static str = "arn:aws:iam::aws:policy/AmazonEC2FullAccess";
    pub const EC2_READ_ONLY_ACCESS: &'static str = "arn:aws:iam::aws:policy/AmazonEC2ReadOnlyAccess";

    // ECR
    pub const ECR_FULL_ACCESS: &'static str =
        "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryFullAccess";
    pub const ECR_READ_ONLY: &'static str =
        "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly";
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iam_role_for_github_actions() {
        let role = IamRoleDef::for_github_actions("123456789012", "github-actions-role", "owner/repo");

        assert_eq!(role.role_name, "github-actions-role");
        assert!(role.description.is_some());

        let trust_json = role.trust_policy_json();
        assert_eq!(trust_json["Version"], "2012-10-17");

        let statements = trust_json["Statement"].as_array().unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0]["Action"], "sts:AssumeRoleWithWebIdentity");
    }

    #[test]
    fn test_iam_role_arn() {
        let role = IamRoleDef::for_github_actions("123456789012", "my-role", "owner/repo");
        assert_eq!(role.arn(), "arn:aws:iam::123456789012:role/my-role");
    }

    #[test]
    fn test_iam_role_check_command() {
        let role = IamRoleDef::for_github_actions("123456789012", "my-role", "owner/repo");
        let cmd = role.check_command();

        assert_eq!(cmd.command, "aws");
        assert!(cmd.args.contains(&"get-role".to_string()));
        assert!(cmd.args.contains(&"my-role".to_string()));
    }

    #[test]
    fn test_iam_role_create_command() {
        let role = IamRoleDef::for_github_actions("123456789012", "my-role", "owner/repo");
        let cmd = role.create_command();

        assert_eq!(cmd.command, "aws");
        assert!(cmd.args.contains(&"create-role".to_string()));
        assert!(cmd.args.contains(&"--assume-role-policy-document".to_string()));
    }

    #[test]
    fn test_iam_role_with_policies() {
        let role = IamRoleDef::for_github_actions("123456789012", "my-role", "owner/repo")
            .with_managed_policy(ManagedPolicies::S3_READ_ONLY_ACCESS)
            .with_tag("Environment", "production");

        assert_eq!(role.managed_policies.len(), 1);
        assert!(role.tags.contains_key("Environment"));
    }

    #[test]
    fn test_iam_policy_secrets_reader() {
        let policy = IamPolicyDef::secrets_reader(
            "123456789012",
            "secrets-reader",
            vec!["arn:aws:secretsmanager:us-east-1:123456789012:secret:my-secret".to_string()],
        );

        assert_eq!(policy.policy_name, "secrets-reader");
        let doc = &policy.policy_document;
        let statements = doc["Statement"].as_array().unwrap();
        assert!(statements[0]["Action"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a == "secretsmanager:GetSecretValue"));
    }

    #[test]
    fn test_trust_policy_json() {
        let role = IamRoleDef::for_github_actions("123456789012", "test-role", "owner/repo");
        let json = role.trust_policy_json();

        // Check structure
        assert!(json.get("Version").is_some());
        assert!(json.get("Statement").is_some());

        // Check principal is Federated
        let statement = &json["Statement"][0];
        assert!(statement["Principal"]["Federated"].is_string());
    }

    #[test]
    fn test_role_handle() {
        let role = IamRoleDef::for_github_actions("123456789012", "my-role", "owner/repo");
        let handle = role.to_handle(ResourceState::Active);

        assert_eq!(handle.provider, CloudProvider::Aws);
        assert!(handle.resource_id.contains("my-role"));
        assert!(handle.state.is_ready());
    }
}
