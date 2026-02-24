//! Infrastructure plan DAG builder.

use crate::infra_spec::InfraSpec;
use crate::project_spec::ProjectSpec;
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

#[derive(Debug, Clone)]
pub enum InfraPlanApplyOps {
    BuildPlan {
        environment: String,
        targets: Vec<String>,
    },
}

impl Executable for InfraPlanApplyOps {
    fn execute(
        &self,
        _inputs: HashMap<String, gunbc_ir::Value>,
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
        }
    }
}

/// Build a plan DAG for one environment and runtime.
pub fn build_infra_plan_dag(
    project_spec: &'static ProjectSpec,
    infra_spec: &InfraSpec,
    runtime: CloudRuntimeKind,
    filter: &InfraApplyFilter,
) -> Result<Dag<InfraPlanApplyOps>, String> {
    let targets = collect_targets(project_spec, infra_spec, runtime, filter)?;
    let mut builder: DagBuilder<InfraPlanApplyOps> = DagBuilder::new();
    builder
        .add_root_node(Node::opaque(
            "plan",
            vec![],
            vec![
                port("environment", "NonEmptyString"),
                list("planned_targets", "String"),
                port("target_count", "Int"),
            ],
            InfraPlanApplyOps::BuildPlan {
                environment: infra_spec.environment.to_string(),
                targets,
            },
        ))
        .map_err(|e| format!("failed to build plan node: {e}"))?;

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
    use crate::project_spec::GUNBAI_SECRETS;
    use gunbc_ir::detect_boundaries;

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
}
