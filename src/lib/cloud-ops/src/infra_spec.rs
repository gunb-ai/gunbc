//! Unified infrastructure specification model.

use crate::project_spec::{ProjectSpec, SecretSpec, ServiceAccountSpec, WifConfig, GUNBAI_SECRETS};
use std::sync::LazyLock;

/// Environment-level infrastructure configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentConfig {
    pub environment: &'static str,
    pub project: &'static str,
    pub project_number: &'static str,
    pub region: &'static str,
    pub zone: &'static str,
    pub domain: Option<&'static str>,
    pub name_prefix: &'static str,
    pub secrets_project: &'static str,
    pub secrets_prefix: &'static str,
}

/// Unified infrastructure spec for one environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfraSpec {
    pub environment: &'static str,
    pub config: EnvironmentConfig,
    pub service_accounts: &'static [ServiceAccountSpec],
    pub secrets: &'static [SecretSpec],
    pub wif: WifConfig,
}

impl InfraSpec {
    /// Build a spec for an environment from the canonical project spec.
    pub fn from_project_spec(
        project_spec: &'static ProjectSpec,
        environment: &'static str,
    ) -> Result<Self, String> {
        let ns = project_spec
            .namespace(environment)
            .ok_or_else(|| format!("unknown environment '{}'", environment))?;

        let config = EnvironmentConfig {
            environment: ns.name,
            project: ns.project,
            project_number: ns.project_number,
            region: ns.region,
            zone: ns.zone,
            domain: ns.domain,
            name_prefix: ns.name_prefix,
            secrets_project: ns.secrets_project,
            secrets_prefix: ns.secrets_prefix,
        };

        Ok(Self {
            environment: ns.name,
            config,
            service_accounts: std::slice::from_ref(&ns.secrets_service_account),
            secrets: project_spec.secrets,
            wif: project_spec.wif.clone(),
        })
    }

    /// Validate cross-resource consistency.
    pub fn validate(&self) -> Result<(), String> {
        if self.environment.trim().is_empty() {
            return Err("infra spec environment must be non-empty".to_string());
        }
        if self.config.project.trim().is_empty() {
            return Err("infra spec project must be non-empty".to_string());
        }
        if self.config.region.trim().is_empty() || self.config.zone.trim().is_empty() {
            return Err("infra spec region/zone must be non-empty".to_string());
        }
        if self.service_accounts.is_empty() {
            return Err("infra spec must include at least one service account".to_string());
        }
        if self.secrets.is_empty() {
            return Err("infra spec must include at least one secret spec".to_string());
        }
        if self.wif.pool_id.trim().is_empty() || self.wif.provider_id.trim().is_empty() {
            return Err("infra spec WIF identifiers must be non-empty".to_string());
        }
        Ok(())
    }
}

fn load_spec(environment: &'static str) -> InfraSpec {
    InfraSpec::from_project_spec(&GUNBAI_SECRETS, environment)
        .unwrap_or_else(|err| panic!("failed to build {} infra spec: {}", environment, err))
}

pub static DEV_SPEC: LazyLock<InfraSpec> = LazyLock::new(|| load_spec("dev"));
pub static CI_SPEC: LazyLock<InfraSpec> = LazyLock::new(|| load_spec("ci"));
pub static TEST_SPEC: LazyLock<InfraSpec> = LazyLock::new(|| load_spec("test"));
pub static PROD_SPEC: LazyLock<InfraSpec> = LazyLock::new(|| load_spec("prod"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_spec_constants_are_loadable() {
        assert_eq!(DEV_SPEC.environment, "dev");
        assert_eq!(CI_SPEC.environment, "ci");
        assert_eq!(TEST_SPEC.environment, "test");
        assert_eq!(PROD_SPEC.environment, "prod");
    }

    #[test]
    fn infra_spec_validation_passes_for_canonical_envs() {
        assert!(DEV_SPEC.validate().is_ok());
        assert!(CI_SPEC.validate().is_ok());
        assert!(TEST_SPEC.validate().is_ok());
        assert!(PROD_SPEC.validate().is_ok());
    }

    #[test]
    fn infra_spec_from_project_spec_errors_on_unknown_environment() {
        let err = InfraSpec::from_project_spec(&GUNBAI_SECRETS, "unknown")
            .expect_err("unknown env should fail");
        assert!(err.contains("unknown environment"));
    }
}
