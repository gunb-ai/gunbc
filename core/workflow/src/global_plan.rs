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
mod tests {
    use super::*;
    use crate::process_registry::{ProcessUnitRef, ProcessUnitSpec, UnitClaim};
    use crate::schema::{required_input_contract, required_output_contract, WorkflowOp, WorkflowUnit};
    use gunbc_ir::{Dag, Node};

    #[test]
    fn cross_workflow_equivalent_units_are_deduped() {
        let mut ci_dag: Dag<WorkflowUnit> = Dag::new();
        ci_dag.add_node(Node::opaque(
            "ci.codegen",
            required_input_contract(),
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "ci",
                "ci.codegen",
            ))),
        ));
        let mut test_all_dag: Dag<WorkflowUnit> = Dag::new();
        test_all_dag.add_node(Node::opaque(
            "test_all.codegen",
            required_input_contract(),
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::InvokeProcessUnit(ProcessUnitRef::new(
                "test_all",
                "test_all.codegen",
            ))),
        ));

        let specs = vec![
            WorkflowSpec::new("ci", ci_dag, 1),
            WorkflowSpec::new("test-all", test_all_dag, 1),
        ];

        let mut registry = ProcessUnitRegistry::new();
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("ci", "ci.codegen"),
            1,
            vec![UnitClaim::read("tool:cargo")],
        ));
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("test_all", "test_all.codegen"),
            1,
            vec![UnitClaim::read("tool:cargo")],
        ));

        let workspace_root = std::path::Path::new(".");
        let global = plan_global_workflows(
            &specs,
            &registry,
            &PlannerInputsByWorkflow::new(),
            workspace_root,
        )
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
    }
}
