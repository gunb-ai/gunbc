//! Typed process-unit registry backing workflow planner units (WF1/WF2).

use std::collections::BTreeMap;

use gunbc_ir::{AccessMode, NodeId};
use serde::{Deserialize, Serialize};

/// Canonical process identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProcessId(pub String);

impl ProcessId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for ProcessId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Stable typed process-unit reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProcessUnitRef {
    pub process_id: ProcessId,
    pub unit_id: NodeId,
}

impl ProcessUnitRef {
    pub fn new(process_id: impl Into<ProcessId>, unit_id: impl Into<NodeId>) -> Self {
        Self {
            process_id: process_id.into(),
            unit_id: unit_id.into(),
        }
    }
}

/// Canonical claim identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaimId(pub String);

impl ClaimId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_resource_name(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ClaimId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ClaimId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Declared claim for a workflow unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitClaim {
    pub claim_id: ClaimId,
    pub access_mode: AccessMode,
}

impl UnitClaim {
    pub fn new(claim_id: impl Into<ClaimId>, access_mode: AccessMode) -> Self {
        Self {
            claim_id: claim_id.into(),
            access_mode,
        }
    }

    pub fn read(claim_id: impl Into<ClaimId>) -> Self {
        Self::new(claim_id, AccessMode::Read)
    }

    pub fn write(claim_id: impl Into<ClaimId>) -> Self {
        Self::new(claim_id, AccessMode::Write)
    }
}

/// Typed process-unit metadata required by workflow planner phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessUnitSpec {
    pub reference: ProcessUnitRef,
    pub op_version: u32,
    pub required_claims: Vec<UnitClaim>,
}

impl ProcessUnitSpec {
    pub fn new(
        reference: ProcessUnitRef,
        op_version: u32,
        required_claims: Vec<UnitClaim>,
    ) -> Self {
        Self {
            reference,
            op_version,
            required_claims,
        }
    }

    /// Context-free work identity projection for cross-workflow dedup.
    pub fn canonical_work_identity(&self) -> (ProcessId, NodeId) {
        (
            ProcessId::new("process-unit"),
            canonicalize_unit_id(&self.reference.unit_id),
        )
    }
}

/// Registry for all workflow process-unit references.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessUnitRegistry {
    specs: BTreeMap<ProcessUnitRef, ProcessUnitSpec>,
}

impl ProcessUnitRegistry {
    pub fn new() -> Self {
        Self {
            specs: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, spec: ProcessUnitSpec) {
        self.specs.insert(spec.reference.clone(), spec);
    }

    pub fn get(&self, reference: &ProcessUnitRef) -> Option<&ProcessUnitSpec> {
        self.specs.get(reference)
    }

    pub fn contains(&self, reference: &ProcessUnitRef) -> bool {
        self.specs.contains_key(reference)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProcessUnitSpec> {
        self.specs.values()
    }
}

fn canonicalize_unit_id(unit_id: &NodeId) -> NodeId {
    if let Some((_, suffix)) = unit_id.0.split_once('.') {
        NodeId::from(suffix)
    } else {
        unit_id.clone()
    }
}

/// Canonical handle type auto-wiring policy for resource claims.
pub fn claim_handle_type_id(claim_id: &ClaimId) -> &'static str {
    if claim_id.0.starts_with("file:") {
        "FilesystemHandle"
    } else if claim_id.0.starts_with("tool:") {
        "ToolHandle"
    } else if claim_id.0.starts_with("ledger:") {
        "WorkflowLedgerHandle"
    } else if claim_id.0.starts_with("network:") {
        "NetworkHandle"
    } else {
        "ResourceHandle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_handle_type_policy_maps_common_prefixes() {
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("file:workspace")),
            "FilesystemHandle"
        );
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("tool:cargo")),
            "ToolHandle"
        );
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("ledger:workflow")),
            "WorkflowLedgerHandle"
        );
    }

    #[test]
    fn network_claim_maps_to_network_handle() {
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("network:github_gist")),
            "NetworkHandle"
        );
    }

    #[test]
    fn credential_claim_maps_to_resource_handle() {
        assert_eq!(
            claim_handle_type_id(&ClaimId::new("credential:github")),
            "ResourceHandle"
        );
    }

    #[test]
    fn canonical_work_identity_is_context_free_across_workflows() {
        let mut registry = ProcessUnitRegistry::new();
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("ci", "ci.codegen"),
            1,
            vec![UnitClaim::read("file:dsl")],
        ));
        registry.register(ProcessUnitSpec::new(
            ProcessUnitRef::new("test_all", "test_all.codegen"),
            1,
            vec![UnitClaim::read("file:dsl")],
        ));
        let ci = registry
            .get(&ProcessUnitRef::new("ci", "ci.codegen"))
            .expect("ci.codegen");
        let test_all = registry
            .get(&ProcessUnitRef::new("test_all", "test_all.codegen"))
            .expect("test_all.codegen");
        assert_eq!(
            ci.canonical_work_identity(),
            test_all.canonical_work_identity()
        );
    }
}
