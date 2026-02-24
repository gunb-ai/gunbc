//! Provider-neutral cloud credential graphs.
//!
//! This crate stitches provider-specific secret manager DAGs together behind a
//! stable interface so callers can switch providers without reworking DAG shapes.

pub mod config_loader;
pub mod config_resource;
pub mod credential_policy;
pub mod env_requirements;
mod env_status;
mod github_ops;
mod graph;
pub mod health_status;
pub mod infra_bootstrap;
pub mod infra_graph;
pub mod infra_plan_apply;
pub mod infra_spec;
pub mod login_flow;
mod ops;
pub mod project_registry;
pub mod project_spec;
pub mod secret_cache;
pub mod secret_exports;
pub mod secret_provision_graph;
pub mod secret_rotation;

pub use config_loader::{
    default_local_dev_config, graph_cloud_config, resolve_graph_cloud_config,
    resolve_graph_cloud_config_with_context, ConfigError, ResolveContext,
};
pub use credential_policy::{
    bind_credential_intent_policy, policy_allows_impersonation, BoundCredentialIntent,
    ENV_CREDENTIAL_POLICY_JSON, ENV_CREDENTIAL_POLICY_PATH, ENV_CREDENTIAL_POLICY_PROFILE,
};
pub use env_requirements::{
    aws_github_actions_env_stub, azure_github_actions_env_stub, cloud_env_matrix,
    collect_missing_requirements, detect_cloud_env_requirements, detect_provider_runtime,
    format_missing_requirements_message, gcp_github_actions_env, gcp_local_env, gcp_metadata_env,
    requirements_for, CloudEnvRequirements, MissingCloudEnvRequirements, CLOUD_ENV_COMMON_OPTIONAL,
};
pub use env_status::CloudEnvStatus;
pub use github_ops::GitHubCredentialOps;
pub use health_status::{evaluate_health, HealthCheckItem, HealthCheckReport};
pub use infra_bootstrap::{build_wif_bootstrap_dag, InfraBootstrapGraphOp, InfraBootstrapOps};
pub use infra_graph::render_infra_spec_dot;
pub use infra_plan_apply::{
    build_infra_apply_dag, build_infra_plan_dag, InfraApplyFilter, InfraPlanApplyGraphOp,
    InfraPlanApplyOps,
};
pub use infra_spec::{EnvironmentConfig, InfraSpec, CI_SPEC, DEV_SPEC, PROD_SPEC, TEST_SPEC};
pub use login_flow::{inspect_login_flow, LoginDiagnostics};
pub use ops::CloudOps;
pub use project_registry::{
    derive_cross_project_wif_bindings, CrossProjectWifBinding, ProjectRegistry, GUNBAI_PLATFORM,
};
pub use secret_cache::{plan_secret_fetch, SecretCacheEntry, SecretValueCache};
pub use secret_exports::{render_direnv_exports, SecretExportResult};
pub use secret_provision_graph::{
    build_secrets_provision_dag, build_secrets_provision_dag_from_spec,
    build_secrets_provision_dag_from_spec_with_filter, SecretProvisionFilter,
};
pub use secret_rotation::{check_secret_age, rotate_secret, SecretAgeCheck, SecretRotationAction};

// ---------------------------------------------------------------------------
// ConstCloudConfig: drop-in replacement for CloudEnv in tool graphs
// ---------------------------------------------------------------------------

/// Create a `CloudOps::ConstCloudConfig` that emits a pre-resolved
/// `CloudSecretConfig` as constant outputs.
///
/// This replaces the legacy `CloudEnv` env-reader. The config is resolved
/// once at graph construction time and baked into the node, eliminating
/// all environment variable reads.
///
/// # Usage in tool graphs
///
/// ```ignore
/// let cloud_config = builder.add_root_node(Node::opaque(
///     "cloud_env", ...,
///     GistGraphOp::Cloud(CloudSecretManagerGraphOp::Cloud(
///         const_cloud_config(resolved_config),
///     )),
/// ));
/// ```
pub fn const_cloud_config(config: gunbc_ir::transport::cloud::CloudSecretConfig) -> CloudOps {
    CloudOps::ConstCloudConfig { config }
}
