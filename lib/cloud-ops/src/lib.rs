//! Provider-neutral cloud credential operations.
//!
//! This crate provides credential policy, infrastructure bootstrapping,
//! secret provisioning, and environment configuration for cloud providers.

pub mod config_loader;
pub mod config_resource;
pub mod credential_policy;
pub mod env_requirements;
mod env_status;
pub mod health_status;
pub mod infra_bootstrap;
pub mod infra_graph;
pub mod infra_plan_apply;
pub mod infra_spec;
pub mod login_flow;
pub mod project_registry;
pub mod project_spec;
pub mod secret_cache;
pub mod secret_exports;
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
pub use health_status::{evaluate_health, HealthCheckItem, HealthCheckReport};
pub use infra_bootstrap::{build_wif_bootstrap_dag, InfraBootstrapGraphOp, InfraBootstrapOps};
pub use infra_graph::render_infra_spec_dot;
pub use infra_plan_apply::{
    build_infra_plan_dag, InfraApplyFilter, InfraPlanApplyOps,
};
pub use infra_spec::{EnvironmentConfig, InfraSpec, CI_SPEC, DEV_SPEC, PROD_SPEC, TEST_SPEC};
pub use login_flow::{inspect_login_flow, LoginDiagnostics};
pub use project_registry::{
    derive_cross_project_wif_bindings, CrossProjectWifBinding, ProjectRegistry, GUNBAI_PLATFORM,
};
pub use secret_cache::{plan_secret_fetch, SecretCacheEntry, SecretValueCache};
pub use secret_exports::{render_direnv_exports, SecretExportResult};
pub use secret_rotation::{check_secret_age, rotate_secret, SecretAgeCheck, SecretRotationAction};
