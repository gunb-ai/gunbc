//! Stub graph for AWS Secrets Manager.

use crate::ops::AwsOps;
use gunbc_exec::DynOp;
use gunbc_ir::build::{list, optional, port};
use gunbc_ir::{BuilderError, Dag, DagBuilder, Node};

pub type AwsSecretManagerGraphOp = DynOp;

/// Placeholder DAG for AWS Secrets Manager credentials.
///
/// This keeps the interface surface for cloud providers stable while the
/// real implementation is built.
pub fn build_aws_secrets_manager_credential_graph(
) -> Result<Dag<AwsSecretManagerGraphOp>, BuilderError> {
    let mut builder: DagBuilder<AwsSecretManagerGraphOp> = DagBuilder::new();

    builder.add_root_node(Node::opaque(
        "aws_secrets_manager_stub",
        vec![
            port("config", "CloudSecretConfig"),
            port("scheme", "NonEmptyString"),
            optional("header_name", "OptionalString"),
            port("source_id", "NonEmptyString"),
            list("required_scopes", "NonEmptyString"),
            optional("lifetime_seconds", "OptionalInt"),
            optional("request_url", "Url"),
            optional("request_token", "Secret"),
        ],
        vec![port("credential", "Credential")],
        DynOp::new(AwsOps::Unsupported),
    ))?;

    Ok(builder.build())
}

/// Placeholder DAG for AWS Secrets Manager secret upsert.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "aws-secrets-upsert-stub",
    builder = "build_aws_secrets_manager_upsert_graph()",
    returns_result
)]
pub fn build_aws_secrets_manager_upsert_graph() -> Result<Dag<AwsSecretManagerGraphOp>, BuilderError>
{
    let mut builder: DagBuilder<AwsSecretManagerGraphOp> = DagBuilder::new();

    builder.add_root_node(Node::opaque(
        "aws_secrets_manager_upsert_stub",
        vec![
            port("config", "CloudSecretConfig"),
            port("secret_value", "Secret"),
        ],
        vec![port("version", "NonEmptyString")],
        DynOp::new(AwsOps::Unsupported),
    ))?;

    Ok(builder.build())
}
