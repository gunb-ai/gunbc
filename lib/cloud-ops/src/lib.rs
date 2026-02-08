//! Provider-neutral cloud credential graphs.
//!
//! This crate stitches provider-specific secret manager DAGs together behind a
//! stable interface so callers can switch providers without reworking DAG shapes.

mod env;
pub mod env_requirements;
mod env_status;
mod github_credential_graph;
mod graph;
mod ops;

pub use env::CloudEnv;
pub use env_requirements::{
    aws_github_actions_env_stub, azure_github_actions_env_stub, cloud_env_matrix,
    collect_missing_requirements, detect_cloud_env_requirements, detect_provider_runtime,
    format_missing_requirements_message, gcp_github_actions_env, gcp_local_env, gcp_metadata_env,
    requirements_for, CloudEnvRequirements, MissingCloudEnvRequirements, CLOUD_ENV_COMMON_OPTIONAL,
};
pub use env_status::CloudEnvStatus;
pub use github_credential_graph::{build_github_credential_graph, GitHubCredentialGraphOp};
pub use graph::{
    build_cloud_credential_graph_for_runtime, build_cloud_secret_manager_credential_graph_aws_stub,
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
