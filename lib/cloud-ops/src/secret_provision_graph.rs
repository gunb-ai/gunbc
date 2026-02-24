//! Secret provisioning DAG builder.

use crate::project_spec::{ProjectSpec, SecretStatus, GUNBAI_SECRETS};
use gunbc_exec::DynOp;
use gunbc_ir::transport::cloud::{CloudRuntimeKind, CloudSecretConfig};
use gunbc_ir::{BuilderError, Dag, DagBuilder, Node};

type CloudSecretManagerGraphOp = DynOp;

fn build_cloud_secret_manager_upsert_graph_from_config(
    _config: &CloudSecretConfig,
) -> Result<Dag<CloudSecretManagerGraphOp>, BuilderError> {
    todo!("legacy graph builders deleted; replace with DSL credential_chain pattern")
}

/// Filters for secret provisioning target selection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecretProvisionFilter {
    /// When non-empty, only these secret IDs are provisioned.
    pub include_secret_ids: Vec<String>,
    /// Secret IDs to exclude from provisioning.
    pub exclude_secret_ids: Vec<String>,
}

impl SecretProvisionFilter {
    fn includes(&self, secret_id: &str) -> bool {
        let include_match = if self.include_secret_ids.is_empty() {
            true
        } else {
            self.include_secret_ids
                .iter()
                .any(|id| id.as_str() == secret_id)
        };
        include_match
            && !self
                .exclude_secret_ids
                .iter()
                .any(|id| id.as_str() == secret_id)
    }
}

/// Build a provisioning DAG that upserts all active secrets for one namespace.
///
/// Each active secret becomes one sub-DAG node named `provision_<secret_id>`.
/// The sub-DAG is the existing cloud secret upsert graph for that secret config.
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
pub fn build_secrets_provision_dag_from_spec_with_filter(
    project_spec: &'static ProjectSpec,
    namespace: &str,
    runtime: CloudRuntimeKind,
    filter: &SecretProvisionFilter,
) -> Result<Dag<CloudSecretManagerGraphOp>, String> {
    let mut base_config = project_spec
        .to_cloud_secret_config(namespace, runtime)
        .ok_or_else(|| format!("unknown namespace '{}'", namespace))?;

    let mut builder: DagBuilder<CloudSecretManagerGraphOp> = DagBuilder::new();

    for secret in project_spec
        .secrets
        .iter()
        .filter(|s| s.status == SecretStatus::Active && filter.includes(s.secret_id))
    {
        base_config.secret.name = secret.secret_id.to_string();
        let secret_subdag = build_cloud_secret_manager_upsert_graph_from_config(&base_config)
            .map_err(|e| format!("failed to build upsert graph: {}", e))?;
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
    #[ignore = "legacy graph builders deleted"]
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
    #[ignore = "legacy graph builders deleted"]
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

    #[test]
    #[ignore = "legacy graph builders deleted"]
    fn build_secrets_provision_dag_filter_respects_include_and_exclude() {
        let filter = SecretProvisionFilter {
            include_secret_ids: vec!["github-token".to_string()],
            exclude_secret_ids: vec!["other".to_string()],
        };
        let dag = build_secrets_provision_dag_from_spec_with_filter(
            &GUNBAI_SECRETS,
            "dev",
            CloudRuntimeKind::LocalDev,
            &filter,
        )
        .expect("filtered provision dag should build");

        let node_ids = dag
            .nodes
            .iter()
            .map(|n| n.id.0.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert!(node_ids.contains("provision_github_token"));
        assert_eq!(node_ids.len(), 1);
    }
}
