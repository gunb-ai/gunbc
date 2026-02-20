//! Projection drift guards for canonical workflow semantics (M18).

use std::collections::BTreeSet;

use super::global_plan::GlobalWorkflowPlan;
use super::key::{MaterializationDigest, WorkIdentity};
use super::planner::PlanAction;

/// Projection of canonical execute semantics into wrapper surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecuteProjection {
    pub execute_set: BTreeSet<(WorkIdentity, MaterializationDigest)>,
}

/// Projection drift diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionDrift {
    MissingExecuteVertex {
        work_id: WorkIdentity,
        digest: MaterializationDigest,
    },
    UnexpectedExecuteVertex {
        work_id: WorkIdentity,
        digest: MaterializationDigest,
    },
}

/// Build projection from canonical global plan.
pub fn project_execute_set(plan: &GlobalWorkflowPlan) -> ExecuteProjection {
    let execute_set = plan
        .vertices
        .iter()
        .filter(|vertex| matches!(vertex.action, PlanAction::Execute { .. }))
        .map(|vertex| (vertex.work_id.clone(), vertex.digest.clone()))
        .collect();
    ExecuteProjection { execute_set }
}

/// Validate projected execute semantics against canonical global plan.
pub fn validate_projection_equivalence(
    plan: &GlobalWorkflowPlan,
    projected: &ExecuteProjection,
) -> Result<(), Vec<ProjectionDrift>> {
    let canonical = project_execute_set(plan);
    let mut drift = Vec::new();

    for (work_id, digest) in &canonical.execute_set {
        if !projected
            .execute_set
            .contains(&(work_id.clone(), digest.clone()))
        {
            drift.push(ProjectionDrift::MissingExecuteVertex {
                work_id: work_id.clone(),
                digest: digest.clone(),
            });
        }
    }

    for (work_id, digest) in &projected.execute_set {
        if !canonical
            .execute_set
            .contains(&(work_id.clone(), digest.clone()))
        {
            drift.push(ProjectionDrift::UnexpectedExecuteVertex {
                work_id: work_id.clone(),
                digest: digest.clone(),
            });
        }
    }

    if drift.is_empty() {
        Ok(())
    } else {
        Err(drift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::global_plan::{
        GlobalExecutionVertex, GlobalWorkflowPlan, WorkflowNodeRef,
    };
    use crate::workflow::key::MaterializationDigest;
    use crate::workflow::planner::PlanAction;
    use crate::workflow::schema::WorkflowId;
    use gunbc_ir::NodeId;

    fn sample_plan() -> GlobalWorkflowPlan {
        GlobalWorkflowPlan {
            vertices: vec![GlobalExecutionVertex {
                work_id: WorkIdentity::new("process-unit", "codegen"),
                digest: MaterializationDigest("abc".to_string()),
                action: PlanAction::Execute {
                    miss_reason: crate::workflow::MissReason::NoPriorRun,
                },
                node_refs: vec![WorkflowNodeRef {
                    workflow_id: WorkflowId::new("ci"),
                    node_id: NodeId::from("ci.codegen"),
                }],
            }],
        }
    }

    #[test]
    fn projection_matches_canonical_execute_set() {
        let plan = sample_plan();
        let projection = project_execute_set(&plan);
        assert!(validate_projection_equivalence(&plan, &projection).is_ok());
    }
}
