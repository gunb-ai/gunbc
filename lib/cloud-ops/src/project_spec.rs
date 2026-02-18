//! GCP project spec — the single source of truth for project structure.
//!
//! This module models the GCP infrastructure that gunbc depends on, following
//! the same code-as-config pattern as `gunb.ai/tools/infra/spec/spec.go`.
//! All values are compile-time constants — no env files, no YAML, no magic
//! strings scattered across the codebase.
//!
//! # Design
//!
//! - **`ProjectSpec`** defines the top-level project structure (like `InfraSpec`
//!   in gunb.ai).
//! - **`NamespaceSpec`** defines deployment contexts (dev, ci) with service
//!   account naming and prefix derivation.
//! - **`SecretSpec`** is the canonical secret catalog (like
//!   `tools/secrets/spec/spec.go` in gunb.ai).
//! - **Derived values** — SA emails, prefixes, WIF resource names — are
//!   computed from base fields, not stored as separate strings.
//!
//! # Adding new environments
//!
//! Add a `NamespaceSpec` entry to `GUNBAI_SECRETS.namespaces` and the
//! conformance tests will verify it produces valid `CloudSecretConfig`.

use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::transport::gist::GITHUB_SECRET_ID;

// ---------------------------------------------------------------------------
// GCP project types
// ---------------------------------------------------------------------------

/// A GCP project reference.
///
/// Mirrors `EnvironmentConfig.Project` + `ProjectNumber` from
/// `gunb.ai/tools/infra/spec/spec.go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcpProject {
    /// Project ID (globally unique, e.g., "gunbai-secrets").
    pub project_id: &'static str,
    /// Project number (numeric, e.g., 314501921854).
    pub project_number: u64,
}

/// Workload Identity Federation config for keyless auth.
///
/// Mirrors `WIFBinding` and the `WIFPoolID`/`WIFProviderID` constants from
/// `gunb.ai/tools/infra/spec/spec.go`.
///
/// The WIF pool lives in a separate project (gunbai-auto) from the secrets
/// project (gunbai-secrets), so the pool's project number is stored here
/// rather than derived from the secrets project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifConfig {
    /// Project number where the WIF pool lives (e.g., gunbai-auto = 314501921854).
    pub project_number: u64,
    /// Pool ID (e.g., "github-pool").
    pub pool_id: &'static str,
    /// Provider ID (e.g., "github").
    pub provider_id: &'static str,
    /// OIDC issuer URI for tokens accepted by this provider.
    pub oidc_issuer_uri: &'static str,
    /// Attribute mapping from Google fields to OIDC token assertions.
    pub attribute_mapping: &'static [(&'static str, &'static str)],
    /// Optional CEL condition restricting accepted tokens.
    pub attribute_condition: Option<&'static str>,
}

impl WifConfig {
    /// Full resource path for the WIF provider.
    ///
    /// Format: `projects/{number}/locations/global/workloadIdentityPools/{pool}/providers/{provider}`
    pub fn provider_resource_name(&self) -> String {
        format!(
            "projects/{}/locations/global/workloadIdentityPools/{}/providers/{}",
            self.project_number, self.pool_id, self.provider_id
        )
    }

    /// WIF provider parent resource path.
    ///
    /// Format: `projects/{number}/locations/global/workloadIdentityPools/{pool}`
    pub fn pool_resource_name(&self) -> String {
        format!(
            "projects/{}/locations/global/workloadIdentityPools/{}",
            self.project_number, self.pool_id
        )
    }
}

// ---------------------------------------------------------------------------
// Service account types
// ---------------------------------------------------------------------------

/// A GCP service account spec.
///
/// Mirrors `ServiceAccountSpec` from `gunb.ai/tools/infra/spec/spec.go`.
/// The full email is derived from `name` + project ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAccountSpec {
    /// SA name without the `@project.iam.gserviceaccount.com` suffix.
    pub name: &'static str,
    /// Human-friendly service account display name.
    pub display_name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// IAM roles this service account should hold on the target project.
    pub self_roles: &'static [&'static str],
    /// Members that should be allowed to impersonate this SA through WIF.
    pub wif_bindings: &'static [&'static str],
}

