//! Generic workflow planner and executor engine.

pub mod admission;
pub mod coordination;
pub mod errors;
pub mod executor;
pub mod global_plan;
pub mod key;
pub mod planner;
pub mod process_registry;
pub mod projection;
pub mod proof;
pub mod schema;
pub mod slo;

pub use admission::{
    validate_conflicting_claims, validate_effectful_claim_declarations, validate_required_claims,
    validate_workflow_admission,
};
pub use coordination::{coordination_status, BlockedReason, CoordinationStatus};
pub use errors::WorkflowAdmissionError;
pub use executor::{execute_workflow_plan, ExecutionSummary, UnitCommand, UnitResult};
pub use global_plan::{
    plan_global_workflows, GlobalExecutionVertex, GlobalWorkflowPlan, PlannerInputsByWorkflow,
    WorkflowNodeRef,
};
pub use key::{
    CanonicalKeyPayload, MaterializationDigest, MaterializationKey, MissReason, WorkIdentity,
};
pub use planner::{
    explain_plan, plan_workflow, plan_workflow_with_mode, CapabilityAction, CapabilityStatus,
    DryRunMode, NodePlan, PlanAction, PlanExplain, PlannerInputs, WorkflowPlan,
    WorkflowPlannerError,
};
pub use process_registry::{
    claim_handle_type_id, ClaimId, ProcessId, ProcessUnitRef, ProcessUnitRegistry, ProcessUnitSpec,
    UnitClaim,
};
pub use projection::{
    project_execute_set, validate_projection_equivalence, ExecuteProjection, ProjectionDrift,
};
pub use proof::{prove_non_redundancy, InvariantViolation};
pub use schema::{
    has_required_unit_contract, required_input_contract, required_output_contract, AggregateSpec,
    ReportSpec, WorkflowId, WorkflowOp, WorkflowSpec, WorkflowUnit, PORT_AFTER, PORT_COMMIT,
    PORT_RESULT, TYPE_WORKFLOW_RESULT,
};
pub use slo::{
    check_slo, default_slo_budgets, render_execution_report, top_slow_units, SloBudget, SloResult,
    SlowUnit,
};
