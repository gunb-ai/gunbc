use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

pub type AwsSecretManagerGraphOp = DynOp;

fn dropped_provider_error(surface: &str) -> BuilderError {
    BuilderError::InternalInvariant(format!(
        "AWS provider stack is dropped in this branch; `{surface}` is intentionally unsupported"
    ))
}

pub fn build_aws_secrets_manager_credential_graph(
) -> Result<Dag<AwsSecretManagerGraphOp>, BuilderError> {
    Err(dropped_provider_error(
        "build_aws_secrets_manager_credential_graph",
    ))
}

pub fn build_aws_secrets_manager_upsert_graph() -> Result<Dag<AwsSecretManagerGraphOp>, BuilderError>
{
    Err(dropped_provider_error(
        "build_aws_secrets_manager_upsert_graph",
    ))
}
