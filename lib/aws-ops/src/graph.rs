//! Stub graph for AWS Secrets Manager.

use crate::ops::AwsOps;
use gunbc_exec::{ExecError, Executable};
use gunbc_ir::build::{optional, port};
use gunbc_ir::{Dag, DagBuilder, Node, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AwsSecretManagerGraphOp {
    Aws(AwsOps),
}

impl Executable for AwsSecretManagerGraphOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            AwsSecretManagerGraphOp::Aws(op) => op.execute(inputs),
        }
    }
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
pub fn build_aws_secrets_manager_upsert_graph() -> Dag<AwsSecretManagerGraphOp> {
    let mut builder: DagBuilder<AwsSecretManagerGraphOp> = DagBuilder::new();

    builder
        .add_root_node(Node::opaque(
            "aws_secrets_manager_upsert_stub",
            vec![port("config", "CloudSecretConfig"), port("secret_value", "Secret")],
            vec![port("version", "String")],
            AwsSecretManagerGraphOp::Aws(AwsOps::Unsupported),
        ))
        .expect("aws_secrets_manager_upsert_stub node");

    builder.build()
}
