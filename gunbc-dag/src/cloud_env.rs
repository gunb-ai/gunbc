//! Repo-facing re-exports for cloud environment requirements.

pub use gunbc_lib_cloud_ops::env_requirements::{
    aws_github_actions_env_stub, azure_github_actions_env_stub, cloud_env_matrix,
    detect_cloud_env_requirements, gcp_github_actions_env, gcp_local_env, gcp_metadata_env,
    CloudEnvRequirements, CLOUD_ENV_COMMON_OPTIONAL,
};
