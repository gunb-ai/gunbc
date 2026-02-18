//! Secret provisioning DAG builder.

use crate::graph::{
    build_cloud_secret_manager_upsert_graph_from_config, CloudSecretManagerGraphOp,
};
use crate::project_spec::{ProjectSpec, SecretStatus, GUNBAI_SECRETS};
use gunbc_ir::transport::cloud::CloudRuntimeKind;
use gunbc_ir::{Dag, DagBuilder, Node};

/// Build a provisioning DAG that upserts all active secrets for one namespace.
///
/// Each active secret becomes one sub-DAG node named `provision_<secret_id>`.
/// The sub-DAG is the existing cloud secret upsert graph for that secret config.
pub fn build_secrets_provision_dag(
    namespace: &str,
    runtime: CloudRuntimeKind,
) -> Result<Dag<CloudSecretManagerGraphOp>, String> {
    build_secrets_provision_dag_from_spec(&GUNBAI_SECRETS, namespace, runtime)
}

/// Build provisioning DAG from an explicit project spec.
pub fn build_secrets_provision_dag_from_spec(
    project_spec: &'static ProjectSpec,
    namespace: &str,
    runtime: CloudRuntimeKind,
) -> Result<Dag<CloudSecretManagerGraphOp>, String> {
    let mut base_config = project_spec
        .to_cloud_secret_config(namespace, runtime)
        .ok_or_else(|| format!("unknown namespace '{}'", namespace))?;

    let mut builder: DagBuilder<CloudSecretManagerGraphOp> = DagBuilder::new();

    for secret in project_spec
        .secrets
        .iter()
        .filter(|s| s.status == SecretStatus::Active)
    {
        base_config.secret.name = secret.secret_id.to_string();
        let secret_subdag = build_cloud_secret_manager_upsert_graph_from_config(&base_config);
        let node_id = format!("provision_{}", secret.secret_id.replace('-', "_"));
        builder
            .add_root_node(Node::subdag(node_id.as_str(), secret_subdag))
            .map_err(|e| format!("failed to add {}: {}", node_id, e))?;
    }

    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn build_secrets_provision_dag_creates_nodes_for_active_secrets() {
        let dag = build_secrets_provision_dag("dev", CloudRuntimeKind::LocalDev)
            .expect("provision dag should build");
        let active_count = GUNBAI_SECRETS
            .secrets
            .iter()
            .filter(|s| s.status == SecretStatus::Active)
            .count();
        assert_eq!(dag.nodes.len(), active_count);
    }

    #[test]
    fn build_secrets_provision_dag_exposes_secret_value_entrypoints_and_versions() {
        let dag = build_secrets_provision_dag("dev", CloudRuntimeKind::LocalDev)
            .expect("provision dag should build");
        let entrypoints = detect_entrypoints(&dag);
        let boundaries = detect_boundaries(&dag);

        for secret in GUNBAI_SECRETS
            .secrets
            .iter()
            .filter(|s| s.status == SecretStatus::Active)
        {
            let node_id = format!("provision_{}", secret.secret_id.replace('-', "_"));
            assert!(
                entrypoints.is_entrypoint_node(&node_id.clone().into()),
                "provision subdag '{}' should expose secret_value entrypoint",
                node_id
            );
            assert!(
                boundaries.is_boundary_node(&node_id.clone().into()),
                "provision subdag '{}' should expose version boundary",
                node_id
            );
        }
    }
}
