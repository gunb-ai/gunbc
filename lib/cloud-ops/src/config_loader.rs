//! Config loader: TOML loading, namespace resolution, CloudSecretConfig generation.
//!
//! This module handles:
//! 1. Reading the generated TOML config file
//! 2. Deserializing it into `CloudConfigSpec`
//! 3. Resolving namespace inheritance
//! 4. Converting to runtime `CloudSecretConfig`

use std::fmt;

use gunbc_ir::transport::cloud::{
    CloudConfigSpec, CloudNamespace, CloudProviderKind, CloudRuntimeKind, CloudSecretConfig,
    CloudSecretRef,
};

// ---------------------------------------------------------------------------
// Config error types
// ---------------------------------------------------------------------------

/// Error type for cloud config resolution.
///
/// Distinguishes between "no config source is set" (safe to fall back to
/// dev defaults) and "a config source is present but invalid" (must surface
/// the misconfiguration immediately).
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// No config env vars are set at all -- safe to use dev defaults.
    NotConfigured(String),
    /// A config source is present but malformed or references an unknown namespace.
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotConfigured(msg) => write!(f, "not configured: {msg}"),
            ConfigError::Invalid(msg) => write!(f, "invalid config: {msg}"),
        }
    }
}

const ENV_CONFIG_JSON: &str = "GUNBC_CLOUD_CONFIG_JSON";
const ENV_CONFIG_TOML: &str = "GUNBC_CLOUD_CONFIG_TOML";
const ENV_PROFILE: &str = "GUNBC_CLOUD_PROFILE";
const ENV_NAMESPACE: &str = "GUNBC_CLOUD_NAMESPACE";
const ENV_SECRET_VERSION: &str = "GUNBC_CLOUD_SECRET_VERSION";
const ENV_CONFIG_REQUIRED: &str = "GUNBC_CLOUD_CONFIG_REQUIRED";

const LEGACY_ENV_WIF_PROVIDER: &str = "GCP_WIF_PROVIDER";
const LEGACY_ENV_SECRETS_PROJECT: &str = "GCP_SECRETS_PROJECT";
const LEGACY_ENV_SECRETS_PREFIX: &str = "GCP_SECRETS_PREFIX";
const LEGACY_ENV_SECRETS_SA: &str = "GCP_SECRETS_SA";
const LEGACY_ENV_IMPERSONATE_SA: &str = "GCP_SECRETS_IMPERSONATE_SA";

// ---------------------------------------------------------------------------
// TOML parser (minimal, avoids pulling in toml crate)
// ---------------------------------------------------------------------------

/// Parse a minimal TOML config into a `CloudConfigSpec`.
///
/// Supports the subset of TOML used by our config format:
/// - Top-level key = "value" pairs
/// - `[[namespaces]]` array of tables
///
/// This is not a full TOML parser -- it handles our specific format only.
pub fn parse_config_toml(input: &str) -> Result<CloudConfigSpec, String> {
    let mut default_namespace: Option<String> = None;
    let mut generated_at: Option<String> = None;
    let mut source_project: Option<String> = None;
    let mut namespaces: Vec<CloudNamespace> = Vec::new();
    let mut current_ns: Option<CloudNamespace> = None;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "[[namespaces]]" {
            // Flush current namespace
            if let Some(ns) = current_ns.take() {
                namespaces.push(ns);
            }
            current_ns = Some(CloudNamespace {
                name: String::new(),
                inherits_from: None,
                provider: None,
                secrets_project: None,
                wif_provider: None,
                service_account: None,
                impersonate_account: None,
            });
            continue;
        }

        if let Some((key, value)) = parse_kv(trimmed) {
            if let Some(ref mut ns) = current_ns {
                match key {
                    "name" => ns.name = value,
                    "inherits_from" => ns.inherits_from = Some(value),
                    "provider" => ns.provider = CloudProviderKind::parse(&value),
                    "secrets_project" => ns.secrets_project = Some(value),
                    "wif_provider" => ns.wif_provider = Some(value),
                    "service_account" => ns.service_account = Some(value),
                    "impersonate_account" => ns.impersonate_account = Some(value),
                    _ => {} // ignore unknown keys
                }
            } else {
                match key {
                    "default_namespace" => default_namespace = Some(value),
                    "generated_at" => generated_at = Some(value),
                    "source_project" => source_project = Some(value),
                    _ => {}
                }
            }
        }
    }

    // Flush last namespace
    if let Some(ns) = current_ns.take() {
        namespaces.push(ns);
    }

    Ok(CloudConfigSpec {
        namespaces,
        default_namespace,
        generated_at,
        source_project,
    })
}