impl ServiceAccountSpec {
    /// Full service account email: `{name}@{project_id}.iam.gserviceaccount.com`.
    pub fn email(&self, project_id: &str) -> String {
        format!("{}@{}.iam.gserviceaccount.com", self.name, project_id)
    }

    /// IAM bindings where this service account is granted its own project roles.
    pub fn self_role_bindings(&self, project_id: &str) -> Vec<ServiceAccountBinding> {
        let member = format!("serviceAccount:{}", self.email(project_id));
        self.self_roles
            .iter()
            .map(|role| ServiceAccountBinding {
                role: (*role).to_string(),
                members: vec![member.clone()],
            })
            .collect()
    }

    /// IAM bindings allowing configured principals to impersonate this SA.
    pub fn wif_impersonation_bindings(&self) -> Vec<ServiceAccountBinding> {
        if self.wif_bindings.is_empty() {
            return Vec::new();
        }
        vec![ServiceAccountBinding {
            role: "roles/iam.workloadIdentityUser".to_string(),
            members: self.wif_bindings.iter().map(|m| (*m).to_string()).collect(),
        }]
    }
}

/// Canonical IAM binding shape used by service account catalog specs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAccountBinding {
    pub role: String,
    pub members: Vec<String>,
}

// ---------------------------------------------------------------------------
// Namespace (deployment context)
// ---------------------------------------------------------------------------

/// A deployment namespace (dev, ci, etc.)
///
/// Mirrors `EnvironmentConfig` from `gunb.ai/tools/infra/spec/spec.go`.
/// Each namespace has a secrets service account and derives its own
/// prefix from its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceSpec {
    /// Namespace name (e.g., "dev", "ci").
    pub name: &'static str,
    /// Primary project ID for environment resources.
    pub project: &'static str,
    /// Primary project number for environment resources.
    pub project_number: &'static str,
    /// Default region for regional resources.
    pub region: &'static str,
    /// Default zone for zonal resources.
    pub zone: &'static str,
    /// Optional DNS domain for environment endpoints.
    pub domain: Option<&'static str>,
    /// Prefix for resource names in this environment.
    pub name_prefix: &'static str,
    /// Project that stores secrets for this environment.
    pub secrets_project: &'static str,
    /// Prefix for secret IDs in this environment.
    pub secrets_prefix: &'static str,
    /// The secrets service account for this namespace.
    pub secrets_service_account: ServiceAccountSpec,
}

impl NamespaceSpec {
    pub fn secret_prefix(&self) -> String {
        self.secrets_prefix.to_string()
    }

    /// Full service account email for secrets access.
    pub fn service_account_email(&self) -> String {
        self.secrets_service_account.email(self.secrets_project)
    }
}

// ---------------------------------------------------------------------------
// Secret catalog
// ---------------------------------------------------------------------------

/// Requirement level for a secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretRequirement {
    Required,
    Optional,
}

/// Status of a secret (active or deleted/deprecated).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretStatus {
    Active,
    Deleted,
}

/// How a secret can be rotated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationHandler {
    /// Manual rotation via provider dashboard/UI.
    Manual,
    /// Interactive GitHub OAuth browser flow.
    GitHubPat,
    /// `gcloud` SA key creation.
    ServiceAccountKey,
    /// No rotation needed (static/permanent value).
    None,
}

/// A single secret's specification.
///
/// Mirrors `Secret` from `gunb.ai/tools/secrets/spec/spec.go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretSpec {
    /// Environment variable name (e.g., "GITHUB_TOKEN").
    pub env_name: &'static str,
    /// Base Secret Manager ID (e.g., "github-token").
    /// Actual ID has namespace prefix: `"{prefix}{secret_id}"`.
    pub secret_id: &'static str,
    /// Whether this secret is required for the system to function.
    pub requirement: SecretRequirement,
    /// Whether this secret is currently active or deprecated/deleted.
    pub status: SecretStatus,
    /// OAuth/API scopes required (empty for non-token secrets).
    pub scopes: &'static [&'static str],
    /// How this secret can be rotated.
    pub rotation: RotationHandler,
    /// Optional max age (in days) before rotation should be recommended.
    pub max_age_days: Option<u32>,
}

