use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ArtifactLedger {
    pub records: BTreeMap<String, ArtifactRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub marker: String,
    pub intake_key: String,
    pub run_key: String,
    pub content_hash: String,
    pub canonical: bool,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactUpsertOutcome {
    Inserted,
    Updated,
    Noop,
}

pub fn provisional_marker(intake_key: &str) -> String {
    format!("sdlc:artifact:provisional:{intake_key}")
}

pub fn canonical_marker(intake_key: &str) -> String {
    format!("sdlc:artifact:canonical:{intake_key}")
}

pub fn upsert_provisional_artifact(
    ledger: &mut ArtifactLedger,
    intake_key: &str,
    run_key: &str,
    content_hash: &str,
    now_epoch_ms: u128,
) -> Result<ArtifactUpsertOutcome, String> {
    upsert_artifact(
        ledger,
        &provisional_marker(intake_key),
        intake_key,
        run_key,
        content_hash,
        false,
        now_epoch_ms,
    )
}

pub fn promote_to_canonical_artifact(
    ledger: &mut ArtifactLedger,
    intake_key: &str,
    run_key: &str,
    content_hash: &str,
    now_epoch_ms: u128,
) -> Result<ArtifactUpsertOutcome, String> {
    upsert_artifact(
        ledger,
        &canonical_marker(intake_key),
        intake_key,
        run_key,
        content_hash,
        true,
        now_epoch_ms,
    )
}

fn upsert_artifact(
    ledger: &mut ArtifactLedger,
    marker: &str,
    intake_key: &str,
    run_key: &str,
    content_hash: &str,
    canonical: bool,
    now_epoch_ms: u128,
) -> Result<ArtifactUpsertOutcome, String> {
    if let Some(existing) = ledger.records.get(marker) {
        if existing.run_key != run_key && existing.canonical {
            return Err(format!(
                "artifact marker collision on `{marker}`: existing canonical run_key `{}` conflicts with `{run_key}`",
                existing.run_key
            ));
        }
    }

    let next = ArtifactRecord {
        marker: marker.to_string(),
        intake_key: intake_key.to_string(),
        run_key: run_key.to_string(),
        content_hash: content_hash.to_string(),
        canonical,
        updated_at_epoch_ms: now_epoch_ms,
    };
    match ledger.records.get(marker) {
        None => {
            ledger.records.insert(marker.to_string(), next);
            Ok(ArtifactUpsertOutcome::Inserted)
        }
        Some(existing)
            if existing.run_key == run_key
                && existing.content_hash == content_hash
                && existing.canonical == canonical =>
        {
            Ok(ArtifactUpsertOutcome::Noop)
        }
        Some(_) => {
            ledger.records.insert(marker.to_string(), next);
            Ok(ArtifactUpsertOutcome::Updated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provisional_upsert_is_idempotent_for_same_payload() {
        let mut ledger = ArtifactLedger::default();
        let first = upsert_provisional_artifact(&mut ledger, "intent-a", "run-a", "hash-1", 1000)
            .expect("first provisional upsert should succeed");
        assert_eq!(first, ArtifactUpsertOutcome::Inserted);
        let second =
            upsert_provisional_artifact(&mut ledger, "intent-a", "run-a", "hash-1", 1100)
                .expect("second provisional upsert should succeed");
        assert_eq!(second, ArtifactUpsertOutcome::Noop);
    }

    #[test]
    fn canonical_collision_fails_closed() {
        let mut ledger = ArtifactLedger::default();
        let _ = promote_to_canonical_artifact(&mut ledger, "intent-a", "run-a", "hash-1", 1000)
            .expect("canonical insert should succeed");
        let err = promote_to_canonical_artifact(&mut ledger, "intent-a", "run-b", "hash-2", 1200)
            .expect_err("canonical collision should fail closed");
        assert!(err.contains("artifact marker collision"));
    }

    #[test]
    fn promote_to_canonical_is_supported_for_same_run_key() {
        let mut ledger = ArtifactLedger::default();
        let _ = upsert_provisional_artifact(&mut ledger, "intent-a", "run-a", "hash-1", 1000)
            .expect("provisional insert should succeed");
        let promoted =
            promote_to_canonical_artifact(&mut ledger, "intent-a", "run-a", "hash-1", 1200)
                .expect("promotion should succeed");
        assert_eq!(promoted, ArtifactUpsertOutcome::Inserted);
        assert!(ledger
            .records
            .get(&canonical_marker("intent-a"))
            .expect("canonical marker should exist")
            .canonical);
    }
}
