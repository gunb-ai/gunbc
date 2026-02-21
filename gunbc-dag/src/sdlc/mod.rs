//! SDLC runtime state primitives.

pub mod artifacts;
pub mod claims;
pub mod reconcile;
pub mod retry;

pub use artifacts::{
    canonical_marker, promote_to_canonical_artifact, provisional_marker, upsert_provisional_artifact,
    ArtifactLedger, ArtifactRecord, ArtifactUpsertOutcome,
};
pub use claims::{
    claim_slot_key, release_claim, try_acquire_claim, ClaimAcquireResult, ClaimLedger, ClaimRecord,
};
pub use reconcile::{reconcile_entries, ReconcileAction, ReconcileEntry, ReconcilePlan};
pub use retry::{register_retry_failure, retry_ready, RetryState};
