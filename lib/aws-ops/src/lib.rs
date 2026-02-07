//! AWS OIDC + Secrets Manager ops (stub).
//!
//! This is a placeholder to keep provider-neutral modeling honest.
//! The implementation will follow the same subject-token → STS →
//! optional role chaining → Secrets Manager pattern.

mod ops;
mod graph;

pub use graph::{build_aws_secrets_manager_credential_graph, AwsSecretManagerGraphOp};
pub use ops::AwsOps;
