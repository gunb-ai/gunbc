//! Config loader: TOML loading, namespace resolution, CloudSecretConfig generation.
//!
//! This module handles:
//! 1. Reading the generated TOML config file
//! 2. Deserializing it into `CloudConfigSpec`
//! 3. Resolving namespace inheritance
//! 4. Converting to runtime `CloudSecretConfig`

use gunbc_ir::transport::cloud::{
    CloudConfigSpec, CloudNamespace, CloudProviderKind, CloudRuntimeKind, CloudSecretConfig,
    CloudSecretRef,
};

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
/// The values are sensible defaults for the `gunbai-secrets` project.
/// For real deployments, config should come from the generated TOML
/// via [`resolve_default_secret_config`].
pub fn default_local_dev_config() -> CloudSecretConfig {
    CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime: detect_runtime(),
        audience: "local-dev".to_string(),
        project_or_account: "gunbai-secrets".to_string(),
        secret: CloudSecretRef {
            prefix: "dev-".to_string(),
            name: String::new(),
            delimiter: String::new(),
            version: None,
        },
        service_account_or_role: Some(
            "gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string(),
        ),
        impersonate_account_or_role: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
        let spec = parse_config_toml(SAMPLE_TOML).unwrap();
        let config = spec
            .to_secret_config("dev", CloudRuntimeKind::LocalDev, "github-token")
            .unwrap();
        assert_eq!(config.provider, CloudProviderKind::Gcp);
        assert_eq!(config.runtime, CloudRuntimeKind::LocalDev);
        assert_eq!(config.project_or_account, "gunbai-secrets");
        assert_eq!(config.secret.prefix, "dev-");
        assert_eq!(config.secret.name, "github-token");
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
}
