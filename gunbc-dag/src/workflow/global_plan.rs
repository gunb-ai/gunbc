//! Global flattening + dedup planner contracts (M17).

use std::collections::BTreeMap;
use std::path::Path;

use gunbc_ir::NodeId;

use super::key::{MaterializationDigest, WorkIdentity};
use super::planner::{
    plan_workflow, PlanAction, PlannerInputs, WorkflowPlan, WorkflowPlannerError,
};
use super::process_registry::ProcessUnitRegistry;
use super::schema::{WorkflowId, WorkflowSpec};

/// Workflow+node reference attached to a global dedup vertex.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkflowNodeRef {
    pub workflow_id: WorkflowId,
    pub node_id: NodeId,
}

/// Global deduplicated execution vertex.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalExecutionVertex {
    pub work_id: WorkIdentity,
    pub digest: MaterializationDigest,
    pub action: PlanAction,
    pub node_refs: Vec<WorkflowNodeRef>,
}

/// Flattened global workflow plan across multiple entrypoint workflows.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalWorkflowPlan {
    pub vertices: Vec<GlobalExecutionVertex>,
}

/// Per-workflow planner inputs used by global planner.
pub type PlannerInputsByWorkflow = BTreeMap<WorkflowId, PlannerInputs>;

/// Flatten and deduplicate workflow plans by `(WorkIdentity, MaterializationDigest)`.
pub fn plan_global_workflows(
    specs: &[WorkflowSpec],
    registry: &ProcessUnitRegistry,
    planner_inputs: &PlannerInputsByWorkflow,
    workspace_root: &Path,
) -> Result<GlobalWorkflowPlan, WorkflowPlannerError> {
    let mut grouped: BTreeMap<(WorkIdentity, MaterializationDigest), GlobalExecutionVertex> =
        BTreeMap::new();

    for spec in specs {
        let inputs = planner_inputs.get(&spec.id).cloned().unwrap_or_default();
        let plan = plan_workflow(spec, registry, &inputs, workspace_root)?;
        merge_workflow_plan(spec, &plan, &mut grouped);
    }

    let mut vertices = grouped.into_values().collect::<Vec<_>>();
    vertices.sort_by(|left, right| {
        left.work_id
            .cmp(&right.work_id)
            .then(left.digest.cmp(&right.digest))
    });
    Ok(GlobalWorkflowPlan { vertices })
}

fn merge_workflow_plan(
    workflow_spec: &WorkflowSpec,
    plan: &WorkflowPlan,
    grouped: &mut BTreeMap<(WorkIdentity, MaterializationDigest), GlobalExecutionVertex>,
) {
    for node in &plan.nodes {
        let key = (node.work_id.clone(), node.key.digest.clone());
        let node_ref = WorkflowNodeRef {
            workflow_id: workflow_spec.id.clone(),
            node_id: node.node_id.clone(),
        };
        grouped
            .entry(key.clone())
            .and_modify(|existing| {
                existing.node_refs.push(node_ref.clone());
                existing.node_refs.sort();
            })
            .or_insert_with(|| GlobalExecutionVertex {
                work_id: key.0,
                digest: key.1,
                action: node.action.clone(),
                node_refs: vec![node_ref],
            });
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::workflow::process_registry::default_process_unit_registry;
    use crate::workflow::spec_builders::{
        ci_workflow_spec, gist_snapshot_workflow_spec, test_all_workflow_spec,
    };

    fn temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "gunbc-global-plan-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    #[test]
    fn cross_workflow_equivalent_units_are_deduped() {
        let root = temp_root();
        let specs = vec![
            ci_workflow_spec().expect("ci spec"),
            test_all_workflow_spec().expect("test-all spec"),
        ];
        let registry = default_process_unit_registry();
        let global =
            plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
                .expect("global plan");

        let codegen = global
            .vertices
            .iter()
            .find(|vertex| vertex.work_id.unit_id == NodeId::from("codegen"))
            .expect("expected canonical codegen vertex");
        assert_eq!(codegen.node_refs.len(), 2);
        assert!(codegen
            .node_refs
            .iter()
            .any(|reference| reference.workflow_id == WorkflowId::new("ci")));
        assert!(codegen
            .node_refs
            .iter()
            .any(|reference| reference.workflow_id == WorkflowId::new("test-all")));
        let _ = std::fs::remove_dir_all(root);
    }

    /// WF14/WF15: compilation and codegen capabilities shared between
    /// gist-snapshot and CI workflows via global dedup.
    #[test]
    fn universal_capabilities_deduped_across_ci_and_gist() {
        let root = temp_root();
        let specs = vec![
            ci_workflow_spec().expect("ci spec"),
            gist_snapshot_workflow_spec().expect("gist-snapshot spec"),
        ];
        let registry = default_process_unit_registry();
        let global =
            plan_global_workflows(&specs, &registry, &PlannerInputsByWorkflow::new(), &root)
                .expect("global plan");

        // compilation_ensure should appear as a shared vertex.
        let compilation = global
            .vertices
            .iter()
            .find(|vertex| vertex.work_id.unit_id == NodeId::from("compilation_ensure"))
            .expect("expected canonical compilation_ensure vertex");
        assert!(
            compilation
                .node_refs
                .iter()
                .any(|r| r.workflow_id == WorkflowId::new("gist-snapshot")),
            "compilation_ensure should reference gist-snapshot workflow"
        );

        // codegen_ensure should appear as a shared vertex.
        let codegen_ensure = global
            .vertices
            .iter()
            .find(|vertex| vertex.work_id.unit_id == NodeId::from("codegen_ensure"))
            .expect("expected canonical codegen_ensure vertex");
        assert!(
            codegen_ensure
                .node_refs
                .iter()
                .any(|r| r.workflow_id == WorkflowId::new("gist-snapshot")),
            "codegen_ensure should reference gist-snapshot workflow"
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
