//! Workflow planner core modules (WF1+).

pub mod admission;
pub mod errors;
pub mod key;
pub mod ledger;
pub mod planner;
pub mod process_registry;
pub mod schema;
pub mod spec_builders;

pub use admission::{
    validate_conflicting_claims, validate_required_claims, validate_workflow_admission,
};
pub use errors::WorkflowAdmissionError;
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
    plan_workflow, NodePlan, PlanAction, PlannerInputs, WorkflowPlan, WorkflowPlannerError,
};
pub use process_registry::{
    default_process_unit_registry, ClaimId, ProcessId, ProcessUnitRef, ProcessUnitRegistry,
    ProcessUnitSpec, UnitClaim,
};
pub use schema::{
    has_required_unit_contract, required_input_contract, required_output_contract, AggregateSpec,
    ReportSpec, WorkflowId, WorkflowOp, WorkflowSpec, WorkflowUnit, PORT_AFTER, PORT_COMMIT,
    PORT_RESULT, TYPE_WORKFLOW_RESULT,
};
pub use spec_builders::{
    ci_workflow_spec, ci_workflow_spec_with_registry, test_all_workflow_spec,
    test_all_workflow_spec_with_registry,
};
