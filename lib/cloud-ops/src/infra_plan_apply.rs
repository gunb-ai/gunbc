//! Infrastructure plan/apply DAG builders.

use crate::graph::{
    build_cloud_secret_manager_upsert_graph_from_config, CloudSecretManagerGraphOp,
};
use crate::infra_spec::InfraSpec;
use crate::project_spec::{ProjectSpec, SecretStatus, GUNBAI_SECRETS};
use gunbc_delegate_macros::DelegateExecutable;
use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::build::{list, port};
use gunbc_ir::transport::cloud::CloudRuntimeKind;
use gunbc_ir::{Dag, DagBuilder, Node};
use std::collections::HashMap;

/// Filtering options for plan/apply DAG generation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InfraApplyFilter {
    pub target: Vec<String>,
    pub skip: Vec<String>,
}

impl InfraApplyFilter {
    fn allows(&self, target: &str) -> bool {
        let targeted = if self.target.is_empty() {
            true
        } else {
            self.target.iter().any(|t| t == target)
        };
        targeted && !self.skip.iter().any(|s| s == target)
    }
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

#[derive(Debug, Clone, DelegateExecutable)]
pub enum InfraPlanApplyGraphOp {
    Infra(InfraPlanApplyOps),
    Cloud(CloudSecretManagerGraphOp),
}

#[derive(Debug, Clone)]
pub enum InfraPlanApplyOps {
    BuildPlan {
        environment: String,
        targets: Vec<String>,
    },
    ReconcileRuntimeDependencies {
        environment: String,
        targets: Vec<String>,
    },
    SummarizeApply {
        environment: String,
    },
}

impl Executable for InfraPlanApplyOps {
    fn execute(
        &self,
        inputs: HashMap<String, gunbc_ir::Value>,
    ) -> Result<HashMap<String, gunbc_ir::Value>, ExecError> {
        match self {
            InfraPlanApplyOps::BuildPlan {
                environment,
                targets,
            } => OutputMap::new()
                .str("environment", environment)
                .str_list("planned_targets", targets.clone())
                .int("target_count", targets.len() as i64)
                .ok(),
            InfraPlanApplyOps::ReconcileRuntimeDependencies {
                environment,
                targets,
            } => OutputMap::new()
                .str("environment", environment)
                .str_list("reconciled_targets", targets.clone())
                .int("reconciled_count", targets.len() as i64)
                .ok(),
            InfraPlanApplyOps::SummarizeApply { environment } => {
                let target_count = inputs
                    .get("target_count")
                    .and_then(|v| v.as_int())
                    .ok_or_else(|| ExecError::new("missing or invalid 'target_count' input"))?;
                let _reconciled_count = inputs
                    .get("reconciled_count")
                    .and_then(|v| v.as_int())
                    .unwrap_or(0);
                OutputMap::new()
                    .str("environment", environment)
                    .int("applied_count", target_count)
                    .str(
                        "report",
                        format!(
                            "Applied {} infrastructure targets for {}",
                            target_count, environment
                        ),
                    )
                    .ok()
            }
        }
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

/// Build a plan DAG for one environment and runtime.
pub fn build_infra_plan_dag(
    project_spec: &'static ProjectSpec,
    infra_spec: &InfraSpec,
    runtime: CloudRuntimeKind,
    filter: &InfraApplyFilter,
) -> Result<Dag<InfraPlanApplyGraphOp>, String> {
    let targets = collect_targets(project_spec, infra_spec, runtime, filter)?;
    let mut builder: DagBuilder<InfraPlanApplyGraphOp> = DagBuilder::new();
    builder
        .add_root_node(Node::opaque(
            "plan",
            vec![],
            vec![
                port("environment", "NonEmptyString"),
                list("planned_targets", "NonEmptyString"),
                port("target_count", "Int"),
            ],
            InfraPlanApplyGraphOp::Infra(InfraPlanApplyOps::BuildPlan {
                environment: infra_spec.environment.to_string(),
                targets,
            }),
        ))
        .map_err(|e| format!("failed to build plan node: {e}"))?;

    Ok(builder.build())
}

/// Build an apply DAG (plan + provisioning execution).
pub fn build_infra_apply_dag(
    project_spec: &'static ProjectSpec,
    infra_spec: &InfraSpec,
    runtime: CloudRuntimeKind,
    filter: &InfraApplyFilter,
) -> Result<Dag<InfraPlanApplyGraphOp>, String> {
    let targets = collect_targets(project_spec, infra_spec, runtime, filter)?;
    let runtime_targets: Vec<String> = targets
        .iter()
        .filter(|target| !target.starts_with("secret:"))
        .cloned()
        .collect();
    let provision_filter = SecretProvisionFilter {
        include_secret_ids: targets
            .iter()
            .filter_map(|t| t.strip_prefix("secret:").map(|s| s.to_string()))
            .collect(),
        exclude_secret_ids: Vec::new(),
    };
    let provision = build_secrets_provision_dag_from_spec_with_filter(
        project_spec,
        infra_spec.environment,
        runtime,
        &provision_filter,
    )?;

    let mut builder: DagBuilder<InfraPlanApplyGraphOp> = DagBuilder::new();
    let plan = builder
        .add_root_node(Node::opaque(
            "plan",
            vec![],
            vec![
                port("environment", "NonEmptyString"),
                list("planned_targets", "NonEmptyString"),
                port("target_count", "Int"),
            ],
            InfraPlanApplyGraphOp::Infra(InfraPlanApplyOps::BuildPlan {
                environment: infra_spec.environment.to_string(),
                targets,
            }),
        ))
        .map_err(|e| format!("failed to build plan node: {e}"))?;

    let runtime_reconcile = builder
        .add_node_after(
            Node::opaque(
                "runtime_reconcile",
                vec![],
                vec![
                    port("environment", "NonEmptyString"),
                    list("reconciled_targets", "NonEmptyString"),
                    port("reconciled_count", "Int"),
                ],
                InfraPlanApplyGraphOp::Infra(InfraPlanApplyOps::ReconcileRuntimeDependencies {
                    environment: infra_spec.environment.to_string(),
                    targets: runtime_targets,
                }),
            ),
            &plan,
        )
        .map_err(|e| format!("failed to build runtime_reconcile node: {e}"))?;

    let provision_subdag = provision.map_ops(&mut InfraPlanApplyGraphOp::Cloud);
    let provision_node = builder
        .add_node_after(
            Node::subdag("provision", provision_subdag),
            &runtime_reconcile,
        )
        .map_err(|e| format!("failed to build provision node: {e}"))?;

    let summary = builder
        .add_node_after(
            Node::opaque(
                "apply_summary",
                vec![port("target_count", "Int"), port("reconciled_count", "Int")],
                vec![
                    port("environment", "NonEmptyString"),
                    port("applied_count", "Int"),
                    port("report", "NonEmptyString"),
                ],
                InfraPlanApplyGraphOp::Infra(InfraPlanApplyOps::SummarizeApply {
                    environment: infra_spec.environment.to_string(),
                }),
            ),
            &provision_node,
        )
        .map_err(|e| format!("failed to build apply_summary node: {e}"))?;

    builder
        .add_edge(plan.out("target_count"), summary.in_port("target_count"))
        .map_err(|e| format!("failed to wire apply summary: {e}"))?;
    builder
        .add_edge(
            runtime_reconcile.out("reconciled_count"),
            summary.in_port("reconciled_count"),
        )
        .map_err(|e| format!("failed to wire runtime reconcile summary input: {e}"))?;

    Ok(builder.build())
}

fn collect_targets(
    project_spec: &'static ProjectSpec,
    infra_spec: &InfraSpec,
    runtime: CloudRuntimeKind,
    filter: &InfraApplyFilter,
) -> Result<Vec<String>, String> {
    project_spec
        .to_cloud_secret_config(infra_spec.environment, runtime)
        .ok_or_else(|| format!("unknown environment '{}'", infra_spec.environment))?;

    let mut targets = Vec::new();
    for secret in project_spec
        .active_secrets()
        .map(|secret| format!("secret:{}", secret.secret_id))
    {
        if filter.allows(&secret) {
            targets.push(secret);
        }
    }
    for service_account in infra_spec
        .service_accounts
        .iter()
        .map(|service_account| format!("service-account:{}", service_account.name))
    {
        if filter.allows(&service_account) {
            targets.push(service_account);
        }
    }
    let wif_target = format!(
        "wif:{}:{}",
        infra_spec.wif.pool_id, infra_spec.wif.provider_id
    );
    if filter.allows(&wif_target) {
        targets.push(wif_target);
    }
    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra_spec::DEV_SPEC;
    use gunbc_ir::{detect_boundaries, detect_entrypoints};

    #[test]
    fn build_infra_plan_dag_reports_filtered_targets() {
        let dag = build_infra_plan_dag(
            &GUNBAI_SECRETS,
            &DEV_SPEC,
            CloudRuntimeKind::LocalDev,
            &InfraApplyFilter {
                target: vec!["secret:github-token".to_string()],
                skip: Vec::new(),
            },
        )
        .expect("plan dag should build");

        assert!(dag.get_node(&"plan".into()).is_some());
        let boundaries = detect_boundaries(&dag);
        assert!(boundaries.is_boundary_node(&"plan".into()));
    }

    #[test]
    fn build_infra_apply_dag_contains_plan_provision_and_summary() {
        let dag = build_infra_apply_dag(
            &GUNBAI_SECRETS,
            &DEV_SPEC,
            CloudRuntimeKind::LocalDev,
            &InfraApplyFilter::default(),
        )
        .expect("apply dag should build");
        assert!(dag.get_node(&"plan".into()).is_some());
        assert!(dag.get_node(&"provision".into()).is_some());
        assert!(dag.get_node(&"runtime_reconcile".into()).is_some());
        assert!(dag.get_node(&"apply_summary".into()).is_some());
        let entrypoints = detect_entrypoints(&dag);
        assert!(entrypoints.is_entrypoint_node(&"provision".into()));
    }

    #[test]
    fn infra_apply_filter_supports_target_and_skip() {
        let filter = InfraApplyFilter {
            target: vec!["secret:github-token".to_string()],
            skip: vec!["secret:github-token".to_string()],
        };
        assert!(!filter.allows("secret:github-token"));
    }

    #[test]
    fn collect_targets_includes_runtime_dependencies() {
        let targets = collect_targets(
            &GUNBAI_SECRETS,
            &DEV_SPEC,
            CloudRuntimeKind::LocalDev,
            &InfraApplyFilter::default(),
        )
        .expect("collect_targets should succeed");
        assert!(
            targets.iter().any(|target| target.starts_with("secret:")),
            "targets should include secret dependencies"
        );
        assert!(
            targets
                .iter()
                .any(|target| target.starts_with("service-account:")),
            "targets should include service-account dependencies"
        );
        assert!(
            targets.iter().any(|target| target.starts_with("wif:")),
            "targets should include WIF dependency"
        );
    }

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

    #[test]
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
