use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ArtifactLedger {
    pub records: BTreeMap<String, ArtifactRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactPayload {
    Inline { body: String },
    BlobRef { uri: String, size_bytes: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub marker: String,
    pub intake_key: String,
    pub run_key: String,
    pub content_hash: String,
    #[serde(default)]
    pub payload: Option<ArtifactPayload>,
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
        None,
        false,
        now_epoch_ms,
    )
}

pub fn upsert_provisional_artifact_with_payload(
    ledger: &mut ArtifactLedger,
    intake_key: &str,
    run_key: &str,
    payload: ArtifactPayload,
    now_epoch_ms: u128,
) -> Result<ArtifactUpsertOutcome, String> {
    let normalized = normalize_payload(payload)?;
    let content_hash = content_hash_for_payload(&normalized);
    upsert_artifact(
        ledger,
        &provisional_marker(intake_key),
        intake_key,
        run_key,
        &content_hash,
        Some(normalized),
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
        None,
        true,
        now_epoch_ms,
    )
}

pub fn promote_to_canonical_artifact_with_payload(
    ledger: &mut ArtifactLedger,
    intake_key: &str,
    run_key: &str,
    payload: ArtifactPayload,
    now_epoch_ms: u128,
) -> Result<ArtifactUpsertOutcome, String> {
    let normalized = normalize_payload(payload)?;
    let content_hash = content_hash_for_payload(&normalized);
    upsert_artifact(
        ledger,
        &canonical_marker(intake_key),
        intake_key,
        run_key,
        &content_hash,
        Some(normalized),
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
    payload: Option<ArtifactPayload>,
    canonical: bool,
    now_epoch_ms: u128,
) -> Result<ArtifactUpsertOutcome, String> {
    if let Some(existing) = ledger.records.get(marker) {
        if existing.run_key != run_key && existing.canonical {
            if existing.content_hash == content_hash {
                return Ok(ArtifactUpsertOutcome::Noop);
            }
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
        payload,
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
                && existing.payload == next.payload
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

fn normalize_payload(payload: ArtifactPayload) -> Result<ArtifactPayload, String> {
    match payload {
        ArtifactPayload::Inline { body } => Ok(ArtifactPayload::Inline {
            body: body.replace("\r\n", "\n"),
        }),
        ArtifactPayload::BlobRef { uri, size_bytes } => {
            let normalized_uri = uri.trim().to_string();
            if normalized_uri.is_empty() {
                return Err("artifact blob payload uri cannot be empty".to_string());
            }
            Ok(ArtifactPayload::BlobRef {
                uri: normalized_uri,
                size_bytes,
            })
        }
    }
}

pub fn content_hash_for_payload(payload: &ArtifactPayload) -> String {
    let bytes = serde_json::to_vec(payload).expect("artifact payload should serialize");
    gunbc_infra::hash::ContentHash::from_bytes(&bytes)
        .as_str()
        .to_string()
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

    #[test]
    fn payload_inline_normalization_is_deterministic() {
        let mut ledger = ArtifactLedger::default();
        let first = upsert_provisional_artifact_with_payload(
            &mut ledger,
            "intent-a",
            "run-a",
            ArtifactPayload::Inline {
                body: "line-1\r\nline-2".to_string(),
            },
            1000,
        )
        .expect("first payload insert should succeed");
        let second = upsert_provisional_artifact_with_payload(
            &mut ledger,
            "intent-a",
            "run-a",
            ArtifactPayload::Inline {
                body: "line-1\nline-2".to_string(),
            },
            1100,
        )
        .expect("second payload insert should succeed");
        assert_eq!(first, ArtifactUpsertOutcome::Inserted);
        assert_eq!(second, ArtifactUpsertOutcome::Noop);
        let stored = ledger
            .records
            .get(&provisional_marker("intent-a"))
            .expect("provisional record should exist");
        assert_eq!(
            stored.payload,
            Some(ArtifactPayload::Inline {
                body: "line-1\nline-2".to_string()
            })
        );
    }

    #[test]
    fn canonical_collision_with_equal_payload_hash_is_noop() {
        let mut ledger = ArtifactLedger::default();
        let payload = ArtifactPayload::BlobRef {
            uri: "s3://bucket/path".to_string(),
            size_bytes: 42,
        };
        let first = promote_to_canonical_artifact_with_payload(
            &mut ledger,
            "intent-a",
            "run-a",
            payload.clone(),
            1000,
        )
        .expect("first canonical insert should succeed");
        let second = promote_to_canonical_artifact_with_payload(
            &mut ledger,
            "intent-a",
            "run-b",
            payload,
            1100,
        )
        .expect("same payload hash canonical collision should noop");
        assert_eq!(first, ArtifactUpsertOutcome::Inserted);
        assert_eq!(second, ArtifactUpsertOutcome::Noop);
    }

    #[test]
    fn blob_payload_requires_non_empty_uri() {
        let mut ledger = ArtifactLedger::default();
        let err = upsert_provisional_artifact_with_payload(
            &mut ledger,
            "intent-a",
            "run-a",
            ArtifactPayload::BlobRef {
                uri: "   ".to_string(),
                size_bytes: 7,
            },
            1000,
        )
        .expect_err("empty blob uri should fail closed");
        assert!(err.contains("artifact blob payload uri cannot be empty"));
    }
}
