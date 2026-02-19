//! Typed process-unit registry backing workflow planner units (WF1/WF2).

use std::collections::BTreeMap;

use gunbc_ir::{AccessMode, NodeId};

/// Canonical process identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone)]
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
}

/// Registry for all workflow process-unit references.
#[derive(Debug, Clone, Default)]
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

fn ci_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("ci", unit)
}

fn test_all_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("test_all", unit)
}

/// Default registry for WF1/WF2 planner bootstrap.
pub fn default_process_unit_registry() -> ProcessUnitRegistry {
    let mut registry = ProcessUnitRegistry::new();

    // CI workflow units
    for spec in [
        ProcessUnitSpec::new(
            ci_ref("ci.lint_upsert"),
            1,
            vec![
                UnitClaim::write("file:workspace"),
                UnitClaim::write("ledger:workflow"),
            ],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.codegen"),
            1,
            vec![UnitClaim::write("file:generated:cli")],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.bootstrap"),
            1,
            vec![UnitClaim::write("file:manifest")],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.pragma"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.testgen"),
            1,
            vec![UnitClaim::write("file:generated:tests")],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.build_compile"),
            1,
            vec![
                UnitClaim::write("file:target"),
                UnitClaim::read("tool:cargo"),
            ],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.test_run"),
            1,
            vec![
                UnitClaim::read("file:target"),
                UnitClaim::read("tool:cargo"),
            ],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.clippy_run"),
            1,
            vec![
                UnitClaim::read("file:target"),
                UnitClaim::read("tool:cargo"),
            ],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.guardrails"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            ci_ref("ci.verify"),
            1,
            vec![UnitClaim::read("file:generated")],
        ),
        ProcessUnitSpec::new(ci_ref("ci.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    // test-all workflow units
    for spec in [
        ProcessUnitSpec::new(
            test_all_ref("test_all.lint_upsert"),
            1,
            vec![
                UnitClaim::write("file:workspace"),
                UnitClaim::write("ledger:workflow"),
            ],
        ),
        ProcessUnitSpec::new(
            test_all_ref("test_all.codegen"),
            1,
            vec![UnitClaim::write("file:generated:cli")],
        ),
        ProcessUnitSpec::new(
            test_all_ref("test_all.testgen"),
            1,
            vec![UnitClaim::write("file:generated:tests")],
        ),
        ProcessUnitSpec::new(
            test_all_ref("test_all.build_compile"),
            1,
            vec![
                UnitClaim::write("file:target"),
                UnitClaim::read("tool:cargo"),
            ],
        ),
        ProcessUnitSpec::new(
            test_all_ref("test_all.verify_fix"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(
            test_all_ref("test_all.cargo_test_xl"),
            1,
            vec![
                UnitClaim::read("file:target"),
                UnitClaim::read("tool:cargo"),
            ],
        ),
        ProcessUnitSpec::new(test_all_ref("test_all.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_contains_ci_and_test_all_units() {
        let registry = default_process_unit_registry();
        assert!(registry.contains(&ProcessUnitRef::new("ci", "ci.codegen")));
        assert!(registry.contains(&ProcessUnitRef::new("test_all", "test_all.codegen")));
    }

    #[test]
    fn registry_exposes_required_claims() {
        let registry = default_process_unit_registry();
        let spec = registry
            .get(&ProcessUnitRef::new("ci", "ci.build_compile"))
            .expect("ci.build_compile should exist");
        assert!(spec.required_claims.iter().any(
            |claim| claim.claim_id.0 == "file:target" && claim.access_mode == AccessMode::Write
        ));
    }
}
