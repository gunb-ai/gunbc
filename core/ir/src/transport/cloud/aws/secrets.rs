//! AWS Secrets Manager and SSM Parameter Store resource definitions.

use super::{Arn, AwsRegion, AwsResourceType};
use crate::transport::cloud::{CloudProvider, ResourceHandle, ResourceState};
use crate::transport::ShellRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Secrets Manager Secret
// ============================================================================

/// Secrets Manager secret definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwsSecretDef {
    /// AWS account ID
    pub account_id: String,
    /// AWS region
    pub region: AwsRegion,
    /// Secret name
    pub secret_name: String,
    /// Secret description
    pub description: Option<String>,
    /// KMS key ID for encryption (optional, uses default if not specified)
    pub kms_key_id: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl AwsSecretDef {
    /// Create a new secret definition.
    pub fn new(
        account_id: impl Into<String>,
        region: AwsRegion,
        secret_name: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            region,
            secret_name: secret_name.into(),
            description: None,
            kms_key_id: None,
            tags: HashMap::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the KMS key ID.
    pub fn with_kms_key(mut self, key_id: impl Into<String>) -> Self {
        self.kms_key_id = Some(key_id.into());
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Get the secret ARN.
    pub fn arn(&self) -> String {
        Arn::secret(self.region.as_str(), &self.account_id, &self.secret_name)
    }

    /// Generate the AWS CLI command to check if this secret exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "secretsmanager",
                "describe-secret",
                "--secret-id",
                &self.secret_name,
                "--region",
                self.region.as_str(),
                "--output",
                "json",
            ])
    }

    /// Generate the AWS CLI command to create this secret.
    ///
    /// Note: The secret value should be provided via --secret-string or --secret-binary.
    pub fn create_command(&self) -> ShellRequest {
        let mut args = vec![
            "secretsmanager".to_string(),
            "create-secret".to_string(),
            "--name".to_string(),
            self.secret_name.clone(),
            "--region".to_string(),
            self.region.as_str().to_string(),
        ];

        if let Some(ref desc) = self.description {
            args.push("--description".to_string());
            args.push(desc.clone());
        }

        if let Some(ref kms_key) = self.kms_key_id {
            args.push("--kms-key-id".to_string());
            args.push(kms_key.clone());
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

    /// Generate the AWS CLI command to get the secret value.
    pub fn get_value_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "secretsmanager",
                "get-secret-value",
                "--secret-id",
                &self.secret_name,
                "--region",
                self.region.as_str(),
                "--output",
                "json",
            ])
    }

    /// Generate the AWS CLI command to put a new secret value.
    ///
    /// The secret string should be provided via stdin.
    pub fn put_value_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "secretsmanager",
                "put-secret-value",
                "--secret-id",
                &self.secret_name,
                "--secret-string",
                "file:///dev/stdin",
                "--region",
                self.region.as_str(),
            ])
    }

    /// Generate the AWS CLI command to delete this secret.
    pub fn delete_command(&self, force: bool) -> ShellRequest {
        let mut args = vec![
            "secretsmanager".to_string(),
            "delete-secret".to_string(),
            "--secret-id".to_string(),
            self.secret_name.clone(),
            "--region".to_string(),
            self.region.as_str().to_string(),
        ];

        if force {
            args.push("--force-delete-without-recovery".to_string());
        }

        ShellRequest::new("aws").args(args)
    }

    /// Create a resource handle for this secret.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Aws,
            AwsResourceType::Secret.as_str(),
            self.arn(),
            state,
        )
        .with_metadata("secret_name", serde_json::json!(&self.secret_name))
        .with_metadata("region", serde_json::json!(self.region.as_str()))
    }
}

// ============================================================================
// SSM Parameter Store
// ============================================================================

/// SSM Parameter Store parameter definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDef {
    /// AWS account ID
    pub account_id: String,
    /// AWS region
    pub region: AwsRegion,
    /// Parameter name (path)
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Description
    pub description: Option<String>,
    /// KMS key ID (for SecureString)
    pub kms_key_id: Option<String>,
    /// Tags
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

/// SSM Parameter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    /// Plain text string
    String,
    /// Comma-separated list
    StringList,
    /// Encrypted string (using KMS)
    SecureString,
}

impl ParameterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::StringList => "StringList",
            Self::SecureString => "SecureString",
        }
    }
}

