//! Global flattening + dedup planner contracts (M17).

use std::collections::BTreeMap;
use std::path::Path;

use gunbc_ir::NodeId;

use crate::key::{MaterializationDigest, WorkIdentity};
use crate::planner::{
    plan_workflow, PlanAction, PlannerInputs, WorkflowPlan, WorkflowPlannerError,
};
use crate::process_registry::ProcessUnitRegistry;
use crate::schema::{WorkflowId, WorkflowSpec};

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
