//! Provider-neutral cloud credential graphs.
//!
//! This crate stitches provider-specific secret manager DAGs together behind a
//! stable interface so callers can switch providers without reworking DAG shapes.

pub mod config_loader;
pub mod config_resource;
pub mod credential_policy;
pub mod env_requirements;
mod env_status;
mod github_credential_graph;
mod graph;
mod ops;
pub mod project_spec;

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
pub use github_credential_graph::{build_github_credential_graph, GitHubCredentialGraphOp};
pub use graph::{
    build_cloud_secret_manager_credential_graph_aws_stub,
    build_cloud_secret_manager_credential_graph_azure_stub,
    build_cloud_secret_manager_credential_graph_from_config,
    build_cloud_secret_manager_credential_graph_gcp_github,
    build_cloud_secret_manager_credential_graph_gcp_local,
    build_cloud_secret_manager_credential_graph_gcp_metadata,
    build_cloud_secret_manager_upsert_graph_aws_stub,
    build_cloud_secret_manager_upsert_graph_azure_stub,
    build_cloud_secret_manager_upsert_graph_from_config,
    build_cloud_secret_manager_upsert_graph_gcp_github,
    build_cloud_secret_manager_upsert_graph_gcp_local,
    build_cloud_secret_manager_upsert_graph_gcp_metadata, CloudSecretManagerGraphOp,
};
pub use ops::CloudOps;

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