impl SecretSpec {
    /// Prefixed secret ID for a given namespace.
    pub fn prefixed_id(&self, namespace: &NamespaceSpec) -> String {
        format!("{}{}", namespace.secret_prefix(), self.secret_id)
    }
}

// ---------------------------------------------------------------------------
// Top-level project spec
// ---------------------------------------------------------------------------

/// Top-level GCP project spec.
///
/// Mirrors `InfraSpec` from `gunb.ai/tools/infra/spec/spec.go`, scoped to
/// the resources gunbc actually depends on: a secrets project with WIF and
/// per-namespace service accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSpec {
    /// The GCP project used for secret storage.
    pub secrets_project: GcpProject,
    /// Workload Identity Federation config (for GitHub Actions keyless auth).
    pub wif: WifConfig,
    /// Deployment namespaces (dev, ci, etc.)
    pub namespaces: &'static [NamespaceSpec],
    /// Canonical secret catalog.
    pub secrets: &'static [SecretSpec],
}

impl ProjectSpec {
    /// Look up a namespace by name.
    pub fn namespace(&self, name: &str) -> Option<&NamespaceSpec> {
        self.namespaces.iter().find(|ns| ns.name == name)
    }

    /// WIF provider resource name (from the WIF pool's project, not the secrets project).
    pub fn wif_provider_resource_name(&self) -> String {
        self.wif.provider_resource_name()
    }

    /// Build a `CloudSecretConfig` for a given namespace and runtime.
    ///
    /// This is the primary conversion from the typed spec to the runtime
    /// config used by graph builders. For local dev, `runtime` comes from
    /// `detect_runtime()` in config_loader.
    pub fn to_cloud_secret_config(
        &self,
        namespace: &str,
        runtime: CloudRuntimeKind,
    ) -> Option<CloudSecretConfig> {
        let ns = self.namespace(namespace)?;
        Some(CloudSecretConfig {
            provider: CloudProviderKind::Gcp,
            runtime,
            audience: self.wif_provider_resource_name(),
            project_or_account: ns.secrets_project.to_string(),
            secret: CloudSecretRef {
                prefix: ns.secret_prefix(),
                name: String::new(),
                delimiter: String::new(),
                version: None,
            },
            service_account_or_role: Some(ns.service_account_email()),
            impersonate_account_or_role: None,
        })
    }

    /// Active secrets only.
    pub fn active_secrets(&self) -> impl Iterator<Item = &SecretSpec> {
        self.secrets
            .iter()
            .filter(|s| s.status == SecretStatus::Active)
    }

    /// Required active secrets only.
    pub fn required_secrets(&self) -> impl Iterator<Item = &SecretSpec> {
        self.active_secrets()
            .filter(|s| s.requirement == SecretRequirement::Required)
    }
}

// ===========================================================================
// Canonical spec: gunbai-secrets
// ===========================================================================

