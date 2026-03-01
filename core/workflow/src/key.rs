//! Deterministic workflow key payloads and miss-reason typing (WF3).

use std::collections::BTreeMap;

use gunbc_infra::hash::ContentHash;
use gunbc_ir::{NodeId, PortName};
use serde::{Deserialize, Serialize};

use crate::process_registry::ProcessId;

/// Deterministic materialization digest (sha256 of canonical key payload).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MaterializationDigest(pub String);

impl MaterializationDigest {
    pub fn from_payload(payload: &CanonicalKeyPayload) -> Result<Self, String> {
        let bytes = serde_json::to_vec(payload)
            .map_err(|error| format!("failed to serialize canonical key payload: {error}"))?;
        Ok(Self(ContentHash::from_bytes(&bytes).as_str().to_string()))
    }
}

/// Context-free work identity (independent of workflow node naming).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorkIdentity {
    pub process_id: ProcessId,
    pub unit_id: NodeId,
}

impl WorkIdentity {
    pub fn new(process_id: impl Into<ProcessId>, unit_id: impl Into<NodeId>) -> Self {
        Self {
            process_id: process_id.into(),
            unit_id: unit_id.into(),
        }
    }
}

/// Versioned canonical key payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalKeyPayload {
    pub key_format_version: u32,
    pub op_version: u32,
    pub input_hashes: BTreeMap<PortName, Vec<String>>,
    pub upstream_keys: BTreeMap<PortName, Vec<MaterializationDigest>>,
    pub policy_version: u32,
}

impl CanonicalKeyPayload {
    pub fn canonicalized(mut self) -> Self {
        for hashes in self.input_hashes.values_mut() {
            hashes.sort();
            hashes.dedup();
        }
        for digests in self.upstream_keys.values_mut() {
            digests.sort();
            digests.dedup();
        }
        self
    }
}

/// Full materialization key object used by planner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializationKey {
    pub work_id: WorkIdentity,
    pub payload: CanonicalKeyPayload,
    pub digest: MaterializationDigest,
}

impl MaterializationKey {
    pub fn new(work_id: WorkIdentity, payload: CanonicalKeyPayload) -> Result<Self, String> {
        let payload = payload.canonicalized();
        let digest = MaterializationDigest::from_payload(&payload)?;
        Ok(Self {
            work_id,
            payload,
            digest,
        })
    }
}

/// Typed cache miss causes used for deterministic planner explainability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissReason {
    NoPriorRun,
    InputChanged {
        port: PortName,
        old: Vec<String>,
        new: Vec<String>,
    },
    UpstreamKeyChanged {
        port: PortName,
        old: Vec<MaterializationDigest>,
        new: Vec<MaterializationDigest>,
    },
    OpVersionChanged {
        old: u32,
        new: u32,
    },
    PolicyVersionChanged {
        old: u32,
        new: u32,
    },
    OutputMissing {
        port: PortName,
    },
    OutputTampered {
        port: PortName,
        expected: String,
        actual: String,
    },
    VolatileEffect {
        effect: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_with_input_hash(port: &str, hash: &str) -> MaterializationKey {
        MaterializationKey::new(
            WorkIdentity::new("ci", "ci.codegen"),
            CanonicalKeyPayload {
                key_format_version: 1,
                op_version: 1,
                input_hashes: BTreeMap::from([(PortName::from(port), vec![hash.to_string()])]),
                upstream_keys: BTreeMap::new(),
                policy_version: 1,
            },
        )
        .expect("key should build")
    }

    #[test]
    fn key_digest_is_deterministic_for_same_payload() {
        let a = key_with_input_hash("source", "aaa");
        let b = key_with_input_hash("source", "aaa");
        assert_eq!(a.digest, b.digest);
    }

    #[test]
    fn key_digest_changes_for_different_input() {
        let a = key_with_input_hash("source", "aaa");
        let b = key_with_input_hash("source", "bbb");
        assert_ne!(a.digest, b.digest);
    }
}