/// Parse a TOML key = "value" line.
fn parse_kv(line: &str) -> Option<(&str, String)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let value = rest.trim();
    // Strip surrounding quotes
    let value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        &value[1..value.len() - 1]
    } else {
        value
    };
    Some((key, value.to_string()))
}

// ---------------------------------------------------------------------------
// Runtime detection
// ---------------------------------------------------------------------------

/// Detect the current cloud runtime kind from environment signals.
///
/// Checks environment variables to determine whether we're running in:
/// - GitHub Actions (GITHUB_ACTIONS=true)
/// - GCP metadata server (GCE_METADATA_HOST is set)
/// - Local development (default)
pub fn detect_runtime() -> CloudRuntimeKind {
    if std::env::var("GITHUB_ACTIONS").ok().as_deref() == Some("true") {
        return CloudRuntimeKind::GitHubActions;
    }
    if std::env::var("GCE_METADATA_HOST").is_ok() {
        return CloudRuntimeKind::CloudMetadata;
    }
    CloudRuntimeKind::LocalDev
}

// ---------------------------------------------------------------------------
// Graph config resolution (centralized precedence)
// ---------------------------------------------------------------------------

/// Resolve graph cloud config using deterministic precedence.
///
/// Precedence:
/// 1) `GUNBC_CLOUD_CONFIG_JSON` (serialized `CloudSecretConfig`)
/// 2) `GUNBC_CLOUD_CONFIG_TOML` (+ namespace/profile selection)
/// 3) Legacy env model (`GCP_*`)
///
/// Returns `ConfigError::NotConfigured` when no config source is set at all,
/// and `ConfigError::Invalid` when a source is present but malformed or
/// references an unknown namespace.
pub fn resolve_graph_cloud_config() -> Result<CloudSecretConfig, ConfigError> {
    if let Some(raw_json) = env_nonempty(ENV_CONFIG_JSON) {
        let mut config: CloudSecretConfig = serde_json::from_str(&raw_json).map_err(|e| {
            ConfigError::Invalid(format!(
                "{ENV_CONFIG_JSON} must contain valid CloudSecretConfig JSON: {e}"
            ))
        })?;
        if let Some(version) = env_nonempty(ENV_SECRET_VERSION) {
            config.secret.version = Some(version);
        }
        return Ok(config);
    }

    if let Some(raw_toml) = env_nonempty(ENV_CONFIG_TOML) {
        let spec = parse_config_toml(&raw_toml)
            .map_err(|e| ConfigError::Invalid(format!("failed to parse {ENV_CONFIG_TOML}: {e}")))?;
        let runtime = runtime_with_override();
        let namespace = env_nonempty(ENV_NAMESPACE)
            .or_else(|| env_nonempty(ENV_PROFILE))
            .or_else(|| spec.default_namespace.clone())
            .ok_or_else(|| {
                ConfigError::Invalid(format!(
                    "{ENV_CONFIG_TOML} is set but no namespace/profile resolved; set {ENV_NAMESPACE} or {ENV_PROFILE}, or include default_namespace"
                ))
            })?;

        let mut config = spec
            .to_secret_config(&namespace, runtime, "")
            .ok_or_else(|| {
                ConfigError::Invalid(format!(
                    "namespace '{namespace}' could not be resolved from config"
                ))
            })?;
        if let Some(version) = env_nonempty(ENV_SECRET_VERSION) {
            config.secret.version = Some(version);
        }
        return Ok(config);
    }

    if let Some(config) = resolve_legacy_gcp_env_config() {
        return Ok(config);
    }

    Err(ConfigError::NotConfigured(format!(
        "no cloud config source found (expected {ENV_CONFIG_JSON} or {ENV_CONFIG_TOML}, or legacy {LEGACY_ENV_SECRETS_PROJECT}/{LEGACY_ENV_SECRETS_PREFIX})"
    )))
}

