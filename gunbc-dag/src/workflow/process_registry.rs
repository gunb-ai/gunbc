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

fn bootstrap_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("bootstrap", unit)
}

fn makegen_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("makegen", unit)
}

fn pragma_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("pragma", unit)
}

fn deps_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("deps", unit)
}

fn dag_viz_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("dag_viz", unit)
}

fn dag_snapshot_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("dag_snapshot", unit)
}

fn gist_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("gist", unit)
}

fn build_all_ref(unit: &str) -> ProcessUnitRef {
    ProcessUnitRef::new("build_all", unit)
}

fn compilation_ref() -> ProcessUnitRef {
    ProcessUnitRef::new(COMPILATION_PROCESS_ID, COMPILATION_ENSURE_UNIT)
}

fn codegen_ref() -> ProcessUnitRef {
    ProcessUnitRef::new(CODEGEN_PROCESS_ID, CODEGEN_ENSURE_UNIT)
}

/// Universal capability claims shared across all tool workflows.
fn compilation_ensure_claims() -> Vec<UnitClaim> {
    vec![
        UnitClaim::write("file:target"),
        UnitClaim::read("tool:cargo"),
    ]
}

/// Universal codegen capability claims.
fn codegen_ensure_claims() -> Vec<UnitClaim> {
    vec![UnitClaim::write("file:generated:cli")]
}

