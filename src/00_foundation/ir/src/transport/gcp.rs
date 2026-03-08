//! GCP resource types for typed service interfaces.
//!
//! These types model GCP infrastructure resources as first-class Rust structs.
//! They serve as the "nouns" that GCP service traits operate on.
//!
//! Design follows:
//! - gunb.ai's `spec.go` (ServiceAccountSpec, WIFBinding, etc.)
//! - the-gunbai's behavior-based understanding model
//!
//! All types are serde-serializable for TOML/JSON config generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

/// GCP project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpProject {
    /// Project ID (globally unique, e.g., "gunbai-auto").
    pub project_id: String,
    /// Project number (numeric, assigned by GCP).
    pub project_number: Option<i64>,
    /// Human-readable display name.
    pub display_name: Option<String>,
    /// Project lifecycle state (ACTIVE, DELETE_REQUESTED, etc.).
    pub state: Option<String>,
    /// Labels on the project.
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Identity: Service Accounts
// ---------------------------------------------------------------------------

/// GCP service account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpServiceAccount {
    /// Full email address (e.g., "sa@project.iam.gserviceaccount.com").
    pub email: String,
    /// Project ID the SA belongs to.
    pub project_id: String,
    /// Human-readable display name.
    pub display_name: Option<String>,
    /// Description of the SA's purpose.
    pub description: Option<String>,
    /// Unique ID (numeric).
    pub unique_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Workload Identity Federation
// ---------------------------------------------------------------------------

/// Workload Identity Federation pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpWifPool {
    /// Full resource name (e.g., "projects/{number}/locations/global/workloadIdentityPools/{id}").
    pub name: String,
    /// Pool ID (short name).
    pub pool_id: String,
    /// Project number that owns this pool.
    pub project_number: i64,
    /// Pool state (ACTIVE, DELETED, etc.).
    pub state: String,
    /// Human-readable display name.
    pub display_name: Option<String>,
    /// Description.
    pub description: Option<String>,
}

/// Workload Identity Federation provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpWifProvider {
    /// Full resource name.
    pub name: String,
    /// Provider ID (short name).
    pub provider_id: String,
    /// Display name.
    pub display_name: Option<String>,
    /// Attribute mappings (e.g., "google.subject" -> "assertion.sub").
    #[serde(default)]
    pub attribute_mappings: HashMap<String, String>,
    /// Attribute condition (CEL expression for access control).
    pub attribute_condition: Option<String>,
    /// OIDC issuer URI (for OIDC providers).
    pub oidc_issuer_uri: Option<String>,
}

impl GcpWifProvider {
    /// Construct the full WIF provider resource path used as audience in STS exchanges.
    ///
    /// Format: `projects/{number}/locations/global/workloadIdentityPools/{pool}/providers/{provider}`
    pub fn audience_path(&self) -> &str {
        &self.name
    }
}

// ---------------------------------------------------------------------------
// Secret Manager
// ---------------------------------------------------------------------------

/// Secret Manager secret (metadata, not payload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpSecret {
    /// Full resource name (e.g., "projects/{project}/secrets/{secret_id}").
    pub name: String,
    /// Secret ID (short name, e.g., "dev-github-token").
    pub secret_id: String,
    /// Project ID that owns this secret.
    pub project_id: String,
    /// Labels on the secret.
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Replication policy (AUTOMATIC or user-managed).
    pub replication: Option<String>,
}

/// Secret Manager secret version payload (decoded).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpSecretPayload {
    /// The decoded secret data.
    pub data: String,
    /// Version number.
    pub version: String,
}

// ---------------------------------------------------------------------------
// Cloud Storage
// ---------------------------------------------------------------------------

/// Cloud Storage bucket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpBucket {
    /// Bucket name (globally unique).
    pub name: String,
    /// Location (e.g., "US", "us-central1").
    pub location: String,
    /// Storage class (STANDARD, NEARLINE, COLDLINE, ARCHIVE).
    pub storage_class: Option<String>,
    /// Labels on the bucket.
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Whether versioning is enabled.
    pub versioning_enabled: Option<bool>,
    /// Whether uniform bucket-level access is enabled.
    pub uniform_bucket_level_access: Option<bool>,
}

// ---------------------------------------------------------------------------
// IAM
// ---------------------------------------------------------------------------

/// IAM binding (member + role on a resource).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpIamBinding {
    /// Role (e.g., "roles/secretmanager.secretAccessor").
    pub role: String,
    /// Member (e.g., "serviceAccount:sa@project.iam.gserviceaccount.com").
    pub member: String,
    /// Optional IAM condition.
    pub condition: Option<GcpIamCondition>,
}

/// IAM condition for conditional bindings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpIamCondition {
    /// Condition title.
    pub title: String,
    /// CEL expression.
    pub expression: String,
    /// Optional description.
    pub description: Option<String>,
}