/// Resolve graph cloud config with compatibility fallback.
///
/// When no config source is present at all (`NotConfigured`), this falls back
/// to `default_local_dev_config()` unless `GUNBC_CLOUD_CONFIG_REQUIRED=1|true`.
///
/// When a config source **is** present but malformed or references an unknown
/// namespace (`Invalid`), this always panics -- silently using dev defaults
/// for broken config would route credentials to the wrong project/prefix.
pub fn graph_cloud_config() -> CloudSecretConfig {
    match resolve_graph_cloud_config() {
        Ok(config) => config,
        Err(ConfigError::Invalid(msg)) => {
            panic!("cloud config is present but invalid: {msg}");
        }
        Err(ConfigError::NotConfigured(msg)) => {
            if env_truthy(ENV_CONFIG_REQUIRED) {
                panic!("cloud config resolution failed: {msg}");
            }
            default_local_dev_config()
        }
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn env_truthy(name: &str) -> bool {
    env_nonempty(name)
        .map(|v| {
            matches!(
                v.as_str(),
                "1" | "true" | "TRUE" | "True" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

fn runtime_with_override() -> CloudRuntimeKind {
    env_nonempty("CLOUD_RUNTIME")
        .and_then(|v| CloudRuntimeKind::parse(&v))
        .unwrap_or_else(detect_runtime)
}

fn resolve_legacy_gcp_env_config() -> Option<CloudSecretConfig> {
    if let Some(provider) =
        env_nonempty("CLOUD_PROVIDER").and_then(|v| CloudProviderKind::parse(&v))
    {
        if provider != CloudProviderKind::Gcp {
            return None;
        }
    }

    let project = env_nonempty(LEGACY_ENV_SECRETS_PROJECT)?;
    let prefix = env_nonempty(LEGACY_ENV_SECRETS_PREFIX)?;
    let audience = env_nonempty(LEGACY_ENV_WIF_PROVIDER).unwrap_or_else(|| "local-dev".to_string());
    let runtime = runtime_with_override();

    Some(CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime,
        audience,
        project_or_account: project,
        secret: CloudSecretRef {
            prefix,
            name: String::new(),
            delimiter: String::new(),
            version: env_nonempty(ENV_SECRET_VERSION),
        },
        service_account_or_role: env_nonempty(LEGACY_ENV_SECRETS_SA),
        impersonate_account_or_role: env_nonempty(LEGACY_ENV_IMPERSONATE_SA),
    })
}

// ---------------------------------------------------------------------------
// Config resolution
// ---------------------------------------------------------------------------

/// Load a CloudConfigSpec from a TOML file path.
pub fn load_config_from_file(content: &str) -> Result<CloudConfigSpec, String> {
    parse_config_toml(content)
}

/// Resolve a CloudSecretConfig from a spec, namespace, and secret name.
///
/// This is the main entry point for converting stored config to runtime config.
pub fn resolve_secret_config(
    spec: &CloudConfigSpec,
    namespace: &str,
    secret_name: &str,
) -> Option<CloudSecretConfig> {
    let runtime = detect_runtime();
    spec.to_secret_config(namespace, runtime, secret_name)
}

/// Resolve a CloudSecretConfig using the default namespace.
pub fn resolve_default_secret_config(
    spec: &CloudConfigSpec,
    secret_name: &str,
) -> Option<CloudSecretConfig> {
    let namespace = spec.default_namespace.as_deref()?;
    resolve_secret_config(spec, namespace, secret_name)
}

// ---------------------------------------------------------------------------
// Default config (replaces legacy CloudEnv)
// ---------------------------------------------------------------------------

/// Provide a default `CloudSecretConfig` for local development.
///
/// This is used when no explicit config is provided (e.g., in tests,
/// graph construction for mock specs, or early bootstrapping).
///
/// Always uses `CloudRuntimeKind::LocalDev` regardless of environment
/// signals like `GITHUB_ACTIONS`. This is a *fallback* for when no
/// explicit config is set — it must produce a deterministic graph
/// topology so that mock specs (which target LocalDev) stay consistent
/// across local and CI environments. For CI deployments that need a
/// different runtime, set `GUNBC_CLOUD_CONFIG_TOML` or
/// `GUNBC_CLOUD_CONFIG_JSON` explicitly.
///
/// Values are derived from the canonical project spec
/// ([`crate::project_spec::GUNBAI_SECRETS`]), not hardcoded strings.
/// For real deployments, config should come from the generated TOML
/// via [`resolve_default_secret_config`].
pub fn default_local_dev_config() -> CloudSecretConfig {
    crate::project_spec::GUNBAI_SECRETS
        .to_cloud_secret_config("dev", CloudRuntimeKind::LocalDev)
        .expect("dev namespace must exist in GUNBAI_SECRETS project spec")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    const SAMPLE_TOML: &str = r#"
default_namespace = "dev"
generated_at = "2026-02-08T00:00:00Z"
source_project = "gunbai-secrets"

[[namespaces]]
name = "base"
provider = "gcp"
secrets_project = "gunbai-secrets"
wif_provider = "projects/314501921854/locations/global/workloadIdentityPools/github-pool/providers/github"
service_account = "gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com"

[[namespaces]]
name = "dev"
inherits_from = "base"

[[namespaces]]
name = "prod"
inherits_from = "base"
service_account = "gunbai-prod-secrets@gunbai-secrets.iam.gserviceaccount.com"
"#;

    #[test]
    fn test_parse_config_toml() {
        let spec = parse_config_toml(SAMPLE_TOML).unwrap();
        assert_eq!(spec.default_namespace, Some("dev".to_string()));
        assert_eq!(spec.source_project, Some("gunbai-secrets".to_string()));
        assert_eq!(spec.namespaces.len(), 3);
        assert_eq!(spec.namespaces[0].name, "base");
        assert_eq!(spec.namespaces[1].name, "dev");
        assert_eq!(spec.namespaces[1].inherits_from, Some("base".to_string()));
        assert_eq!(spec.namespaces[2].name, "prod");
    }

    #[test]
    fn test_resolve_namespace_inheritance() {
        let spec = parse_config_toml(SAMPLE_TOML).unwrap();
        let dev = spec.resolve_namespace("dev").unwrap();
        assert_eq!(dev.provider, Some(CloudProviderKind::Gcp));
        assert_eq!(dev.secrets_project, Some("gunbai-secrets".to_string()));
        assert!(dev.wif_provider.is_some());
        assert_eq!(
            dev.service_account,
            Some("gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string())
        );
    }

    #[test]
    fn test_resolve_namespace_override() {
        let spec = parse_config_toml(SAMPLE_TOML).unwrap();
        let prod = spec.resolve_namespace("prod").unwrap();
        // prod overrides service_account but inherits everything else
        assert_eq!(
            prod.service_account,
            Some("gunbai-prod-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string())
        );
        assert_eq!(prod.secrets_project, Some("gunbai-secrets".to_string()));
    }

    #[test]
    fn test_to_secret_config() {
        use gunbc_ir::transport::gist::GITHUB_SECRET_ID;
        let spec = parse_config_toml(SAMPLE_TOML).unwrap();
        let config = spec
            .to_secret_config("dev", CloudRuntimeKind::LocalDev, GITHUB_SECRET_ID)
            .unwrap();
        assert_eq!(config.provider, CloudProviderKind::Gcp);
        assert_eq!(config.runtime, CloudRuntimeKind::LocalDev);
        assert_eq!(config.project_or_account, "gunbai-secrets");
        assert_eq!(config.secret.prefix, "dev-");
        assert_eq!(config.secret.name, GITHUB_SECRET_ID);
        assert_eq!(config.secret_name(), "dev-github-token");
        assert!(config.service_account_or_role.is_some());
    }

    #[test]
    fn test_namespace_prefix_derivation() {
        let ns = CloudNamespace {
            name: "staging".to_string(),
            inherits_from: None,
            provider: None,
            secrets_project: None,
            wif_provider: None,
            service_account: None,
            impersonate_account: None,
        };
        assert_eq!(ns.secret_prefix(), "staging-");
    }

    #[test]
    fn test_parse_empty_toml() {
        let spec = parse_config_toml("").unwrap();
        assert!(spec.namespaces.is_empty());
        assert!(spec.default_namespace.is_none());
    }

    #[test]
    fn resolve_graph_cloud_config_prefers_json_over_toml() {
        with_env_lock(|| {
            let json_config = CloudSecretConfig {
                provider: CloudProviderKind::Gcp,
                runtime: CloudRuntimeKind::LocalDev,
                audience: "json-audience".to_string(),
                project_or_account: "json-project".to_string(),
                secret: CloudSecretRef {
                    prefix: "json-".to_string(),
                    name: String::new(),
                    delimiter: String::new(),
                    version: None,
                },
                service_account_or_role: Some("json-sa@example".to_string()),
                impersonate_account_or_role: None,
            };
            std::env::set_var(
                ENV_CONFIG_JSON,
                serde_json::to_string(&json_config).expect("serialize json config"),
            );
            std::env::set_var(ENV_CONFIG_TOML, SAMPLE_TOML);
            std::env::set_var(ENV_PROFILE, "prod");

            let resolved = resolve_graph_cloud_config().expect("json config should resolve");
            assert_eq!(resolved.project_or_account, "json-project");
            assert_eq!(resolved.audience, "json-audience");
        });
    }

    #[test]
    fn resolve_graph_cloud_config_uses_toml_profile_when_present() {
        with_env_lock(|| {
            std::env::set_var(ENV_CONFIG_TOML, SAMPLE_TOML);
            std::env::set_var(ENV_PROFILE, "prod");

            let resolved = resolve_graph_cloud_config().expect("toml config should resolve");
            assert_eq!(resolved.project_or_account, "gunbai-secrets");
            assert_eq!(resolved.secret.prefix, "prod-");
            assert_eq!(
                resolved.service_account_or_role,
                Some("gunbai-prod-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string())
            );
        });
    }

    #[test]
    fn resolve_graph_cloud_config_uses_legacy_env_when_no_spec_source() {
        with_env_lock(|| {
            std::env::set_var(LEGACY_ENV_SECRETS_PROJECT, "legacy-project");
            std::env::set_var(LEGACY_ENV_SECRETS_PREFIX, "legacy-");
            std::env::set_var(LEGACY_ENV_WIF_PROVIDER, "legacy-audience");
            std::env::set_var(LEGACY_ENV_SECRETS_SA, "legacy-sa@example");

            let resolved = resolve_graph_cloud_config().expect("legacy env should resolve");
            assert_eq!(resolved.project_or_account, "legacy-project");
            assert_eq!(resolved.secret.prefix, "legacy-");
            assert_eq!(resolved.audience, "legacy-audience");
            assert_eq!(
                resolved.service_account_or_role,
                Some("legacy-sa@example".to_string())
            );
        });
    }

    #[test]
    fn graph_cloud_config_falls_back_to_default_when_unconfigured() {
        with_env_lock(|| {
            let resolved = graph_cloud_config();
            assert_eq!(resolved.project_or_account, "gunbai-secrets");
            assert_eq!(resolved.secret.prefix, "dev-");
        });
    }

    #[test]
    #[should_panic(expected = "cloud config resolution failed")]
    fn graph_cloud_config_panics_when_required_flag_is_set() {
        with_env_lock(|| {
            std::env::set_var(ENV_CONFIG_REQUIRED, "true");
            let _ = graph_cloud_config();
        });
    }

    #[test]
    #[should_panic(expected = "cloud config is present but invalid")]
    fn graph_cloud_config_panics_on_malformed_json() {
        with_env_lock(|| {
            std::env::set_var(ENV_CONFIG_JSON, "{ not valid json }");
            let _ = graph_cloud_config();
        });
    }

    #[test]
    #[should_panic(expected = "cloud config is present but invalid")]
    fn graph_cloud_config_panics_on_unknown_toml_namespace() {
        with_env_lock(|| {
            std::env::set_var(ENV_CONFIG_TOML, SAMPLE_TOML);
            std::env::set_var(ENV_NAMESPACE, "nonexistent");
            let _ = graph_cloud_config();
        });
    }

    fn with_env_lock<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe,
    {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        clear_cloud_config_env();
        let result = std::panic::catch_unwind(f);
        clear_cloud_config_env();
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    fn clear_cloud_config_env() {
        for key in [
            ENV_CONFIG_JSON,
            ENV_CONFIG_TOML,
            ENV_PROFILE,
            ENV_NAMESPACE,
            ENV_SECRET_VERSION,
            ENV_CONFIG_REQUIRED,
            "CLOUD_PROVIDER",
            "CLOUD_RUNTIME",
            LEGACY_ENV_WIF_PROVIDER,
            LEGACY_ENV_SECRETS_PROJECT,
            LEGACY_ENV_SECRETS_PREFIX,
            LEGACY_ENV_SECRETS_SA,
            LEGACY_ENV_IMPERSONATE_SA,
        ] {
            std::env::remove_var(key);
        }
    }
}