/// Default registry for WF1/WF2 planner bootstrap.
pub fn default_process_unit_registry() -> ProcessUnitRegistry {
    let mut registry = ProcessUnitRegistry::new();

    // Canonical universal capabilities for cross-workflow reuse.
    for spec in [
        ProcessUnitSpec::new(compilation_ref(), 1, compilation_ensure_claims()),
        ProcessUnitSpec::new(codegen_ref(), 1, codegen_ensure_claims()),
    ] {
        registry.register(spec);
    }

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

    // =========================================================================
    // WF16/WF17/WF18: Gist workflow units (snapshot/diff/recent)
    // =========================================================================
    for spec in [
        // Shared base units
        ProcessUnitSpec::new(
            gist_ref("gist.branch_resolution"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            gist_ref("gist.credential_resolve"),
            1,
            vec![UnitClaim::read("credential:github")],
        ),
        ProcessUnitSpec::new(
            gist_ref("gist.gist_create"),
            1,
            vec![UnitClaim::write("network:github_gist")],
        ),
        // WF16: snapshot content acquisition
        ProcessUnitSpec::new(
            gist_ref("gist.list_files"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            gist_ref("gist.read_files"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(gist_ref("gist.render_snapshot"), 1, vec![]),
        // WF17/WF18 shared diff rendering
        ProcessUnitSpec::new(
            gist_ref("gist.diff"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(gist_ref("gist.render_diff"), 1, vec![]),
        // WF18: recent-specific source selection
        ProcessUnitSpec::new(
            gist_ref("gist.rev_list"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
    ] {
        registry.register(spec);
    }

    // =========================================================================
    // WF19: Bootstrap workflow units
    // =========================================================================
    for spec in [
        // Tool-specific: workspace scan
        ProcessUnitSpec::new(
            bootstrap_ref("bootstrap.workspace_scan"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        // Tool-specific: parallel generation
        ProcessUnitSpec::new(
            bootstrap_ref("bootstrap.generate_makefile"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            bootstrap_ref("bootstrap.generate_gitignore"),
            1,
            vec![], // pure computation
        ),
        // Tool-specific: parallel upsert (filesystem write)
        ProcessUnitSpec::new(
            bootstrap_ref("bootstrap.upsert_makefile"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(
            bootstrap_ref("bootstrap.upsert_gitignore"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(bootstrap_ref("bootstrap.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    // =========================================================================
    // WF19: Makegen workflow units
    // =========================================================================
    for spec in [
        ProcessUnitSpec::new(
            makegen_ref("makegen.load_registry"),
            1,
            vec![], // pure: reads tool registry
        ),
        ProcessUnitSpec::new(
            makegen_ref("makegen.render_makefile"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            makegen_ref("makegen.upsert_makefile"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(makegen_ref("makegen.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    // =========================================================================
    // WF19: Pragma workflow units
    // =========================================================================
    for spec in [
        // Three independent parallel render+upsert chains
        ProcessUnitSpec::new(
            pragma_ref("pragma.render_clippy"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            pragma_ref("pragma.upsert_clippy"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(
            pragma_ref("pragma.render_allowlist"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            pragma_ref("pragma.upsert_allowlist"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(
            pragma_ref("pragma.render_policy"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            pragma_ref("pragma.upsert_policy"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(pragma_ref("pragma.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    // =========================================================================
    // WF20: Deps workflow units (install + generate)
    // =========================================================================
    for spec in [
        // Install graph
        ProcessUnitSpec::new(
            deps_ref("deps.platform_env"),
            1,
            vec![], // pure: platform detection
        ),
        ProcessUnitSpec::new(
            deps_ref("deps.load_manifest"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            deps_ref("deps.generate_scripts"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            deps_ref("deps.execute_installs"),
            1,
            vec![UnitClaim::write("tool:package_manager")],
        ),
        // Generate graph
        ProcessUnitSpec::new(
            deps_ref("deps.load_tool_registry"),
            1,
            vec![], // pure
        ),
        ProcessUnitSpec::new(
            deps_ref("deps.render_deps_toml"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            deps_ref("deps.write_deps_toml"),
            1,
            vec![UnitClaim::write("file:workspace")],
        ),
        ProcessUnitSpec::new(deps_ref("deps.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    // =========================================================================
    // WF20: DAG Viz workflow units (shared base + mode-specific)
    // =========================================================================
    for spec in [
        // Shared base units (same WorkIdentity as gist via canonical dedup)
        ProcessUnitSpec::new(
            dag_viz_ref("dag_viz.branch_resolution"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            dag_viz_ref("dag_viz.credential_resolve"),
            1,
            vec![UnitClaim::read("credential:github")],
        ),
        // Viz-specific content acquisition
        ProcessUnitSpec::new(
            dag_viz_ref("dag_viz.serialize_dag"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            dag_viz_ref("dag_viz.render_viz"),
            1,
            vec![], // pure computation
        ),
        // Network transport (volatile)
        ProcessUnitSpec::new(
            dag_viz_ref("dag_viz.gist_upload"),
            1,
            vec![
                UnitClaim::write("network:github_gist"),
                UnitClaim::read("credential:github"),
            ],
        ),
        ProcessUnitSpec::new(dag_viz_ref("dag_viz.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    // =========================================================================
    // WF20: DAG Snapshot workflow units
    // =========================================================================
    for spec in [
        ProcessUnitSpec::new(
            dag_snapshot_ref("dag_snapshot.branch_resolution"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            dag_snapshot_ref("dag_snapshot.credential_resolve"),
            1,
            vec![UnitClaim::read("credential:github")],
        ),
        ProcessUnitSpec::new(
            dag_snapshot_ref("dag_snapshot.list_files"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            dag_snapshot_ref("dag_snapshot.read_files"),
            1,
            vec![UnitClaim::read("file:workspace")],
        ),
        ProcessUnitSpec::new(
            dag_snapshot_ref("dag_snapshot.render_snapshot"),
            1,
            vec![], // pure computation
        ),
        ProcessUnitSpec::new(
            dag_snapshot_ref("dag_snapshot.gist_upload"),
            1,
            vec![
                UnitClaim::write("network:github_gist"),
                UnitClaim::read("credential:github"),
            ],
        ),
        ProcessUnitSpec::new(dag_snapshot_ref("dag_snapshot.report"), 1, vec![]),
    ] {
        registry.register(spec);
    }

    // =========================================================================
    // Build-all workflow units
    // =========================================================================
    registry.register(ProcessUnitSpec::new(
        build_all_ref("build_all.build"),
        1,
        vec![
            UnitClaim::write("file:target"),
            UnitClaim::read("tool:cargo"),
        ],
    ));

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

    #[test]
    fn default_registry_contains_canonical_capability_units() {
        let registry = default_process_unit_registry();
        assert!(registry.contains(&compilation_ref()));
        assert!(registry.contains(&codegen_ref()));
    }

    #[test]
    fn default_registry_contains_tool_workflow_units() {
        let registry = default_process_unit_registry();
        // WF16/WF17/WF18: gist modes
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.branch_resolution")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.credential_resolve")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.gist_create")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.list_files")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.read_files")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.render_snapshot")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.diff")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.render_diff")));
        assert!(registry.contains(&ProcessUnitRef::new("gist", "gist.rev_list")));

        // WF19: bootstrap
        assert!(registry.contains(&ProcessUnitRef::new(
            "bootstrap",
            "bootstrap.workspace_scan"
        )));
        assert!(registry.contains(&ProcessUnitRef::new(
            "bootstrap",
            "bootstrap.generate_makefile"
        )));
        assert!(registry.contains(&ProcessUnitRef::new(
            "bootstrap",
            "bootstrap.upsert_makefile"
        )));
        // WF19: makegen
        assert!(registry.contains(&ProcessUnitRef::new("makegen", "makegen.load_registry")));
        assert!(registry.contains(&ProcessUnitRef::new("makegen", "makegen.upsert_makefile")));
        // WF19: pragma
        assert!(registry.contains(&ProcessUnitRef::new("pragma", "pragma.render_clippy")));
        assert!(registry.contains(&ProcessUnitRef::new("pragma", "pragma.upsert_clippy")));
        // WF20: deps
        assert!(registry.contains(&ProcessUnitRef::new("deps", "deps.load_manifest")));
        assert!(registry.contains(&ProcessUnitRef::new("deps", "deps.execute_installs")));
        // WF20: dag_viz
        assert!(registry.contains(&ProcessUnitRef::new("dag_viz", "dag_viz.branch_resolution")));
        assert!(registry.contains(&ProcessUnitRef::new("dag_viz", "dag_viz.gist_upload")));
        // WF20: dag_snapshot
        assert!(registry.contains(&ProcessUnitRef::new(
            "dag_snapshot",
            "dag_snapshot.list_files"
        )));
        assert!(registry.contains(&ProcessUnitRef::new(
            "dag_snapshot",
            "dag_snapshot.gist_upload"
        )));
        // Build-all
        assert!(registry.contains(&ProcessUnitRef::new("build_all", "build_all.build")));
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
