//! SDLC runtime state primitives.

pub mod artifacts;
pub mod claims;
pub mod reconcile;
pub mod retry;
pub mod state;
pub mod transaction;

pub use artifacts::{
    canonical_marker, content_hash_for_payload, promote_to_canonical_artifact,
    promote_to_canonical_artifact_with_payload, provisional_marker, upsert_provisional_artifact,
    upsert_provisional_artifact_with_payload, ArtifactLedger, ArtifactPayload, ArtifactRecord,
    ArtifactUpsertOutcome,
};
pub use claims::{
    claim_slot_key, heartbeat_claim, release_claim, try_acquire_claim, ClaimAcquireResult,
    ClaimLedger, ClaimRecord,
};
pub use reconcile::{reconcile_entries, ReconcileAction, ReconcileEntry, ReconcilePlan};
pub use retry::{register_retry_failure, retry_ready, RetryState};
pub use state::{
    mark_run_completed, mark_run_failed, should_replay_skip, RunExecutionStatus, RunStateLedger,
    RunStateRecord,
};
pub use transaction::validate_stage_transition;
