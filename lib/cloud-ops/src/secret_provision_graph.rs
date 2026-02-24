//! Secret provisioning DAG builder.

use crate::project_spec::{ProjectSpec, GUNBAI_SECRETS};
use gunbc_exec::DynOp;
use gunbc_ir::transport::cloud::CloudRuntimeKind;
use gunbc_ir::{Dag, DagBuilder};

type CloudSecretManagerGraphOp = DynOp;

/// Filters for secret provisioning target selection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecretProvisionFilter {
    /// When non-empty, only these secret IDs are provisioned.
    pub include_secret_ids: Vec<String>,
    /// Secret IDs to exclude from provisioning.
    pub exclude_secret_ids: Vec<String>,
}

/// Build a provisioning DAG that upserts all active secrets for one namespace.
///
/// Returns a placeholder empty DAG. Secret provisioning will be re-implemented
/// via DSL credential_chain pattern.
pub fn build_secrets_provision_dag(
    namespace: &str,
    runtime: CloudRuntimeKind,
) -> Result<Dag<CloudSecretManagerGraphOp>, String> {
    build_secrets_provision_dag_from_spec_with_filter(
        &GUNBAI_SECRETS,
        namespace,
        runtime,
        &SecretProvisionFilter::default(),
    )
}

/// Build provisioning DAG from an explicit project spec.
pub fn build_secrets_provision_dag_from_spec(
    project_spec: &'static ProjectSpec,
    namespace: &str,
    runtime: CloudRuntimeKind,
) -> Result<Dag<CloudSecretManagerGraphOp>, String> {
    build_secrets_provision_dag_from_spec_with_filter(
        project_spec,
        namespace,
        runtime,
        &SecretProvisionFilter::default(),
    )
}

/// Build provisioning DAG from an explicit project spec and filter.
///
/// Currently returns an empty DAG; the legacy graph builders have been deleted.
/// Secret provisioning will be re-implemented via DSL credential_chain pattern.
pub fn build_secrets_provision_dag_from_spec_with_filter(
    project_spec: &'static ProjectSpec,
    namespace: &str,
    runtime: CloudRuntimeKind,
    _filter: &SecretProvisionFilter,
) -> Result<Dag<CloudSecretManagerGraphOp>, String> {
    project_spec
        .to_cloud_secret_config(namespace, runtime)
        .ok_or_else(|| format!("unknown namespace '{}'", namespace))?;

    let builder: DagBuilder<CloudSecretManagerGraphOp> = DagBuilder::new();
    Ok(builder.build())
}
