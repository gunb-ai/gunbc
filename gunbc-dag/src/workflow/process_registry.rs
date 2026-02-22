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

/// Default registry for WF1/WF2 planner bootstrap.
///
/// Derived from `dsl/workflows/*.dag` stage claim annotations.
pub fn default_process_unit_registry() -> ProcessUnitRegistry {
    super::catalog::build_process_unit_registry().unwrap_or_else(|error| {
        panic!("failed to derive process unit registry from DSL workflows: {error}")
    })
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
    use crate::workflow::capabilities::{
        CODEGEN_ENSURE_UNIT, CODEGEN_PROCESS_ID, COMPILATION_ENSURE_UNIT, COMPILATION_PROCESS_ID,
    };

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

    #[test]
    fn default_registry_contains_canonical_capability_units() {
        let registry = default_process_unit_registry();
        assert!(registry.contains(&ProcessUnitRef::new(
            COMPILATION_PROCESS_ID,
            COMPILATION_ENSURE_UNIT
        )));
        assert!(registry.contains(&ProcessUnitRef::new(
            CODEGEN_PROCESS_ID,
            CODEGEN_ENSURE_UNIT
        )));
    }

    #[test]
    fn default_registry_contains_tool_workflow_units() {
        let registry = default_process_unit_registry();
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.branch_resolution")));
        assert!(registry.contains(&ProcessUnitRef::new(
            "bootstrap",
            "bootstrap.workspace_scan"
        )));
        assert!(registry.contains(&ProcessUnitRef::new("makegen", "makegen.load_registry")));
        assert!(registry.contains(&ProcessUnitRef::new("pragma", "pragma.render_clippy")));
        assert!(registry.contains(&ProcessUnitRef::new("deps", "deps.load_manifest")));
        assert!(registry.contains(&ProcessUnitRef::new("dag_viz", "dag_viz.branch_resolution")));
        assert!(registry.contains(&ProcessUnitRef::new(
            "dag_snapshot",
            "dag_snapshot.list_files"
        )));
        assert!(registry.contains(&ProcessUnitRef::new("build_all", "build_all.build")));
        assert!(registry.contains(&ProcessUnitRef::new("sdlc", "sdlc.intake")));
    }

    #[test]
    fn gist_create_unit_has_network_write_claim() {
        let registry = default_process_unit_registry();
        let spec = registry
            .get(&ProcessUnitRef::new("gist", "gist.gist_create"))
            .expect("gist.gist_create");
        assert!(spec
            .required_claims
            .iter()
            .any(|claim| claim.claim_id.0 == "network:github_gist"
                && claim.access_mode == AccessMode::Write));
    }

    #[test]
    fn gist_credential_unit_has_credential_read_claim() {
        let registry = default_process_unit_registry();
        let spec = registry
            .get(&ProcessUnitRef::new("gist", "gist.credential_resolve"))
            .expect("gist.credential_resolve");
        assert!(spec
            .required_claims
            .iter()
            .any(|claim| claim.claim_id.0 == "credential:github"
                && claim.access_mode == AccessMode::Read));
    }

    #[test]
    fn universal_capabilities_are_registered_once_without_workflow_duplication() {
        let registry = default_process_unit_registry();
        let compilation_specs: Vec<_> = registry
            .iter()
            .filter(|spec| spec.reference.process_id.0 == COMPILATION_PROCESS_ID)
            .collect();
        assert_eq!(
            compilation_specs.len(),
            1,
            "compilation capability should be registered once"
        );
        assert_eq!(
            compilation_specs[0].reference.unit_id.0,
            COMPILATION_ENSURE_UNIT
        );

        let codegen_specs: Vec<_> = registry
            .iter()
            .filter(|spec| spec.reference.process_id.0 == CODEGEN_PROCESS_ID)
            .collect();
        assert_eq!(
            codegen_specs.len(),
            1,
            "codegen capability should be registered once"
        );
        assert_eq!(codegen_specs[0].reference.unit_id.0, CODEGEN_ENSURE_UNIT);
    }

    #[test]
    fn dag_viz_and_dag_snapshot_share_base_capability_identities() {
        let registry = default_process_unit_registry();
        let viz_branch = registry
            .get(&ProcessUnitRef::new("dag_viz", "dag_viz.branch_resolution"))
            .expect("dag_viz.branch_resolution");
        let snap_branch = registry
            .get(&ProcessUnitRef::new(
                "dag_snapshot",
                "dag_snapshot.branch_resolution",
            ))
            .expect("dag_snapshot.branch_resolution");
        assert_eq!(
            viz_branch.canonical_work_identity(),
            snap_branch.canonical_work_identity(),
            "branch_resolution should share identity across dag_viz and dag_snapshot"
        );

        let viz_cred = registry
            .get(&ProcessUnitRef::new(
                "dag_viz",
                "dag_viz.credential_resolve",
            ))
            .expect("dag_viz.credential_resolve");
        let snap_cred = registry
            .get(&ProcessUnitRef::new(
                "dag_snapshot",
                "dag_snapshot.credential_resolve",
            ))
            .expect("dag_snapshot.credential_resolve");
        assert_eq!(
            viz_cred.canonical_work_identity(),
            snap_cred.canonical_work_identity(),
            "credential_resolve should share identity across dag_viz and dag_snapshot"
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
}
