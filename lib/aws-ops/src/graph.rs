//! Stub graph for AWS Secrets Manager.

use crate::ops::AwsOps;
use gunbc_delegate_macros::DelegateExecutable;
use gunbc_ir::build::{list, optional, port};
use gunbc_ir::{Dag, DagBuilder, Node};

#[derive(Debug, Clone, DelegateExecutable)]
pub enum AwsSecretManagerGraphOp {
    Aws(AwsOps),
}

/// Placeholder DAG for AWS Secrets Manager credentials.
///
/// This keeps the interface surface for cloud providers stable while the
/// real implementation is built.
pub fn build_aws_secrets_manager_credential_graph() -> Dag<AwsSecretManagerGraphOp> {
    let mut builder: DagBuilder<AwsSecretManagerGraphOp> = DagBuilder::new();

    builder
        .add_root_node(Node::opaque(
            "aws_secrets_manager_stub",
            vec![
                port("config", "CloudSecretConfig"),
                port("scheme", "String"),
                optional("header_name", "OptionalString"),
                port("source_id", "String"),
                list("required_scopes", "String"),
                optional("lifetime_seconds", "OptionalInt"),
                optional("request_url", "OptionalString"),
                optional("request_token", "OptionalString"),
            ],
            vec![port("credential", "Credential")],
            AwsSecretManagerGraphOp::Aws(AwsOps::Unsupported),
        ))
        .expect("aws_secrets_manager_stub node");

    builder.build()
}

/// Placeholder DAG for AWS Secrets Manager secret upsert.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "aws-secrets-upsert-stub",
    builder = "build_aws_secrets_manager_upsert_graph()"
)]
pub fn build_aws_secrets_manager_upsert_graph() -> Dag<AwsSecretManagerGraphOp> {
    let mut builder: DagBuilder<AwsSecretManagerGraphOp> = DagBuilder::new();

    builder
        .add_root_node(Node::opaque(
            "aws_secrets_manager_upsert_stub",
            vec![
                port("config", "CloudSecretConfig"),
                port("secret_value", "Secret"),
            ],
            vec![port("version", "String")],
            AwsSecretManagerGraphOp::Aws(AwsOps::Unsupported),
        ))
        .expect("aws_secrets_manager_upsert_stub node");

    builder.build()
}
