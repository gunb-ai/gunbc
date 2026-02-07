//! Cloud provider baseline config for secret management.
//!
//! This is provider-neutral and intended to keep DAGs and tests from being
//! hardwired to a single cloud. Concrete implementations live in
//! provider-specific crates (gcp-ops, aws-ops, azure-ops).

use serde::{Deserialize, Serialize};

use crate::Value;

// ---------------------------------------------------------------------------
// Value conversions (CloudSecretConfig ↔ Value::Json)
// ---------------------------------------------------------------------------

impl From<CloudSecretConfig> for Value {
    fn from(config: CloudSecretConfig) -> Self {
        Value::Json(serde_json::to_value(config).unwrap_or(serde_json::Value::Null))
    }
}

impl TryFrom<&Value> for CloudSecretConfig {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Json(json) => serde_json::from_value(json.clone())
                .map_err(|e| format!("invalid CloudSecretConfig json: {e}")),
            _ => Err("expected CloudSecretConfig as Json".to_string()),
        }
    }
}

/// Supported cloud providers for secret-backed credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudProviderKind {
    Gcp,
    Aws,
    Azure,
}

impl CloudProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CloudProviderKind::Gcp => "gcp",
            CloudProviderKind::Aws => "aws",
            CloudProviderKind::Azure => "azure",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "gcp" => Some(CloudProviderKind::Gcp),
            "aws" => Some(CloudProviderKind::Aws),
            "azure" => Some(CloudProviderKind::Azure),
            _ => None,
        }
    }
}

/// Where the OIDC subject token comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudRuntimeKind {
    /// GitHub Actions OIDC.
    GitHubActions,
    /// Cloud metadata server (GCE/GKE, EC2/EKS, Azure IMDS).
    CloudMetadata,
}

impl CloudRuntimeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CloudRuntimeKind::GitHubActions => "github",
            CloudRuntimeKind::CloudMetadata => "metadata",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "github" | "github-actions" => Some(CloudRuntimeKind::GitHubActions),
            "metadata" | "cloud-metadata" => Some(CloudRuntimeKind::CloudMetadata),
            _ => None,
        }
    }
}

/// Secret name composition (prefix + delimiter + name).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudSecretRef {
    pub prefix: String,
    pub name: String,
    #[serde(default)]
    pub delimiter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl CloudSecretRef {
    pub fn full_name(&self) -> String {
        format!("{}{}{}", self.prefix, self.delimiter, self.name)
    }
}

/// Provider-neutral config for cloud secret-backed credentials.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudSecretConfig {
    pub provider: CloudProviderKind,
    pub runtime: CloudRuntimeKind,
    /// Audience / WIF provider identifier.
    pub audience: String,
    /// Project ID (GCP), Account ID (AWS), or Tenant/Vault scope (Azure).
    pub project_or_account: String,
    /// Secret name + prefixing convention.
    pub secret: CloudSecretRef,
    /// Service account email (GCP) or IAM role ARN (AWS), if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account_or_role: Option<String>,
    /// Optional impersonation target (GCP) or role chaining.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonate_account_or_role: Option<String>,
}

impl CloudSecretConfig {
    pub fn secret_name(&self) -> String {
        self.secret.full_name()
    }
}
