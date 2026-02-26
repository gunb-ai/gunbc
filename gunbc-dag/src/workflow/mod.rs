//! Workflow planner core modules (WF1+).

mod catalog;

pub mod admission;
pub mod capabilities;
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
pub mod spec_builders;
pub mod unit_commands;

pub use admission::{
    validate_conflicting_claims, validate_effectful_claim_declarations, validate_required_claims,
    validate_workflow_admission,
};
pub use capabilities::{
    codegen_key, compilation_key, CodegenMissReason, CompilationMissReason, CompilationPhase,
    CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
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
    claim_handle_type_id, default_process_unit_registry, ClaimId, ProcessId, ProcessUnitRef,
    ProcessUnitRegistry, ProcessUnitSpec, UnitClaim,
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
pub use spec_builders::{
    all_tool_workflow_names, bootstrap_workflow_spec, ci_workflow_spec, deps_workflow_spec,
    gist_diff_workflow_spec, gist_recent_workflow_spec, gist_workflow_spec, makegen_workflow_spec,
    pragma_workflow_spec, test_all_workflow_spec, tool_workflow_spec,
};
pub use unit_commands::{ci_unit_commands, test_all_unit_commands, workflow_unit_commands};
