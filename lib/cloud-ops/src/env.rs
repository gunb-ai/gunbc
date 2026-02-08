//! Cloud environment acquisition (provider-neutral baseline).

use gunbc_exec::{EnvNode, ExecError, OutputMap};
use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Cloud environment — acquires cloud secret config and runtime OIDC inputs.
#[derive(Debug, Clone)]
pub struct CloudEnv;

impl CloudEnv {
    pub fn new() -> Self {
        Self
    }

    pub fn output_port(&self) -> &'static str {
        "config"
    }

    fn detect_provider() -> Result<CloudProviderKind, ExecError> {
        let provider = std::env::var("CLOUD_PROVIDER").unwrap_or_else(|_| "gcp".to_string());
        CloudProviderKind::parse(&provider).ok_or_else(|| {
            ExecError::new(format!(
                "unknown CLOUD_PROVIDER '{provider}' (expected gcp|aws|azure)"
            ))
        })
    }

    fn detect_runtime() -> Result<CloudRuntimeKind, ExecError> {
        if let Ok(runtime) = std::env::var("CLOUD_RUNTIME") {
            return CloudRuntimeKind::parse(&runtime).ok_or_else(|| {
                ExecError::new(format!(
                    "unknown CLOUD_RUNTIME '{runtime}' (expected github|metadata|local)"
                ))
            });
        }

        if std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").is_ok() {
            return Ok(CloudRuntimeKind::GitHubActions);
        }

        if std::env::var("GCE_METADATA_HOST").is_ok()
            || std::env::var("K_SERVICE").is_ok()
            || std::env::var("K_REVISION").is_ok()
        {
            return Ok(CloudRuntimeKind::CloudMetadata);
        }

        Ok(CloudRuntimeKind::LocalDev)
    }

    fn build_gcp_config(runtime: CloudRuntimeKind) -> Result<CloudSecretConfig, ExecError> {
        let audience = match runtime {
            CloudRuntimeKind::GitHubActions | CloudRuntimeKind::CloudMetadata => {
                std::env::var("GCP_WIF_PROVIDER").map_err(|_| {
                    ExecError::new("missing GCP_WIF_PROVIDER (WIF audience/provider)")
                })?
            }
            CloudRuntimeKind::LocalDev => {
                std::env::var("GCP_WIF_PROVIDER").unwrap_or_else(|_| "local-dev".to_string())
            }
        };
        let project = std::env::var("GCP_SECRETS_PROJECT")
            .map_err(|_| ExecError::new("missing GCP_SECRETS_PROJECT"))?;
        let prefix = std::env::var("GCP_SECRETS_PREFIX")
            .map_err(|_| ExecError::new("missing GCP_SECRETS_PREFIX"))?;

        let service_account = std::env::var("GCP_SECRETS_SA").ok();
        let impersonate = std::env::var("GCP_SECRETS_IMPERSONATE_SA").ok();

        Ok(CloudSecretConfig {
            provider: CloudProviderKind::Gcp,
            runtime,
            audience,
            project_or_account: project,
            secret: CloudSecretRef {
                prefix,
                name: String::new(),
                delimiter: String::new(),
                version: None,
            },
            service_account_or_role: service_account,
            impersonate_account_or_role: impersonate,
        })
    }

    fn outputs_from_config(
        &self,
        config: CloudSecretConfig,
        runtime: CloudRuntimeKind,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let mut out = OutputMap::new().value(self.output_port(), config.into());

        if matches!(runtime, CloudRuntimeKind::GitHubActions) {
            let request_url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL")
                .map_err(|_| ExecError::new("missing ACTIONS_ID_TOKEN_REQUEST_URL"))?;
            let request_token = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN")
                .map_err(|_| ExecError::new("missing ACTIONS_ID_TOKEN_REQUEST_TOKEN"))?;
            out = out
                .str("request_url", request_url)
                .str("request_token", request_token);
        }

        Ok(out.build())
    }

    fn mock_outputs_impl(&self) -> HashMap<String, Value> {
        let config = CloudSecretConfig {
            provider: CloudProviderKind::Gcp,
            runtime: CloudRuntimeKind::GitHubActions,
            audience: "projects/123/locations/global/workloadIdentityPools/github/providers/gha"
                .to_string(),
            project_or_account: "mock-secrets".to_string(),
            secret: CloudSecretRef {
                prefix: "ci-".to_string(),
                name: String::new(),
                delimiter: String::new(),
                version: None,
            },
            service_account_or_role: Some("ci-secrets@mock.iam.gserviceaccount.com".to_string()),
            impersonate_account_or_role: None,
        };

        OutputMap::new()
            .value(self.output_port(), config.into())
            .str("request_url", "https://example.com/oidc")
            .str("request_token", "mock-oidc-token")
            .build()
    }
}

impl Default for CloudEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvNode for CloudEnv {
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError> {
        let provider = Self::detect_provider()?;
        let runtime = Self::detect_runtime()?;

        let config = match provider {
            CloudProviderKind::Gcp => Self::build_gcp_config(runtime)?,
            CloudProviderKind::Aws => {
                return Err(ExecError::new(
                    "CLOUD_PROVIDER=aws env support is not implemented yet",
                ))
            }
            CloudProviderKind::Azure => {
                return Err(ExecError::new(
                    "CLOUD_PROVIDER=azure env support is not implemented yet",
                ))
            }
        };

        self.outputs_from_config(config, runtime)
    }

    fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_outputs_impl()
    }
}
