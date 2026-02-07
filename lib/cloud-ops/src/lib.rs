//! Provider-neutral cloud credential graphs.
//!
//! This crate stitches provider-specific secret manager DAGs together behind a
//! stable interface so callers can switch providers without reworking DAG shapes.

mod env;
mod graph;
mod github_credential_graph;
mod ops;

pub use graph::{
    build_cloud_secret_manager_credential_graph_from_config,
    build_cloud_secret_manager_credential_graph_gcp_github,
    build_cloud_secret_manager_credential_graph_gcp_metadata,
    build_cloud_secret_manager_credential_graph_aws_stub,
    build_cloud_secret_manager_credential_graph_azure_stub,
    CloudSecretManagerGraphOp,
};
pub use github_credential_graph::{build_github_credential_graph, GitHubCredentialGraphOp};
pub use env::CloudEnv;
pub use ops::CloudOps;
