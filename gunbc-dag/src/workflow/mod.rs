//! Workflow planner core modules (WF1+).

pub mod admission;
pub mod coordination;
pub mod errors;
pub mod executor;
pub mod global_plan;
pub mod key;
pub mod ledger;
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
pub use coordination::{coordination_status, BlockedReason, CoordinationStatus};
pub use errors::WorkflowAdmissionError;
pub use global_plan::{
    plan_global_workflows, GlobalExecutionVertex, GlobalWorkflowPlan, PlannerInputsByWorkflow,
    WorkflowNodeRef,
};
pub use key::{
    derive_miss_reason, CanonicalKeyPayload, MaterializationDigest, MaterializationKey, MissReason,
    WorkIdentity,
};
pub use ledger::{
    append_global_ledger_entry, load_global_ledger, rehydrate_outputs_for_entry,
    save_global_ledger, store_output_payload, workflow_ledger_paths, LedgerStatus, RunId,
    RunLedgerEntry, WorkflowLedgerError, WorkflowLedgerPaths,
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
pub use executor::{execute_workflow_plan, ExecutionSummary, UnitCommand, UnitResult};
pub use slo::{
    check_slo, default_slo_budgets, render_execution_report, top_slow_units, SloBudget, SloResult,
    SlowUnit,
};
pub use spec_builders::{
    bootstrap_workflow_spec, bootstrap_workflow_spec_with_registry, ci_workflow_spec,
    ci_workflow_spec_with_registry, dag_snapshot_workflow_spec,
    dag_snapshot_workflow_spec_with_registry, dag_viz_workflow_spec,
    dag_viz_workflow_spec_with_registry, deps_workflow_spec, deps_workflow_spec_with_registry,
    makegen_workflow_spec, makegen_workflow_spec_with_registry, pragma_workflow_spec,
    pragma_workflow_spec_with_registry, test_all_workflow_spec,
    test_all_workflow_spec_with_registry, tool_workflow_spec, TOOL_WORKFLOW_NAMES,
};
pub use unit_commands::{ci_unit_commands, test_all_unit_commands};
