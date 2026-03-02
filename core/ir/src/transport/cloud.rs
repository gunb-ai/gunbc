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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, gunbc_delegate_macros::StringEnum,
)]
pub enum CloudProviderKind {
    Gcp,
    Aws,
    Azure,
}

/// Where the OIDC subject token comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudRuntimeKind {
    /// GitHub Actions OIDC.
    GitHubActions,
    /// Cloud metadata server (GCE/GKE, EC2/EKS, Azure IMDS).
    CloudMetadata,
    /// Local developer workstation (e.g., gcloud CLI auth).
    LocalDev,
}

impl CloudRuntimeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CloudRuntimeKind::GitHubActions => "github",
            CloudRuntimeKind::CloudMetadata => "metadata",
            CloudRuntimeKind::LocalDev => "local",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "github" | "github-actions" => Some(CloudRuntimeKind::GitHubActions),
            "metadata" | "cloud-metadata" => Some(CloudRuntimeKind::CloudMetadata),
            "local" | "local-dev" | "dev" => Some(CloudRuntimeKind::LocalDev),
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

// ---------------------------------------------------------------------------
// Cloud Config Spec (deployment configuration, TOML-serializable)
// ---------------------------------------------------------------------------

/// Deployment configuration spec generated from infrastructure discovery.
///
/// This is the canonical configuration format, stored as TOML and tracked
/// by the ManagedResource pipeline. It replaces hidden env var reads.
///
/// Each namespace represents a deployment context (e.g., "dev", "prod", "test")
/// with namespace inheritance for shared configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudConfigSpec {
    /// Deployment namespaces (e.g., dev, prod, test, local).
    pub namespaces: Vec<CloudNamespace>,
    /// Default namespace to use when none is specified.
    pub default_namespace: Option<String>,
    /// ISO 8601 timestamp when this config was generated.
    pub generated_at: Option<String>,
    /// Source project ID used for discovery.
    pub source_project: Option<String>,
}

/// A single deployment namespace within the config spec.
///
/// Namespace inheritance allows common settings to be shared.
/// For the canonical project structure, see `cloud_ops::project_spec::GUNBAI_SECRETS`
/// which defines typed specs that derive all namespace values (prefixes,
/// SA emails, WIF resource names) from base fields.
///
/// ```toml
/// [[namespaces]]
/// name = "base"
/// secrets_project = "my-secrets-project"
/// wif_provider = "projects/{number}/locations/global/workloadIdentityPools/{pool}/providers/{provider}"
///
/// [[namespaces]]
/// name = "dev"
/// inherits_from = "base"
/// # prefix derived from name: "dev-"
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudNamespace {
    /// Namespace name (e.g., "dev", "prod", "test", "local").
    pub name: String,
    /// Parent namespace to inherit from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits_from: Option<String>,
    /// Cloud provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<CloudProviderKind>,
    /// GCP project for secret storage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets_project: Option<String>,
    /// WIF provider full resource path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wif_provider: Option<String>,
    /// Service account email for impersonation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account: Option<String>,
    /// Impersonation target SA (for two-hop impersonation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonate_account: Option<String>,
}

impl CloudNamespace {
    /// Derive the secret prefix from the namespace name.
    ///
    /// Convention: prefix = "{name}-" (e.g., "dev" -> "dev-").
    pub fn secret_prefix(&self) -> String {
        format!("{}-", self.name)
    }
}

impl CloudConfigSpec {
    /// Find a namespace by name.
    pub fn namespace(&self, name: &str) -> Option<&CloudNamespace> {
        self.namespaces.iter().find(|ns| ns.name == name)
    }

    /// Resolve a namespace, applying inheritance to fill in missing fields.
    pub fn resolve_namespace(&self, name: &str) -> Option<CloudNamespace> {
        let ns = self.namespace(name)?.clone();
        if let Some(ref parent_name) = ns.inherits_from {
            let parent = self.resolve_namespace(parent_name)?;
            Some(CloudNamespace {
                name: ns.name,
                inherits_from: ns.inherits_from,
                provider: ns.provider.or(parent.provider),
                secrets_project: ns.secrets_project.or(parent.secrets_project),
                wif_provider: ns.wif_provider.or(parent.wif_provider),
                service_account: ns.service_account.or(parent.service_account),
                impersonate_account: ns.impersonate_account.or(parent.impersonate_account),
            })
        } else {
            Some(ns)
        }
    }

    /// Convert a resolved namespace to a `CloudSecretConfig` for a given runtime and secret name.
    pub fn to_secret_config(
        &self,
        namespace: &str,
        runtime: CloudRuntimeKind,
        secret_name: &str,
    ) -> Option<CloudSecretConfig> {
        let ns = self.resolve_namespace(namespace)?;
        let provider = ns.provider.unwrap_or(CloudProviderKind::Gcp);
        let audience = ns.wif_provider.clone().unwrap_or_default();
        let prefix = ns.secret_prefix();
        let project = ns.secrets_project.clone()?;

        Some(CloudSecretConfig {
            provider,
            runtime,
            audience,
            project_or_account: project,
            secret: CloudSecretRef {
                prefix,
                name: secret_name.to_string(),
                delimiter: String::new(),
                version: None,
            },
            service_account_or_role: ns.service_account,
            impersonate_account_or_role: ns.impersonate_account,
        })
    }
}