/// The canonical project spec for the gunbai-secrets infrastructure.
///
/// All values are hardcoded from the canonical spec — no env var overrides.
/// Mirrors `DevConfig()` / `CIConfig()` from `gunb.ai/tools/infra/spec/spec.go`
/// and `AllSecrets()` from `gunb.ai/tools/secrets/spec/spec.go`.
pub static GUNBAI_SECRETS: ProjectSpec = ProjectSpec {
    secrets_project: GcpProject {
        project_id: "gunbai-secrets",
        project_number: 582015116396,
    },
    wif: WifConfig {
        project_number: 314501921854, // gunbai-auto (WIF pool host)
        pool_id: "github-pool",
        provider_id: "github",
        oidc_issuer_uri: "https://token.actions.githubusercontent.com",
        attribute_mapping: &[
            ("google.subject", "assertion.sub"),
            ("attribute.repository", "assertion.repository"),
            ("attribute.repository_owner", "assertion.repository_owner"),
        ],
        attribute_condition: Some("assertion.repository_owner == 'gunb-ai'"),
    },
    namespaces: &[
        NamespaceSpec {
            name: "dev",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: Some("dev.gunb.ai"),
            name_prefix: "dev",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "dev-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-dev-secrets",
                display_name: "Gunbai Dev Secrets",
                description: "Dev environment secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "ci",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: None,
            name_prefix: "ci",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "ci-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-ci-secrets",
                display_name: "Gunbai CI Secrets",
                description: "CI environment secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "test",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: Some("test.gunb.ai"),
            name_prefix: "test",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "test-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-test-secrets",
                display_name: "Gunbai Test Secrets",
                description: "Test environment secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "staging",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: Some("staging.gunb.ai"),
            name_prefix: "staging",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "staging-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-staging-secrets",
                display_name: "Gunbai Staging Secrets",
                description: "Staging environment secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "prod",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: Some("gunb.ai"),
            name_prefix: "prod",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-prod-secrets",
                display_name: "Gunbai Prod Secrets",
                description: "Production environment secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "review",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: None,
            name_prefix: "review",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "review-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-review-secrets",
                display_name: "Gunbai Review Secrets",
                description: "Review workflow secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "llm",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: None,
            name_prefix: "llm",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "llm-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-llm-secrets",
                display_name: "Gunbai LLM Secrets",
                description: "LLM workflow secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "ops",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: None,
            name_prefix: "ops",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "ops-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-ops-secrets",
                display_name: "Gunbai Ops Secrets",
                description: "Operational automation secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
        NamespaceSpec {
            name: "sandbox",
            project: "gunbai-secrets",
            project_number: "582015116396",
            region: "us-central1",
            zone: "us-central1-a",
            domain: Some("sandbox.gunb.ai"),
            name_prefix: "sandbox",
            secrets_project: "gunbai-secrets",
            secrets_prefix: "sandbox-",
            secrets_service_account: ServiceAccountSpec {
                name: "gunbai-sandbox-secrets",
                display_name: "Gunbai Sandbox Secrets",
                description: "Sandbox environment secrets access",
                self_roles: &["roles/secretmanager.secretAccessor"],
                wif_bindings: &[
                    "principalSet://iam.googleapis.com/projects/314501921854/locations/global/workloadIdentityPools/github-pool/attribute.repository/gunb-ai/gunbc",
                ],
            },
        },
    ],
    secrets: &KNOWN_SECRETS,
};

/// Canonical secret catalog.
///
/// Only includes secrets that gunbc currently needs. Additional secrets
/// (openai, anthropic, github-app-*) can be added back as needed.
static KNOWN_SECRETS: [SecretSpec; 1] = [
    // GitHub PAT — interactive browser rotation
    SecretSpec {
        env_name: "GITHUB_TOKEN",
        secret_id: GITHUB_SECRET_ID,
        requirement: SecretRequirement::Optional,
        status: SecretStatus::Active,
        scopes: &["repo", "read:org", "gist"],
        rotation: RotationHandler::GitHubPat,
        max_age_days: Some(90),
    },
];

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_namespace_exists() {
        let ns = GUNBAI_SECRETS
            .namespace("dev")
            .expect("dev namespace must exist");
        assert_eq!(ns.name, "dev");
    }

    #[test]
    fn ci_namespace_exists() {
        let ns = GUNBAI_SECRETS
            .namespace("ci")
            .expect("ci namespace must exist");
        assert_eq!(ns.name, "ci");
    }

    #[test]
    fn dev_secret_prefix_is_derived() {
        let ns = GUNBAI_SECRETS.namespace("dev").unwrap();
        assert_eq!(ns.secret_prefix(), "dev-");
    }

    #[test]
    fn ci_secret_prefix_is_derived() {
        let ns = GUNBAI_SECRETS.namespace("ci").unwrap();
        assert_eq!(ns.secret_prefix(), "ci-");
    }

    #[test]
    fn dev_service_account_email_is_derived() {
        let ns = GUNBAI_SECRETS.namespace("dev").unwrap();
        assert_eq!(
            ns.service_account_email(),
            "gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com"
        );
    }

    #[test]
    fn ci_service_account_email_is_derived() {
        let ns = GUNBAI_SECRETS.namespace("ci").unwrap();
        assert_eq!(
            ns.service_account_email(),
            "gunbai-ci-secrets@gunbai-secrets.iam.gserviceaccount.com"
        );
    }

    #[test]
    fn wif_provider_resource_name_is_derived() {
        assert_eq!(
            GUNBAI_SECRETS.wif_provider_resource_name(),
            "projects/314501921854/locations/global/workloadIdentityPools/github-pool/providers/github"
        );
    }

    #[test]
    fn wif_pool_resource_name_is_derived() {
        assert_eq!(
            GUNBAI_SECRETS.wif.pool_resource_name(),
            "projects/314501921854/locations/global/workloadIdentityPools/github-pool"
        );
    }

    #[test]
    fn wif_config_carries_oidc_mapping_and_condition() {
        let wif = &GUNBAI_SECRETS.wif;
        assert_eq!(
            wif.oidc_issuer_uri,
            "https://token.actions.githubusercontent.com"
        );
        assert!(
            wif.attribute_mapping
                .iter()
                .any(|(k, v)| *k == "google.subject" && *v == "assertion.sub"),
            "wif attribute mapping should include google.subject projection"
        );
        assert_eq!(
            wif.attribute_condition,
            Some("assertion.repository_owner == 'gunb-ai'")
        );
    }

    #[test]
    fn to_cloud_secret_config_dev() {
        let config = GUNBAI_SECRETS
            .to_cloud_secret_config("dev", CloudRuntimeKind::LocalDev)
            .expect("dev config should resolve");

        assert_eq!(config.provider, CloudProviderKind::Gcp);
        assert_eq!(config.runtime, CloudRuntimeKind::LocalDev);
        assert_eq!(config.project_or_account, "gunbai-secrets");
        assert_eq!(config.secret.prefix, "dev-");
        assert_eq!(
            config.audience,
            "projects/314501921854/locations/global/workloadIdentityPools/github-pool/providers/github"
        );
        assert_eq!(
            config.service_account_or_role,
            Some("gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string())
        );
        assert_eq!(config.impersonate_account_or_role, None);
    }

    #[test]
    fn to_cloud_secret_config_ci_github_actions() {
        let config = GUNBAI_SECRETS
            .to_cloud_secret_config("ci", CloudRuntimeKind::GitHubActions)
            .expect("ci config should resolve");

        assert_eq!(config.runtime, CloudRuntimeKind::GitHubActions);
        assert_eq!(config.secret.prefix, "ci-");
        assert_eq!(
            config.service_account_or_role,
            Some("gunbai-ci-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string())
        );
    }

    #[test]
    fn to_cloud_secret_config_prod_uses_empty_secret_prefix() {
        let config = GUNBAI_SECRETS
            .to_cloud_secret_config("prod", CloudRuntimeKind::LocalDev)
            .expect("prod config should resolve");
        assert_eq!(config.secret.prefix, "");
    }

    #[test]
    fn namespace_environment_fields_are_populated() {
        for ns in GUNBAI_SECRETS.namespaces {
            assert!(
                !ns.project.trim().is_empty(),
                "namespace {} missing project",
                ns.name
            );
            assert!(
                !ns.project_number.trim().is_empty(),
                "namespace {} missing project_number",
                ns.name
            );
            assert!(
                !ns.region.trim().is_empty(),
                "namespace {} missing region",
                ns.name
            );
            assert!(
                !ns.zone.trim().is_empty(),
                "namespace {} missing zone",
                ns.name
            );
            assert!(
                !ns.name_prefix.trim().is_empty(),
                "namespace {} missing name_prefix",
                ns.name
            );
            assert!(
                !ns.secrets_project.trim().is_empty(),
                "namespace {} missing secrets_project",
                ns.name
            );
        }
    }

    #[test]
    fn unknown_namespace_returns_none() {
        assert!(GUNBAI_SECRETS
            .to_cloud_secret_config("nonexistent", CloudRuntimeKind::LocalDev)
            .is_none());
    }

    #[test]
    fn sa_email_format_is_valid() {
        for ns in GUNBAI_SECRETS.namespaces {
            let email = ns.service_account_email();
            assert!(
                email.contains('@') && email.ends_with(".iam.gserviceaccount.com"),
                "SA email must be valid format: {email}"
            );
        }
    }

    #[test]
    fn all_namespaces_have_unique_names() {
        let mut seen = std::collections::HashSet::new();
        for ns in GUNBAI_SECRETS.namespaces {
            assert!(
                seen.insert(ns.name),
                "duplicate namespace name: {}",
                ns.name
            );
        }
    }

    #[test]
    fn all_namespaces_have_unique_prefixes() {
        let mut seen = std::collections::HashSet::new();
        for ns in GUNBAI_SECRETS.namespaces {
            let prefix = ns.secret_prefix();
            assert!(seen.insert(prefix.clone()), "duplicate prefix: {prefix}");
        }
    }

    #[test]
    fn service_account_catalog_expanded_beyond_dev_and_ci() {
        assert!(
            GUNBAI_SECRETS.namespaces.len() >= 9,
            "expected expanded namespace/service-account catalog"
        );
    }

    #[test]
    fn service_account_specs_define_display_name_roles_and_wif_bindings() {
        for ns in GUNBAI_SECRETS.namespaces {
            let sa = &ns.secrets_service_account;
            assert!(
                !sa.display_name.trim().is_empty(),
                "display_name must be non-empty for namespace {}",
                ns.name
            );
            assert!(
                !sa.self_roles.is_empty(),
                "self_roles must be non-empty for namespace {}",
                ns.name
            );
            assert!(
                !sa.wif_bindings.is_empty(),
                "wif_bindings must be non-empty for namespace {}",
                ns.name
            );
        }
    }

    #[test]
    fn service_account_binding_helpers_emit_expected_roles() {
        let dev = GUNBAI_SECRETS
            .namespace("dev")
            .expect("dev namespace must exist");
        let sa = &dev.secrets_service_account;

        let self_bindings = sa.self_role_bindings(GUNBAI_SECRETS.secrets_project.project_id);
        assert!(
            self_bindings
                .iter()
                .any(|b| b.role == "roles/secretmanager.secretAccessor"),
            "self-role bindings should include secret accessor role"
        );
        assert!(
            self_bindings
                .iter()
                .flat_map(|b| b.members.iter())
                .any(|member| member.starts_with("serviceAccount:gunbai-dev-secrets@")),
            "self-role binding member should target dev service account"
        );

        let wif_bindings = sa.wif_impersonation_bindings();
        assert_eq!(
            wif_bindings.len(),
            1,
            "expected one WIF impersonation binding"
        );
        assert_eq!(
            wif_bindings[0].role, "roles/iam.workloadIdentityUser",
            "WIF bindings should grant workload identity user role"
        );
        assert!(
            wif_bindings[0]
                .members
                .iter()
                .any(|m| m.contains("workloadIdentityPools/github-pool")),
            "WIF binding should reference canonical workload identity pool"
        );
    }

    #[test]
    fn all_secrets_have_unique_ids() {
        let mut seen = std::collections::HashSet::new();
        for s in GUNBAI_SECRETS.secrets {
            assert!(
                seen.insert(s.secret_id),
                "duplicate secret_id: {}",
                s.secret_id
            );
        }
    }

    #[test]
    fn all_secrets_have_unique_env_names() {
        let mut seen = std::collections::HashSet::new();
        for s in GUNBAI_SECRETS.secrets {
            assert!(
                seen.insert(s.env_name),
                "duplicate env_name: {}",
                s.env_name
            );
        }
    }

    #[test]
    fn prefixed_secret_id_is_correct() {
        let ns = GUNBAI_SECRETS.namespace("dev").unwrap();
        let github = GUNBAI_SECRETS
            .secrets
            .iter()
            .find(|s| s.secret_id == GITHUB_SECRET_ID)
            .expect("github-token must exist");
        assert_eq!(github.prefixed_id(ns), "dev-github-token");
    }

    #[test]
    fn active_secrets_excludes_deleted() {
        let active: Vec<_> = GUNBAI_SECRETS.active_secrets().collect();
        assert!(
            active.iter().all(|s| s.status == SecretStatus::Active),
            "active_secrets should only return active secrets"
        );
    }

    #[test]
    fn secrets_catalog_is_not_empty() {
        assert!(
            GUNBAI_SECRETS.active_secrets().count() > 0,
            "at least one active secret must exist"
        );
    }
}