/// IAM policy (complete set of bindings on a resource).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcpIamPolicy {
    /// Resource this policy applies to.
    pub resource: String,
    /// All bindings in this policy.
    pub bindings: Vec<GcpIamBinding>,
    /// ETag for optimistic concurrency control.
    pub etag: Option<String>,
}

// ---------------------------------------------------------------------------
// Full Infrastructure Spec (output of discovery DAG)
// ---------------------------------------------------------------------------

/// Complete discovered infrastructure state for a GCP project.
///
/// This is the output of the infra discovery DAG -- a typed snapshot
/// of all relevant cloud resources. It serves as input to config generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GcpInfraSpec {
    /// Discovered projects.
    pub projects: Vec<GcpProject>,
    /// Discovered WIF pools.
    pub wif_pools: Vec<GcpWifPool>,
    /// Discovered WIF providers.
    pub wif_providers: Vec<GcpWifProvider>,
    /// Discovered service accounts.
    pub service_accounts: Vec<GcpServiceAccount>,
    /// Discovered secrets (metadata only).
    pub secrets: Vec<GcpSecret>,
    /// Discovered storage buckets.
    pub buckets: Vec<GcpBucket>,
    /// Discovered IAM policies.
    pub iam_policies: Vec<GcpIamPolicy>,
    /// ISO 8601 timestamp when this spec was discovered.
    pub discovered_at: String,
    /// Project ID used as the discovery root.
    pub source_project: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcp_project_serde_roundtrip() {
        let project = GcpProject {
            project_id: "gunbai-secrets".to_string(),
            project_number: Some(582015116396),
            display_name: Some("gunbai-secrets".to_string()),
            state: Some("ACTIVE".to_string()),
            labels: HashMap::from([("env".to_string(), "shared".to_string())]),
        };
        let json = serde_json::to_string(&project).unwrap();
        let parsed: GcpProject = serde_json::from_str(&json).unwrap();
        assert_eq!(project, parsed);
    }

    #[test]
    fn test_gcp_service_account_serde_roundtrip() {
        let sa = GcpServiceAccount {
            email: "gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string(),
            project_id: "gunbai-secrets".to_string(),
            display_name: Some("Dev secrets accessor".to_string()),
            description: None,
            unique_id: Some("12345".to_string()),
        };
        let json = serde_json::to_string(&sa).unwrap();
        let parsed: GcpServiceAccount = serde_json::from_str(&json).unwrap();
        assert_eq!(sa, parsed);
    }

    #[test]
    fn test_gcp_wif_provider_audience_path() {
        let provider = GcpWifProvider {
            name: "projects/314501921854/locations/global/workloadIdentityPools/github-pool/providers/github".to_string(),
            provider_id: "github".to_string(),
            display_name: None,
            attribute_mappings: HashMap::from([
                ("google.subject".to_string(), "assertion.sub".to_string()),
            ]),
            attribute_condition: Some("assertion.repository_owner == 'gunb-ai'".to_string()),
            oidc_issuer_uri: Some("https://token.actions.githubusercontent.com".to_string()),
        };
        assert!(provider.audience_path().contains("github-pool"));
        assert!(provider.audience_path().contains("providers/github"));
    }

    #[test]
    fn test_gcp_secret_serde_roundtrip() {
        let secret = GcpSecret {
            name: "projects/gunbai-secrets/secrets/dev-github-token".to_string(),
            secret_id: "dev-github-token".to_string(),
            project_id: "gunbai-secrets".to_string(),
            labels: HashMap::new(),
            replication: Some("AUTOMATIC".to_string()),
        };
        let json = serde_json::to_string(&secret).unwrap();
        let parsed: GcpSecret = serde_json::from_str(&json).unwrap();
        assert_eq!(secret, parsed);
    }

    #[test]
    fn test_gcp_iam_binding_with_condition() {
        let binding = GcpIamBinding {
            role: "roles/secretmanager.secretAccessor".to_string(),
            member: "serviceAccount:sa@project.iam.gserviceaccount.com".to_string(),
            condition: Some(GcpIamCondition {
                title: "Only dev secrets".to_string(),
                expression: "resource.name.startsWith('projects/p/secrets/dev-')".to_string(),
                description: None,
            }),
        };
        let json = serde_json::to_string(&binding).unwrap();
        let parsed: GcpIamBinding = serde_json::from_str(&json).unwrap();
        assert_eq!(binding, parsed);
    }

    #[test]
    fn test_gcp_infra_spec_serde_roundtrip() {
        let spec = GcpInfraSpec {
            projects: vec![GcpProject {
                project_id: "test".to_string(),
                project_number: None,
                display_name: None,
                state: None,
                labels: HashMap::new(),
            }],
            wif_pools: vec![],
            wif_providers: vec![],
            service_accounts: vec![],
            secrets: vec![],
            buckets: vec![],
            iam_policies: vec![],
            discovered_at: "2026-02-08T00:00:00Z".to_string(),
            source_project: "test".to_string(),
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: GcpInfraSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, parsed);
    }
}