impl ParameterDef {
    /// Create a new String parameter.
    pub fn string(
        account_id: impl Into<String>,
        region: AwsRegion,
        name: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            region,
            name: name.into(),
            param_type: ParameterType::String,
            description: None,
            kms_key_id: None,
            tags: HashMap::new(),
        }
    }

    /// Create a new SecureString parameter.
    pub fn secure_string(
        account_id: impl Into<String>,
        region: AwsRegion,
        name: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            region,
            name: name.into(),
            param_type: ParameterType::SecureString,
            description: None,
            kms_key_id: None,
            tags: HashMap::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the KMS key ID.
    pub fn with_kms_key(mut self, key_id: impl Into<String>) -> Self {
        self.kms_key_id = Some(key_id.into());
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Get the parameter ARN.
    pub fn arn(&self) -> String {
        Arn::parameter(self.region.as_str(), &self.account_id, &self.name)
    }

    /// Generate the AWS CLI command to check if this parameter exists.
    pub fn check_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "ssm",
                "get-parameter",
                "--name",
                &self.name,
                "--region",
                self.region.as_str(),
                "--output",
                "json",
            ])
    }

    /// Generate the AWS CLI command to put this parameter.
    ///
    /// Note: The value should be provided via --value.
    pub fn put_command(&self, value: &str, overwrite: bool) -> ShellRequest {
        let mut args = vec![
            "ssm".to_string(),
            "put-parameter".to_string(),
            "--name".to_string(),
            self.name.clone(),
            "--type".to_string(),
            self.param_type.as_str().to_string(),
            "--value".to_string(),
            value.to_string(),
            "--region".to_string(),
            self.region.as_str().to_string(),
        ];

        if overwrite {
            args.push("--overwrite".to_string());
        }

        if let Some(ref desc) = self.description {
            args.push("--description".to_string());
            args.push(desc.clone());
        }

        if let Some(ref kms_key) = self.kms_key_id {
            args.push("--key-id".to_string());
            args.push(kms_key.clone());
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

    /// Generate the AWS CLI command to get this parameter.
    pub fn get_command(&self, with_decryption: bool) -> ShellRequest {
        let mut args = vec![
            "ssm".to_string(),
            "get-parameter".to_string(),
            "--name".to_string(),
            self.name.clone(),
            "--region".to_string(),
            self.region.as_str().to_string(),
        ];

        if with_decryption {
            args.push("--with-decryption".to_string());
        }

        args.push("--output".to_string());
        args.push("json".to_string());

        ShellRequest::new("aws").args(args)
    }

    /// Generate the AWS CLI command to delete this parameter.
    pub fn delete_command(&self) -> ShellRequest {
        ShellRequest::new("aws")
            .args([
                "ssm",
                "delete-parameter",
                "--name",
                &self.name,
                "--region",
                self.region.as_str(),
            ])
    }

    /// Create a resource handle for this parameter.
    pub fn to_handle(&self, state: ResourceState) -> ResourceHandle {
        ResourceHandle::new(
            CloudProvider::Aws,
            AwsResourceType::Parameter.as_str(),
            self.arn(),
            state,
        )
        .with_metadata("name", serde_json::json!(&self.name))
        .with_metadata("type", serde_json::json!(self.param_type.as_str()))
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
        let secret = AwsSecretDef::new("123456789012", AwsRegion::UsEast1, "my-secret")
            .with_description("Test secret");

        assert_eq!(secret.secret_name, "my-secret");
        assert!(secret.description.is_some());
    }

    #[test]
    fn test_secret_arn() {
        let secret = AwsSecretDef::new("123456789012", AwsRegion::UsEast1, "my-secret");
        let arn = secret.arn();

        assert!(arn.contains("secretsmanager"));
        assert!(arn.contains("us-east-1"));
        assert!(arn.contains("my-secret"));
    }

    #[test]
    fn test_secret_check_command() {
        let secret = AwsSecretDef::new("123456789012", AwsRegion::UsEast1, "my-secret");
        let cmd = secret.check_command();

        assert_eq!(cmd.command, "aws");
        assert!(cmd.args.contains(&"describe-secret".to_string()));
        assert!(cmd.args.contains(&"my-secret".to_string()));
    }

    #[test]
    fn test_secret_create_command() {
        let secret = AwsSecretDef::new("123456789012", AwsRegion::UsEast1, "my-secret")
            .with_description("Test")
            .with_tag("Environment", "test");
        let cmd = secret.create_command();

        assert_eq!(cmd.command, "aws");
        assert!(cmd.args.contains(&"create-secret".to_string()));
        assert!(cmd.args.contains(&"--description".to_string()));
        assert!(cmd.args.contains(&"--tags".to_string()));
    }

    #[test]
    fn test_parameter_string() {
        let param = ParameterDef::string("123456789012", AwsRegion::UsEast1, "/app/config/key");
        assert_eq!(param.param_type, ParameterType::String);
    }

    #[test]
    fn test_parameter_secure_string() {
        let param = ParameterDef::secure_string("123456789012", AwsRegion::UsEast1, "/app/secret");
        assert_eq!(param.param_type, ParameterType::SecureString);
    }

    #[test]
    fn test_parameter_arn() {
        let param = ParameterDef::string("123456789012", AwsRegion::UsEast1, "/app/config");
        let arn = param.arn();

        assert!(arn.contains("ssm"));
        assert!(arn.contains("parameter"));
        assert!(arn.contains("app/config"));
    }

    #[test]
    fn test_parameter_put_command() {
        let param = ParameterDef::string("123456789012", AwsRegion::UsEast1, "/app/config")
            .with_description("Config value");
        let cmd = param.put_command("test-value", true);

        assert_eq!(cmd.command, "aws");
        assert!(cmd.args.contains(&"put-parameter".to_string()));
        assert!(cmd.args.contains(&"test-value".to_string()));
        assert!(cmd.args.contains(&"--overwrite".to_string()));
    }

    #[test]
    fn test_parameter_get_command_with_decryption() {
        let param = ParameterDef::secure_string("123456789012", AwsRegion::UsEast1, "/app/secret");
        let cmd = param.get_command(true);

        assert!(cmd.args.contains(&"--with-decryption".to_string()));
    }

    #[test]
    fn test_secret_handle() {
        let secret = AwsSecretDef::new("123456789012", AwsRegion::UsEast1, "my-secret");
        let handle = secret.to_handle(ResourceState::Active);

        assert_eq!(handle.provider, CloudProvider::Aws);
        assert!(handle.state.is_ready());
    }
}
