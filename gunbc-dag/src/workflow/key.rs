//! Deterministic workflow key payloads and miss-reason typing (WF3).

use std::collections::BTreeMap;

use gunbc_infra::hash::ContentHash;
use gunbc_ir::{NodeId, PortName};
use serde::{Deserialize, Serialize};

use super::process_registry::ProcessId;

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

/// Full materialization key object used by ledger and planner.
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

/// Derive a typed miss reason from previous vs current canonical payload.
pub fn derive_miss_reason(
    previous: &MaterializationKey,
    current: &MaterializationKey,
) -> Option<MissReason> {
    if previous.digest == current.digest {
        return None;
    }

    if previous.payload.op_version != current.payload.op_version {
        return Some(MissReason::OpVersionChanged {
            old: previous.payload.op_version,
            new: current.payload.op_version,
        });
    }
    if previous.payload.policy_version != current.payload.policy_version {
        return Some(MissReason::PolicyVersionChanged {
            old: previous.payload.policy_version,
            new: current.payload.policy_version,
        });
    }

    for (port, new_hashes) in &current.payload.input_hashes {
        let old_hashes = previous.payload.input_hashes.get(port);
        if old_hashes != Some(new_hashes) {
            return Some(MissReason::InputChanged {
                port: port.clone(),
                old: old_hashes.cloned().unwrap_or_default(),
                new: new_hashes.clone(),
            });
        }
    }
    for (port, old_hashes) in &previous.payload.input_hashes {
        if !current.payload.input_hashes.contains_key(port) {
            return Some(MissReason::InputChanged {
                port: port.clone(),
                old: old_hashes.clone(),
                new: Vec::new(),
            });
        }
    }

    for (port, new_keys) in &current.payload.upstream_keys {
        let old_keys = previous.payload.upstream_keys.get(port);
        if old_keys != Some(new_keys) {
            return Some(MissReason::UpstreamKeyChanged {
                port: port.clone(),
                old: old_keys.cloned().unwrap_or_default(),
                new: new_keys.clone(),
            });
        }
    }
    for (port, old_keys) in &previous.payload.upstream_keys {
        if !current.payload.upstream_keys.contains_key(port) {
            return Some(MissReason::UpstreamKeyChanged {
                port: port.clone(),
                old: old_keys.clone(),
                new: Vec::new(),
            });
        }
    }

    Some(MissReason::NoPriorRun)
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
    fn miss_reason_identifies_input_change() {
        let old = key_with_input_hash("source", "aaa");
        let new = key_with_input_hash("source", "bbb");
        let reason = derive_miss_reason(&old, &new).expect("digest changed should produce reason");
        assert!(matches!(reason, MissReason::InputChanged { .. }));
    }
}
