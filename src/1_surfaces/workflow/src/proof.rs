//! Non-redundancy invariant harness (M19).

use std::collections::BTreeSet;

use crate::global_plan::GlobalWorkflowPlan;
use crate::key::{MaterializationDigest, WorkIdentity};

/// Invariant failure diagnostics for workflow non-redundancy checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantViolation {
    DuplicateVertexKey {
        work_id: WorkIdentity,
        digest: MaterializationDigest,
    },
    EmptyVertexSources {
        work_id: WorkIdentity,
        digest: MaterializationDigest,
    },
}

/// Prove core non-redundancy invariants over a global workflow plan.
pub fn prove_non_redundancy(plan: &GlobalWorkflowPlan) -> Result<(), Vec<InvariantViolation>> {
    let mut violations = Vec::new();
    let mut seen = BTreeSet::new();

    for vertex in &plan.vertices {
        let key = (vertex.work_id.clone(), vertex.digest.clone());
        if !seen.insert(key.clone()) {
            violations.push(InvariantViolation::DuplicateVertexKey {
                work_id: key.0,
                digest: key.1,
            });
        }
        if vertex.node_refs.is_empty() {
            violations.push(InvariantViolation::EmptyVertexSources {
                work_id: vertex.work_id.clone(),
                digest: vertex.digest.clone(),
            });
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_plan::GlobalExecutionVertex;
    use crate::planner::PlanAction;
    use crate::schema::WorkflowId;
    use gunbc_ir::NodeId;

    fn sample_vertex() -> GlobalExecutionVertex {
        GlobalExecutionVertex {
            work_id: WorkIdentity::new("process-unit", "codegen"),
            digest: MaterializationDigest("abc".to_string()),
            action: PlanAction::Execute {
                miss_reason: crate::MissReason::NoPriorRun,
            },
            node_refs: vec![crate::global_plan::WorkflowNodeRef {
                workflow_id: WorkflowId::new("ci"),
                node_id: NodeId::from("ci.codegen"),
            }],
        }
    }

    #[test]
    fn prove_non_redundancy_accepts_valid_plan() {
        let plan = GlobalWorkflowPlan {
            vertices: vec![sample_vertex()],
        };
        assert!(prove_non_redundancy(&plan).is_ok());
    }

    #[test]
    fn prove_non_redundancy_rejects_duplicate_vertex_keys() {
        let vertex = sample_vertex();
        let plan = GlobalWorkflowPlan {
            vertices: vec![vertex.clone(), vertex],
        };
        let violations = prove_non_redundancy(&plan).expect_err("duplicate keys should fail");
        assert!(violations
            .iter()
            .any(|violation| matches!(violation, InvariantViolation::DuplicateVertexKey { .. })));
    }
}
