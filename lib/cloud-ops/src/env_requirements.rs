//! Environment variable requirements for cloud secret manager flows.

use gunbc_ir::transport::cloud::{CloudProviderKind, CloudRuntimeKind};

#[derive(Debug, Clone, Copy)]
pub struct CloudEnvRequirements {
    pub provider: CloudProviderKind,
    pub runtime: CloudRuntimeKind,
    pub required: &'static [&'static str],
    pub required_any_of: &'static [&'static [&'static str]],
    pub optional: &'static [&'static str],
    pub notes: Option<&'static str>,
}

pub const CLOUD_ENV_COMMON_OPTIONAL: &[&str] = &["CLOUD_PROVIDER", "CLOUD_RUNTIME"];

pub const GCP_GITHUB_REQUIRED: &[&str] = &[
    "GCP_WIF_PROVIDER",
    "GCP_SECRETS_PROJECT",
    "GCP_SECRETS_PREFIX",
    "ACTIONS_ID_TOKEN_REQUEST_URL",
    "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
];

pub const GCP_METADATA_REQUIRED: &[&str] = &[
    "GCP_WIF_PROVIDER",
    "GCP_SECRETS_PROJECT",
    "GCP_SECRETS_PREFIX",
];

pub const GCP_LOCAL_REQUIRED: &[&str] = &["GCP_SECRETS_PROJECT", "GCP_SECRETS_PREFIX"];

pub const GCP_REQUIRED_ANY_OF: &[&[&str]] = &[&["GCP_SECRETS_SA", "GCP_SECRETS_IMPERSONATE_SA"]];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingCloudEnvRequirements {
    pub missing_required: Vec<&'static str>,
    pub missing_any_of: Vec<Vec<&'static str>>,
}

impl MissingCloudEnvRequirements {
    pub fn is_empty(&self) -> bool {
        self.missing_required.is_empty() && self.missing_any_of.is_empty()
    }
}

pub fn gcp_github_actions_env() -> CloudEnvRequirements {
    CloudEnvRequirements {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::GitHubActions,
        required: GCP_GITHUB_REQUIRED,
        required_any_of: GCP_REQUIRED_ANY_OF,
        optional: CLOUD_ENV_COMMON_OPTIONAL,
        notes: Some("Primary CI/prod path (GCP WIF via GitHub Actions)."),
    }
}

pub fn gcp_metadata_env() -> CloudEnvRequirements {
    CloudEnvRequirements {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::CloudMetadata,
        required: GCP_METADATA_REQUIRED,
        required_any_of: GCP_REQUIRED_ANY_OF,
        optional: CLOUD_ENV_COMMON_OPTIONAL,
        notes: Some("Metadata runtime (GCE/GKE) for prod deployments."),
    }
}

pub fn gcp_local_env() -> CloudEnvRequirements {
    CloudEnvRequirements {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::LocalDev,
        required: GCP_LOCAL_REQUIRED,
        required_any_of: GCP_REQUIRED_ANY_OF,
        optional: CLOUD_ENV_COMMON_OPTIONAL,
        notes: Some("Local dev path (gcloud auth + Secret Manager)."),
    }
}

pub fn aws_github_actions_env_stub() -> CloudEnvRequirements {
    CloudEnvRequirements {
        provider: CloudProviderKind::Aws,
        runtime: CloudRuntimeKind::GitHubActions,
        required: &[],
        required_any_of: &[],
        optional: CLOUD_ENV_COMMON_OPTIONAL,
        notes: Some("Stub: define AWS WIF + Secrets Manager env vars."),
    }
}

pub fn azure_github_actions_env_stub() -> CloudEnvRequirements {
    CloudEnvRequirements {
        provider: CloudProviderKind::Azure,
        runtime: CloudRuntimeKind::GitHubActions,
        required: &[],
        required_any_of: &[],
        optional: CLOUD_ENV_COMMON_OPTIONAL,
        notes: Some("Stub: define Azure federated credential + Key Vault env vars."),
    }
}

pub fn cloud_env_matrix() -> Vec<CloudEnvRequirements> {
    vec![
        gcp_github_actions_env(),
        gcp_metadata_env(),
        gcp_local_env(),
        aws_github_actions_env_stub(),
        azure_github_actions_env_stub(),
    ]
}

pub fn requirements_for(
    provider: CloudProviderKind,
    runtime: CloudRuntimeKind,
) -> CloudEnvRequirements {
    match provider {
        CloudProviderKind::Gcp => match runtime {
            CloudRuntimeKind::GitHubActions => gcp_github_actions_env(),
            CloudRuntimeKind::CloudMetadata => gcp_metadata_env(),
            CloudRuntimeKind::LocalDev => gcp_local_env(),
        },
        CloudProviderKind::Aws => aws_github_actions_env_stub(),
        CloudProviderKind::Azure => azure_github_actions_env_stub(),
    }
}

pub fn detect_provider_runtime() -> (CloudProviderKind, CloudRuntimeKind) {
    let provider = std::env::var("CLOUD_PROVIDER")
        .ok()
        .and_then(|v| CloudProviderKind::parse(&v))
        .unwrap_or(CloudProviderKind::Gcp);

    let runtime = if let Ok(runtime) = std::env::var("CLOUD_RUNTIME") {
        CloudRuntimeKind::parse(&runtime).unwrap_or(CloudRuntimeKind::LocalDev)
    } else if std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").is_ok() {
        CloudRuntimeKind::GitHubActions
    } else if std::env::var("GCE_METADATA_HOST").is_ok()
        || std::env::var("K_SERVICE").is_ok()
        || std::env::var("K_REVISION").is_ok()
    {
        CloudRuntimeKind::CloudMetadata
    } else {
        CloudRuntimeKind::LocalDev
    };

    (provider, runtime)
}

pub fn collect_missing_requirements(req: &CloudEnvRequirements) -> MissingCloudEnvRequirements {
    let missing_required = req
        .required
        .iter()
        .copied()
        .filter(|name| std::env::var(name).is_err())
        .collect();

    let missing_any_of = req
        .required_any_of
        .iter()
        .filter(|group| !group.iter().any(|name| std::env::var(name).is_ok()))
        .map(|group| group.to_vec())
        .collect();

    MissingCloudEnvRequirements {
        missing_required,
        missing_any_of,
    }
}

pub fn format_missing_requirements_message(
    req: &CloudEnvRequirements,
    missing: &MissingCloudEnvRequirements,
) -> String {
    let mut parts = Vec::new();
    if !missing.missing_required.is_empty() {
        parts.push(format!(
            "provider-runtime required: {}",
            missing.missing_required.join(", ")
        ));
    }
    if !missing.missing_any_of.is_empty() {
        let groups = missing
            .missing_any_of
            .iter()
            .map(|group| group.join(" | "))
            .collect::<Vec<_>>()
            .join(", ");
        parts.push(format!("developer identity required-any-of: {groups}"));
    }

    format!(
        "missing cloud environment for {}/{}: {}; selectors: CLOUD_PROVIDER, CLOUD_RUNTIME",
        req.provider.as_str(),
        req.runtime.as_str(),
        parts.join("; "),
    )
}

/// Detect the most likely environment requirements from current env vars.
pub fn detect_cloud_env_requirements() -> CloudEnvRequirements {
    let (provider, runtime) = detect_provider_runtime();
    requirements_for(provider, runtime)
}
