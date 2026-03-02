//! Workflow planner core modules (WF1+).
//!
//! Re-exports generic workflow engine from gunbc-workflow and exposes
//! repo-specific catalog, commands, spec builders, and capabilities.

mod capabilities;
mod catalog;
mod commands;
mod spec_builders;

pub use gunbc_workflow::WorkflowAdmissionError;
pub use gunbc_workflow::{
    check_slo, default_slo_budgets, render_execution_report, top_slow_units, SloBudget, SloResult,
    SlowUnit,
};
pub use gunbc_workflow::{
    claim_handle_type_id, ClaimId, ProcessId, ProcessUnitRef, ProcessUnitRegistry, ProcessUnitSpec,
    UnitClaim,
};
pub use gunbc_workflow::{coordination_status, BlockedReason, CoordinationStatus};
pub use gunbc_workflow::{execute_workflow_plan, ExecutionSummary, UnitCommand, UnitResult};
pub use gunbc_workflow::{
    explain_plan, plan_workflow, plan_workflow_with_mode, CapabilityAction, CapabilityStatus,
    DryRunMode, NodePlan, PlanAction, PlanExplain, PlannerInputs, WorkflowPlan,
    WorkflowPlannerError,
};
pub use gunbc_workflow::{
    has_required_unit_contract, required_input_contract, required_output_contract, AggregateSpec,
    ReportSpec, WorkflowId, WorkflowOp, WorkflowSpec, WorkflowUnit, PORT_AFTER, PORT_COMMIT,
    PORT_RESULT, TYPE_WORKFLOW_RESULT,
};
pub use gunbc_workflow::{
    plan_global_workflows, GlobalExecutionVertex, GlobalWorkflowPlan, PlannerInputsByWorkflow,
    WorkflowNodeRef,
};
pub use gunbc_workflow::{
    project_execute_set, validate_projection_equivalence, ExecuteProjection, ProjectionDrift,
};
pub use gunbc_workflow::{prove_non_redundancy, InvariantViolation};
pub use gunbc_workflow::{
    validate_conflicting_claims, validate_effectful_claim_declarations, validate_required_claims,
    validate_workflow_admission,
};
pub use gunbc_workflow::{
    CanonicalKeyPayload, MaterializationDigest, MaterializationKey, MissReason, WorkIdentity,
};

pub use capabilities::{
    codegen_key, compilation_key, CodegenMissReason, CompilationMissReason, CompilationPhase,
    CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
};
pub use catalog::default_process_unit_registry;
pub use commands::{ci_unit_commands, test_all_unit_commands, workflow_unit_commands};
pub use spec_builders::{
    all_tool_workflow_names, bootstrap_workflow_spec, ci_workflow_spec, deps_workflow_spec,
    gist_diff_workflow_spec, gist_recent_workflow_spec, gist_workflow_spec, makegen_workflow_spec,
    pragma_workflow_spec, test_all_workflow_spec, tool_workflow_spec,
};
