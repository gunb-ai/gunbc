//! Typed process-unit registry backing workflow planner units (WF1/WF2).

use std::collections::BTreeMap;

use gunbc_ir::{AccessMode, NodeId};
use serde::{Deserialize, Serialize};

use super::capabilities::{
    CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
};

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

fn ci_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("ci", unit)
}

fn test_all_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("test_all", unit)
}

fn compilation_ref() -> ProcessUnitRef {
    ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT)
}

fn codegen_ref() -> ProcessUnitRef {
    ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT)
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

    // Universal capability units (WF14/WF15).
    //
    // These are shared across all workflows via context-free WorkIdentity.
    // `compilation.ensure` and `codegen.ensure` resolve to the same identity
    // regardless of which workflow invokes them.
    for spec in [
        // WF14: Compilation capability — binary freshness as a planner-managed
        // keyed unit. Key inputs: source hashes, cargo metadata, compiler version.
        // Claims: writes to file:target (binary output), reads tool:cargo.
        ProcessUnitSpec::new(
            compilation_ref(),
            1,
            vec![
                UnitClaim::write("file:target"),
                UnitClaim::read("tool:cargo"),
            ],
        ),
        // WF15: Codegen capability — codegen freshness as a keyed unit.
        // Key inputs: DSL source hashes, codegen binary version.
        // Claims: writes generated CLI entrypoints.
        ProcessUnitSpec::new(
            codegen_ref(),
            1,
            vec![UnitClaim::write("file:generated:cli")],
        ),
    ] {
        registry.register(spec);
    }

    registry
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

fn canonicalize_unit_id(unit_id: &NodeId) -> NodeId {
    if let Some((_, suffix)) = unit_id.0.split_once('.') {
        NodeId::from(suffix)
    } else {
        unit_id.clone()
    }
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

    #[test]
    fn canonical_work_identity_is_context_free_across_workflows() {
        let registry = default_process_unit_registry();
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

    // WF14/WF15: Universal capability registry tests

    #[test]
    fn default_registry_contains_compilation_and_codegen_units() {
        let registry = default_process_unit_registry();
        assert!(
            registry.contains(&compilation_ref()),
            "compilation.ensure should be registered"
        );
        assert!(
            registry.contains(&codegen_ref()),
            "codegen.ensure should be registered"
        );
    }

    #[test]
    fn compilation_unit_has_correct_claims() {
        let registry = default_process_unit_registry();
        let spec = registry
            .get(&compilation_ref())
            .expect("compilation.ensure should exist");
        assert!(spec
            .required_claims
            .iter()
            .any(|c| c.claim_id.0 == "file:target" && c.access_mode == AccessMode::Write));
        assert!(spec
            .required_claims
            .iter()
            .any(|c| c.claim_id.0 == "tool:cargo" && c.access_mode == AccessMode::Read));
    }

    #[test]
    fn codegen_unit_has_correct_claims() {
        let registry = default_process_unit_registry();
        let spec = registry
            .get(&codegen_ref())
            .expect("codegen.ensure should exist");
        assert!(spec
            .required_claims
            .iter()
            .any(|c| c.claim_id.0 == "file:generated:cli" && c.access_mode == AccessMode::Write));
    }

    #[test]
    fn compilation_canonical_identity_is_context_free() {
        let registry = default_process_unit_registry();
        let spec = registry
            .get(&compilation_ref())
            .expect("compilation_ensure should exist");
        let (process, unit) = spec.canonical_work_identity();
        assert_eq!(process.0, "process-unit");
        // Unit ID uses underscore (not dot) to avoid canonicalization stripping.
        assert_eq!(unit.0, "compilation_ensure");
    }

    #[test]
    fn codegen_capability_canonical_identity_is_context_free() {
        let registry = default_process_unit_registry();
        let spec = registry
            .get(&codegen_ref())
            .expect("codegen_ensure should exist");
        let (process, unit) = spec.canonical_work_identity();
        assert_eq!(process.0, "process-unit");
        assert_eq!(unit.0, "codegen_ensure");
    }

    #[test]
    fn compilation_and_codegen_have_distinct_canonical_identities() {
        let registry = default_process_unit_registry();
        let compilation = registry
            .get(&compilation_ref())
            .expect("compilation_ensure");
        let codegen = registry.get(&codegen_ref()).expect("codegen_ensure");
        assert_ne!(
            compilation.canonical_work_identity(),
            codegen.canonical_work_identity(),
            "compilation and codegen must have distinct canonical identities"
        );
    }
}
