//! Resource types, conflict detection, and unified resource management.
//!
//! This module provides:
//!
//! ## Core Resource Types
//! - [`ResourceId`]: Unique identifier for a resource instance
//! - [`AccessMode`]: How the resource is accessed (Read, Write, Exclusive)
//! - [`ResourceKind`]: Capability vs Observation distinction
//! - [`Resource`]: Trait for resources passed through DAG edges
//! - [`DagResource`]: Trait for canonical `res:*` DAG port contracts
//!
//! ## Managed Resources (Upsert Pattern)
//! - [`ManagedResource`]: Trait for resources with freshness checking
//! - [`ResourceHandle`]: Proof of resource acquisition (unified for tools and build artifacts)
//! - [`ResourceDef`]: Declaration of a resource's inputs and outputs
//! - [`ResourceState`]: Fresh, Stale, Missing, or Error
//! - [`ExecMode`]: Verify (fail on stale) vs Ensure (fix stale)
//!
//! ## Manifest
//! - [`ResourceManifest`]: On-disk storage of resource freshness keys
//! - [`ManifestEntry`]: Entry for a single resource
//! - [`ContentHash`]: SHA-256 hash for freshness keys
//!
//! ## Conflict Detection
//! - [`detect_conflicts`]: Find resource access conflicts in a DAG
//! - [`ResourceAccess`]: A resource access by a node
//! - [`ResourceConflict`]: Detected conflict between two nodes
//!
//! # Unified Resource Model
//!
//! All acquirable things—tools, build artifacts, filesystem handles, auth tokens—
//! are resources with the same upsert semantics: Check → Create → Resolve.
//! The only difference is how freshness is determined:
//!
//! - **Tools**: Binary exists on PATH
//! - **Build artifacts**: Content hash of inputs matches manifest
//! - **Auth tokens**: Environment variable is set
//!
//! # Example
//!
//! ```text
//! use gunbc_ir::resource::{load_manifest_default, save_manifest_default, ExecMode, ManagedResource};
//! use gunbc_lib_transport::TransportIo;
//!
//! let io = TransportIo::new();
//! // Load manifest
//! let mut manifest = load_manifest_default(&io)?;
//!
//! // Acquire a resource (checks freshness, creates if needed based on mode)
//! let handle = my_resource.acquire(ExecMode::Ensure, &mut manifest, &io)?;
//!
//! // Save updated manifest
//! save_manifest_default(&io, &manifest)?;
//! ```

// Submodules
pub mod def;
pub mod defs;
pub mod handle;
pub mod managed;
pub mod registry;
pub mod state;

// Re-exports from submodules
pub use def::{DagRef, InputPattern, ResourceDef, ResourceScope};
pub use defs::{
    codegen_input_patterns, codegen_resource_def, CODEGEN_INPUT_FILES, CODEGEN_INPUT_GLOBS,
};
pub use gunbc_infra::hash::{ContentHash, HashBuilder};
pub use gunbc_infra::manifest::{ManifestEntry, ResourceManifest, DEFAULT_MANIFEST_PATH};
pub use handle::{mock_resource_handle_value, HandleParseError, ResourceHandle};
pub use managed::{
    check_manifest_freshness, compute_key_with_files, load_manifest, load_manifest_default,
    save_manifest, save_manifest_default, update_resource_manifest, FreshnessOptions,
    ManagedResource, ManifestFreshness, ManifestUpdateError, ResourceError, ResourceIo,
    SimpleResource,
};
pub use registry::{ResolutionError, ResourceRegistry};
pub use state::{ExecMode, ResourceState};

// ============================================================================
// Original resource.rs content below (conflict detection, Resource trait, etc.)
// ============================================================================

use crate::dag::{Dag, Port};
use crate::node::{Node, NodeBody, NodeKind};
use crate::types::NodeId;
use crate::{SecretString, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use gunbc_infra::ResourceId;

/// Canonical resource port prefix.
pub const RESOURCE_PORT_PREFIX: &str = "res:";
/// Canonical coarse file resource port.
pub const RESOURCE_FILE: &str = "res:file";
/// Canonical file resource port prefix.
pub const RESOURCE_FILE_PREFIX: &str = "res:file:";
/// Canonical coarse network API resource port.
pub const RESOURCE_API_NETWORK: &str = "res:api:network";

/// Canonical type name for filesystem resource handles.
///
/// Single source of truth for the string previously hardcoded as
/// `"FilesystemHandle"` across resolve.rs, compute, and emit (S15).
pub const FILESYSTEM_HANDLE_TYPE: &str = "FilesystemHandle";
/// Canonical repository resource port.
pub const RESOURCE_REPO: &str = "res:repo";
/// Canonical coarse target resource port.
pub const RESOURCE_TARGET: &str = "res:target";
/// Canonical credential resource port (wired by `auth_input`).
pub const RESOURCE_CREDENTIAL: &str = "res:credential";
/// Canonical environment output port for read filesystem handles.
pub const FILE_HANDLE_READ_PORT: &str = "file:read";
/// Canonical environment output port for write filesystem handles.
pub const FILE_HANDLE_WRITE_PORT: &str = "file:write";
/// Canonical environment output port for network handles.
pub const API_NETWORK_HANDLE_PORT: &str = "api:network";

/// How a resource is accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessMode {
    /// Read-only access. Multiple reads can happen in parallel.
    Read,
    /// Write access. Conflicts with other writes and reads.
    Write,
    /// Exclusive access. Conflicts with any other access.
    Exclusive,
}

impl AccessMode {
    /// Check if this access mode conflicts with another.
    ///
    /// - Read + Read = OK
    /// - Read + Write = CONFLICT
    /// - Write + Write = CONFLICT
    /// - Exclusive + anything = CONFLICT
    pub fn conflicts_with(&self, other: &AccessMode) -> bool {
        match (self, other) {
            (AccessMode::Read, AccessMode::Read) => false,
            (AccessMode::Exclusive, _) | (_, AccessMode::Exclusive) => true,
            _ => true, // Write conflicts with everything except itself being analyzed
        }
    }
}

/// Normalize resource IDs to a canonical naming scheme.
///
/// This is intentionally applied at resource-accounting boundaries so all
/// consumers reason over the same canonical vocabulary.
///
/// Canonical forms:
/// - `file` / `file:<path>`
/// - `tool:<id>`
/// - `api:<provider>`
/// - `repo`
/// - `target` / `target:<name>`
pub fn normalize_resource_id(id: &str) -> String {
    let normalized = id.strip_prefix(RESOURCE_PORT_PREFIX).unwrap_or(id);

    // Wildcard file IDs are currently treated as a coarse file capability.
    // This keeps resource accounting deterministic until full glob semantics
    // are designed and implemented end-to-end.
    if normalized == "file:*" || (normalized.starts_with("file:") && normalized.contains('*')) {
        return "file".to_string();
    }

    normalized.to_string()
}

/// Build a canonical `res:*` port from any canonical resource id.
pub fn resource_port(id: &str) -> String {
    format!("{RESOURCE_PORT_PREFIX}{}", normalize_resource_id(id))
}

/// Build a canonical file resource port (`res:file` or `res:file:<path>`).
pub fn resource_file_port(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        RESOURCE_FILE.to_string()
    } else {
        format!("{RESOURCE_FILE_PREFIX}{path}")
    }
}

/// Check whether a port represents a filesystem resource (S15).
///
/// Returns `true` when both:
/// - The port's type ID is [`FILESYSTEM_HANDLE_TYPE`]
/// - The port's name is `res:file` or starts with `res:file:`
///
/// Single source of truth — replaces the duplicated inline checks in
/// `needs_transport_resource` and `wire_missing_filesystem_resources`.
pub fn is_filesystem_resource_port(port: &crate::dag::Port) -> bool {
    port.type_id.0 == FILESYSTEM_HANDLE_TYPE
        && (port.name.0 == RESOURCE_FILE || port.name.0.starts_with(RESOURCE_FILE_PREFIX))
}

/// Build a canonical API resource port (`res:api:<provider>`).
pub fn resource_api_port(provider: &str) -> String {
    let provider = provider.trim();
    if provider.is_empty() {
        RESOURCE_API_NETWORK.to_string()
    } else {
        resource_port(&format!("api:{provider}"))
    }
}

/// Build a canonical target resource port (`res:target` or `res:target:<name>`).
pub fn resource_target_port(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        RESOURCE_TARGET.to_string()
    } else {
        resource_port(&format!("target:{name}"))
    }
}

/// Resource kind: capability vs observation.
///
/// Capabilities are active handles that grant permission to perform actions.
/// Observations are passive snapshots of world state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Active handle — permission to perform operations.
    Capability,
    /// Snapshot value — immutable fact about the world.
    Observation,
}

/// Secret marker used to indicate a capability value was minted by the framework.
///
/// This prevents JSON/user-supplied values from masquerading as capabilities.
const CAPABILITY_MARKER: &str = "capability";

/// Create a secret capability marker value.
pub fn capability_marker() -> SecretString {
    SecretString::new(CAPABILITY_MARKER)
}

/// Validate that a map contains the capability marker.
pub fn ensure_capability_marker(map: &BTreeMap<String, Value>, kind: &str) -> Result<(), String> {
    match map.get("cap") {
        #[allow(clippy::disallowed_methods)] // Approved: capability-marker validation
        Some(Value::Secret(s)) if s.expose_plaintext_for_transport() == CAPABILITY_MARKER => Ok(()),
        _ => Err(format!(
            "{} value missing capability marker (expected secret field 'cap')",
            kind
        )),
    }
}

/// A resource acquired at a DAG boundary and flowed through edges.
///
/// Resources unify tools, filesystem handles, platform info, clocks, and env vars.
/// Capability resources should include the secret `cap` marker in their Value
/// encoding to prevent JSON/user-supplied values from forging capabilities.
pub trait Resource: Into<Value> + TryFrom<Value> {
    /// Unique identifier for this resource kind.
    fn resource_id(&self) -> ResourceId;
    /// Access mode for conflict detection.
    fn access_mode(&self) -> AccessMode;
    /// Whether this is a capability or observation.
    fn kind(&self) -> ResourceKind;
}

/// DAG-native resource abstraction.
///
/// Extends [`Resource`] with the type metadata needed to generate canonical
/// `res:*` input ports for dependency injection.
pub trait DagResource: Resource {
    /// TypeId used on DAG ports for this resource value.
    const TYPE_ID: &'static str;

    /// Canonical `res:*` input port name for this resource instance.
    fn resource_input_port_name(&self) -> String {
        resource_port(&self.resource_id().0)
    }

    /// Canonical typed input port declaration for this resource instance.
    fn resource_input_port(&self) -> crate::dag::Port {
        crate::dag::Port::resource(
            self.resource_id().0.clone(),
            Self::TYPE_ID,
            self.access_mode(),
        )
    }

    /// Whether a port declaration matches this resource's DAG contract.
    fn matches_resource_input_port(&self, port: &crate::dag::Port) -> bool {
        port.name.0 == self.resource_input_port_name()
            && port.type_id.0 == Self::TYPE_ID
            && port.resource_access == Some(self.access_mode())
    }
}

/// Timestamp snapshot (milliseconds since Unix epoch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp {
    millis: i64,
}

impl Timestamp {
    /// Current time snapshot.
    pub fn now() -> Self {
        Self::from_system_time(SystemTime::now())
    }

    /// Create from a SystemTime.
    pub fn from_system_time(time: SystemTime) -> Self {
        let millis = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_millis() as i64;
        Self { millis }
    }

    /// Convert to SystemTime.
    pub fn to_system_time(self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.millis.max(0) as u64)
    }

    /// Milliseconds since epoch.
    pub fn millis(&self) -> i64 {
        self.millis
    }
}

impl Resource for Timestamp {
    fn resource_id(&self) -> ResourceId {
        ResourceId::new("clock")
    }

    fn access_mode(&self) -> AccessMode {
        AccessMode::Read
    }

    fn kind(&self) -> ResourceKind {
        ResourceKind::Observation
    }
}

impl DagResource for Timestamp {
    const TYPE_ID: &'static str = "Timestamp";
}

impl From<Timestamp> for Value {
    fn from(val: Timestamp) -> Self {
        Value::Int(val.millis)
    }
}

impl TryFrom<&Value> for Timestamp {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int(millis) => Ok(Timestamp { millis: *millis }),
            _ => Err("expected Int for Timestamp".to_string()),
        }
    }
}

impl TryFrom<Value> for Timestamp {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Timestamp::try_from(&value)
    }
}

/// A resource access by a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAccess {
    /// The node performing the access.
    pub node_id: NodeId,
    /// The resource being accessed.
    pub resource_id: ResourceId,
    /// How the resource is accessed.
    pub mode: AccessMode,
}

impl ResourceAccess {
    /// Create a new resource access.
    pub fn new(
        node_id: impl Into<NodeId>,
        resource_id: impl Into<ResourceId>,
        mode: AccessMode,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            resource_id: resource_id.into(),
            mode,
        }
    }

    /// Create a read access.
    pub fn read(node_id: impl Into<NodeId>, resource_id: impl Into<ResourceId>) -> Self {
        Self::new(node_id, resource_id, AccessMode::Read)
    }

    /// Create a write access.
    pub fn write(node_id: impl Into<NodeId>, resource_id: impl Into<ResourceId>) -> Self {
        Self::new(node_id, resource_id, AccessMode::Write)
    }

    /// Create an exclusive access.
    pub fn exclusive(node_id: impl Into<NodeId>, resource_id: impl Into<ResourceId>) -> Self {
        Self::new(node_id, resource_id, AccessMode::Exclusive)
    }
}

/// Error for missing or invalid resource access metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAccessError {
    /// Node that declares the resource port.
    pub node_id: NodeId,
    /// Port name that is missing access metadata.
    pub port_name: String,
    /// Resource id derived from the port name.
    pub resource_id: ResourceId,
}

impl std::fmt::Display for ResourceAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "resource access mode missing for {}.{} ({})",
            self.node_id, self.port_name, self.resource_id
        )
    }
}

impl std::error::Error for ResourceAccessError {}

/// A resource conflict between two nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceConflict {
    /// First node in the conflict.
    pub node_a: NodeId,
    /// Second node in the conflict.
    pub node_b: NodeId,
    /// The resource that is being contended.
    pub resource_id: ResourceId,
    /// How node A accesses the resource.
    pub mode_a: AccessMode,
    /// How node B accesses the resource.
    pub mode_b: AccessMode,
}

impl std::fmt::Display for ResourceConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resource conflict: {} ({:?}) and {} ({:?}) both access {}",
            self.node_a, self.mode_a, self.node_b, self.mode_b, self.resource_id
        )
    }
}

/// Detect resource conflicts in a DAG.
///
/// Returns a list of conflicts where:
/// 1. Two nodes access the same resource
/// 2. The access modes conflict (not Read + Read)
/// 3. The nodes have no ordering edge between them
///
/// # Arguments
///
/// * `dag` - The DAG to analyze
/// * `accesses` - List of resource accesses by nodes
///
/// # Returns
///
/// List of detected conflicts
pub fn detect_conflicts<T>(dag: &Dag<T>, accesses: &[ResourceAccess]) -> Vec<ResourceConflict> {
    let mut conflicts = Vec::new();

    // Build a set of ordered pairs (nodes where one must execute before the other)
    let ordered_pairs = compute_ordered_pairs(dag);

    // Check all access pairs for conflicts, including coarse-vs-specific IDs.
    for i in 0..accesses.len() {
        for j in (i + 1)..accesses.len() {
            let access_a = &accesses[i];
            let access_b = &accesses[j];

            if !resource_ids_conflict(&access_a.resource_id, &access_b.resource_id) {
                continue;
            }

            // Check if modes conflict
            if !access_a.mode.conflicts_with(&access_b.mode) {
                continue;
            }

            // Check if nodes are ordered (either A→B or B→A)
            let a_before_b = ordered_pairs.contains(&(&access_a.node_id, &access_b.node_id));
            let b_before_a = ordered_pairs.contains(&(&access_b.node_id, &access_a.node_id));

            if !a_before_b && !b_before_a {
                // No ordering — conflict!
                conflicts.push(ResourceConflict {
                    node_a: access_a.node_id.clone(),
                    node_b: access_b.node_id.clone(),
                    resource_id: conflict_resource_id(&access_a.resource_id, &access_b.resource_id),
                    mode_a: access_a.mode,
                    mode_b: access_b.mode,
                });
            }
        }
    }

    conflicts
}

fn resource_ids_conflict(lhs: &ResourceId, rhs: &ResourceId) -> bool {
    if lhs == rhs {
        return true;
    }

    // Coarse `file` conflicts with any specific `file:<path>` lock.
    let lhs_file = lhs.0 == "file" || lhs.0.starts_with("file:");
    let rhs_file = rhs.0 == "file" || rhs.0.starts_with("file:");
    lhs_file && rhs_file && (lhs.0 == "file" || rhs.0 == "file")
}

fn conflict_resource_id(lhs: &ResourceId, rhs: &ResourceId) -> ResourceId {
    if lhs == rhs {
        return lhs.clone();
    }
    if resource_ids_conflict(lhs, rhs) {
        return ResourceId::new("file");
    }
    lhs.clone()
}

/// Compute all ordered pairs of nodes (A, B) where A must execute before B.
///
/// This is the transitive closure of the edge relationship.
fn compute_ordered_pairs<T>(dag: &Dag<T>) -> HashSet<(&NodeId, &NodeId)> {
    let mut ordered = HashSet::new();

    // Direct edges
    for edge in &dag.edges {
        let from_node = dag.nodes.iter().find(|n| n.id == edge.from_node);
        let to_node = dag.nodes.iter().find(|n| n.id == edge.to_node);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            ordered.insert((&from.id, &to.id));
        }
    }

    // Compute transitive closure
    let node_ids: Vec<&NodeId> = dag.nodes.iter().map(|n| &n.id).collect();

    // Floyd-Warshall style transitive closure
    loop {
        let mut added = false;
        for &a in &node_ids {
            for &b in &node_ids {
                for &c in &node_ids {
                    if ordered.contains(&(a, b))
                        && ordered.contains(&(b, c))
                        && ordered.insert((a, c))
                    {
                        added = true;
                    }
                }
            }
        }
        if !added {
            break;
        }
    }

    ordered
}

/// Check if all resource accesses in a DAG are properly ordered.
///
/// Returns `Ok(())` if there are no conflicts, or `Err(conflicts)` with the list
/// of detected conflicts.
pub fn validate_resource_ordering<T>(
    dag: &Dag<T>,
    accesses: &[ResourceAccess],
) -> Result<(), Vec<ResourceConflict>> {
    let conflicts = detect_conflicts(dag, accesses);
    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(conflicts)
    }
}

fn declared_resource_access(port: &Port) -> Option<(ResourceId, AccessMode)> {
    port.resource_access
        .map(|mode| (ResourceId::new(normalize_resource_id(&port.name.0)), mode))
}

/// Derive resource accesses from declared resource input ports in a DAG.
///
/// Walks all nodes in the DAG and extracts `ResourceAccess` entries from
/// input ports with explicit `resource_access` metadata. Legacy `res:*` ports
/// that are missing `resource_access` remain a hard error.
pub fn derive_resource_accesses<T>(
    dag: &Dag<T>,
) -> Result<Vec<ResourceAccess>, Vec<ResourceAccessError>> {
    let mut accesses = Vec::new();
    let mut errors = Vec::new();
    for node in &dag.nodes {
        for port in &node.inputs {
            if let Some((resource_id, mode)) = declared_resource_access(port) {
                accesses.push(ResourceAccess::new(node.id.clone(), resource_id, mode));
            } else if port.name.0.starts_with(RESOURCE_PORT_PREFIX) {
                let resource_id = ResourceId::new(normalize_resource_id(&port.name.0));
                errors.push(ResourceAccessError {
                    node_id: node.id.clone(),
                    port_name: port.name.0.clone(),
                    resource_id,
                });
            }
        }
    }
    if errors.is_empty() {
        Ok(accesses)
    } else {
        Err(errors)
    }
}

/// Convenience function: derive resource accesses from DAG structure and detect conflicts.
///
/// Combines `derive_resource_accesses()` with `detect_conflicts()` for one-step
/// conflict detection without requiring manual access declarations.
pub fn detect_resource_conflicts<T>(
    dag: &Dag<T>,
) -> Result<Vec<ResourceConflict>, Vec<ResourceAccessError>> {
    let accesses = derive_resource_accesses(dag)?;
    Ok(detect_conflicts(dag, &accesses))
}

// ============================================================================
// M10: Mandatory Resource Declarations
// ============================================================================

/// A node that performs side-effects but declares no resource-access input port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingResourceDeclaration {
    /// The offending node.
    pub node_id: NodeId,
    /// What kind of effect the node performs.
    pub effect_kind: NodeKind,
}

impl std::fmt::Display for MissingResourceDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self.effect_kind {
            NodeKind::TransportExecute => "transport execution",
            NodeKind::ToolEnvironment => "tool environment",
            NodeKind::ResourceEnvironment => "resource environment",
            NodeKind::ToolConsumer => "tool consumption",
            NodeKind::TransportPrepare => "transport prepare",
            NodeKind::TransportParse => "transport parse",
            NodeKind::ResourceAcquire => "resource acquire",
            NodeKind::ResourceRelease => "resource release",
            NodeKind::ParamSource => "param source",
            NodeKind::Collection => "collection",
            NodeKind::Pure => "pure",
        };
        write!(
            f,
            "node '{}' performs {} but declares no resource port",
            self.node_id.0, label
        )
    }
}

/// Whether resource completeness violations are warnings or hard errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceValidationMode {
    /// Log violations but don't fail. For migration.
    Warn,
    /// Fail on any violation. Target state.
    Enforce,
}

/// Classify a node's effect kind, if any.
///
/// Returns `None` for pure nodes. Non-pure nodes return their `NodeKind`.
pub fn classify_effect<T>(node: &Node<T>) -> Option<NodeKind> {
    match node.kind {
        NodeKind::Pure | NodeKind::Collection => None,
        kind => Some(kind),
    }
}

/// Check whether a node declares at least one resource input port.
fn has_resource_port<T>(node: &Node<T>) -> bool {
    node.inputs
        .iter()
        .any(|p| declared_resource_access(p).is_some())
}

/// Validate that all effectful nodes declare resource ports.
///
/// For each node in the DAG, determines if it is effectful (transport executor,
/// tool environment, resource environment, tool consumer) and checks whether it
/// declares at least one input port with explicit `resource_access`. Effectful
/// nodes without resource ports are returned as violations.
///
/// SubDag wrapper nodes are skipped — their resource ports are auto-inferred from
/// inner DAGs. Validation recurses into SubDags to check inner nodes.
pub fn validate_resource_completeness<T>(dag: &Dag<T>) -> Vec<MissingResourceDeclaration> {
    let mut violations = Vec::new();
    validate_resource_completeness_impl(dag, &mut violations);
    violations
}

fn validate_resource_completeness_impl<T>(
    dag: &Dag<T>,
    violations: &mut Vec<MissingResourceDeclaration>,
) {
    for node in &dag.nodes {
        match &node.body {
            NodeBody::SubDag(inner, _) => {
                // SubDag wrappers get resource ports via auto-inference;
                // recurse to validate inner nodes.
                validate_resource_completeness_impl(inner, violations);
            }
            NodeBody::Opaque(_) => {
                if let Some(effect_kind) = classify_effect(node) {
                    if !has_resource_port(node) {
                        violations.push(MissingResourceDeclaration {
                            node_id: node.id.clone(),
                            effect_kind,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{Dag, Edge, Port};
    use crate::node::Node;

    fn test_dag() -> Dag<String> {
        let mut dag = Dag::new();

        dag.add_node(Node::opaque(
            "a",
            vec![],
            vec![Port::scalar("out", "String")],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            "op_b".to_string(),
        ));
        dag.add_node(Node::opaque(
            "c",
            vec![Port::scalar("in", "String")],
            vec![],
            "op_c".to_string(),
        ));

        // a → b → c
        dag.add_edge(Edge::new("a", "out", "b", "in"));
        dag.add_edge(Edge::new("b", "out", "c", "in"));

        dag
    }

    fn parallel_dag() -> Dag<String> {
        let mut dag = Dag::new();

        // Two independent nodes
        dag.add_node(Node::opaque("a", vec![], vec![], "op_a".to_string()));
        dag.add_node(Node::opaque("b", vec![], vec![], "op_b".to_string()));

        dag
    }

    #[test]
    fn test_resource_id_creation() {
        let file = ResourceId::file("/tmp/test.txt");
        assert!(file.0.starts_with("file:"));

        let lock = ResourceId::lock("my_lock");
        assert!(lock.0.starts_with("lock:"));

        let conn = ResourceId::connection("db");
        assert!(conn.0.starts_with("conn:"));

        let build = ResourceId::build("codegen");
        assert!(build.0.starts_with("build:"));
    }

    #[test]
    fn test_access_mode_conflicts() {
        assert!(!AccessMode::Read.conflicts_with(&AccessMode::Read));
        assert!(AccessMode::Read.conflicts_with(&AccessMode::Write));
        assert!(AccessMode::Write.conflicts_with(&AccessMode::Read));
        assert!(AccessMode::Write.conflicts_with(&AccessMode::Write));
        assert!(AccessMode::Exclusive.conflicts_with(&AccessMode::Read));
        assert!(AccessMode::Exclusive.conflicts_with(&AccessMode::Write));
        assert!(AccessMode::Exclusive.conflicts_with(&AccessMode::Exclusive));
    }

    #[test]
    fn test_no_conflict_when_ordered() {
        let dag = test_dag();

        // a writes, then b reads — ordered, no conflict
        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::read("b", "file.txt"),
        ];

        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_conflict_when_parallel() {
        let dag = parallel_dag();

        // a and b both write — parallel, conflict!
        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];

        let conflicts = detect_conflicts(&dag, &accesses);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].resource_id.0, "file.txt");
    }

    #[test]
    fn test_no_conflict_for_parallel_reads() {
        let dag = parallel_dag();

        // a and b both read — parallel, but reads don't conflict
        let accesses = vec![
            ResourceAccess::read("a", "file.txt"),
            ResourceAccess::read("b", "file.txt"),
        ];

        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_conflict_read_write_parallel() {
        let dag = parallel_dag();

        // a reads, b writes — parallel, conflict!
        let accesses = vec![
            ResourceAccess::read("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];

        let conflicts = detect_conflicts(&dag, &accesses);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_transitive_ordering() {
        let dag = test_dag(); // a → b → c

        // a writes, c reads — transitively ordered through b
        let accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::read("c", "file.txt"),
        ];

        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_multiple_resources() {
        let dag = parallel_dag();

        // a writes to file1, b writes to file2 — different resources, no conflict
        let accesses = vec![
            ResourceAccess::write("a", "file1.txt"),
            ResourceAccess::write("b", "file2.txt"),
        ];

        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_coarse_file_conflicts_with_specific_file_lock() {
        let dag = parallel_dag();

        let accesses = vec![
            ResourceAccess::write("a", "file"),
            ResourceAccess::write("b", "file:Makefile"),
        ];

        let conflicts = detect_conflicts(&dag, &accesses);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].resource_id.0, "file");
    }

    #[test]
    fn test_validate_resource_ordering() {
        let dag = parallel_dag();

        // Valid: parallel reads
        let read_accesses = vec![
            ResourceAccess::read("a", "file.txt"),
            ResourceAccess::read("b", "file.txt"),
        ];
        assert!(validate_resource_ordering(&dag, &read_accesses).is_ok());

        // Invalid: parallel writes
        let write_accesses = vec![
            ResourceAccess::write("a", "file.txt"),
            ResourceAccess::write("b", "file.txt"),
        ];
        assert!(validate_resource_ordering(&dag, &write_accesses).is_err());
    }

    // ============ Port::resource() tests ============

    #[test]
    fn test_port_resource_constructor() {
        let port = Port::resource("platform", "Platform", AccessMode::Read);
        assert_eq!(port.name.0, "res:platform");
        assert_eq!(port.type_id.0, "Platform");
        assert_eq!(port.resource_access, Some(AccessMode::Read));
        assert_eq!(port.cardinality, crate::types::Cardinality::ONE);
    }

    #[test]
    fn test_port_resource_write_mode() {
        let port = Port::resource("file", "FilesystemHandle", AccessMode::Write);
        assert_eq!(port.name.0, "res:file");
        assert_eq!(port.resource_access, Some(AccessMode::Write));
    }

    #[test]
    fn test_normalize_resource_id_canonical_only() {
        assert_eq!(normalize_resource_id("res:file"), "file");
        assert_eq!(normalize_resource_id("res:file:Makefile"), "file:Makefile");
        assert_eq!(normalize_resource_id("res:file:*"), "file");
        assert_eq!(normalize_resource_id("res:file:src/*"), "file");
        assert_eq!(normalize_resource_id("res:api:network"), "api:network");
        assert_eq!(normalize_resource_id("res:api:gcp"), "api:gcp");
        assert_eq!(normalize_resource_id("res:target"), "target");
        assert_eq!(
            normalize_resource_id("res:target:manager"),
            "target:manager"
        );
        assert_eq!(normalize_resource_id("res:target:build"), "target:build");
        assert_eq!(normalize_resource_id("res:repo"), "repo");
        assert_eq!(normalize_resource_id("res:tool:clippy"), "tool:clippy");
    }

    #[test]
    fn test_resource_port_builders() {
        assert_eq!(resource_port("file:Makefile"), "res:file:Makefile");
        assert_eq!(resource_port("res:api:gcp"), "res:api:gcp");

        assert_eq!(resource_file_port(""), "res:file");
        assert_eq!(resource_file_port("deps.toml"), "res:file:deps.toml");

        assert_eq!(resource_api_port(""), "res:api:network");
        assert_eq!(resource_api_port("github"), "res:api:github");

        assert_eq!(resource_target_port(""), "res:target");
        assert_eq!(resource_target_port("build"), "res:target:build");
    }

    #[test]
    fn test_dag_resource_timestamp_input_port_contract() {
        let ts = Timestamp::now();
        let port = ts.resource_input_port();
        assert_eq!(port.name.0, "res:clock");
        assert_eq!(port.type_id.0, "Timestamp");
        assert_eq!(port.resource_access, Some(AccessMode::Read));
        assert!(ts.matches_resource_input_port(&port));
    }

    #[test]
    fn test_port_scalar_has_no_resource_access() {
        let port = Port::scalar("data", "String");
        assert!(port.resource_access.is_none());
    }

    // ============ derive_resource_accesses() tests ============

    #[test]
    fn test_derive_resource_accesses_from_res_ports() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "node_a",
            vec![
                Port::resource("platform", "Platform", AccessMode::Read),
                Port::scalar("data", "String"),
            ],
            vec![Port::scalar("out", "String")],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "node_b",
            vec![Port::resource(
                "file",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![],
            "op_b".to_string(),
        ));

        let accesses = derive_resource_accesses(&dag).expect("resource accesses should derive");
        assert_eq!(accesses.len(), 2);

        let platform = accesses
            .iter()
            .find(|a| a.resource_id.0 == "platform")
            .unwrap();
        assert_eq!(platform.node_id.0, "node_a");
        assert_eq!(platform.mode, AccessMode::Read);

        let file = accesses.iter().find(|a| a.resource_id.0 == "file").unwrap();
        assert_eq!(file.node_id.0, "node_b");
        assert_eq!(file.mode, AccessMode::Write);
    }

    #[test]
    fn test_derive_resource_accesses_from_annotated_non_res_ports() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "node_a",
            vec![Port::scalar("tool:clippy", "ToolHandle").with_resource_access(AccessMode::Read)],
            vec![],
            "op_a".to_string(),
        ));

        let accesses = derive_resource_accesses(&dag).expect("resource accesses should derive");
        assert_eq!(accesses.len(), 1);
        assert_eq!(accesses[0].node_id.0, "node_a");
        assert_eq!(accesses[0].resource_id.0, "tool:clippy");
        assert_eq!(accesses[0].mode, AccessMode::Read);
    }

    #[test]
    fn test_derive_resource_accesses_missing_access_errors() {
        // Port declared with scalar("res:platform", "Platform") — no explicit resource_access
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "node_a",
            vec![Port::scalar("res:platform", "Platform")],
            vec![],
            "op_a".to_string(),
        ));

        let errors =
            derive_resource_accesses(&dag).expect_err("missing resource_access should error");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].node_id.0, "node_a");
        assert_eq!(errors[0].port_name, "res:platform");
        assert_eq!(errors[0].resource_id.0, "platform");
    }

    #[test]
    fn test_derive_resource_accesses_ignores_non_res_ports() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "node_a",
            vec![Port::scalar("data", "String"), Port::scalar("count", "Int")],
            vec![],
            "op_a".to_string(),
        ));

        let accesses = derive_resource_accesses(&dag).expect("resource accesses should derive");
        assert!(accesses.is_empty());
    }

    // ============ detect_resource_conflicts() tests ============

    #[test]
    fn test_detect_resource_conflicts_finds_parallel_write() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![Port::resource(
                "file",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![Port::resource(
                "file",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![],
            "op_b".to_string(),
        ));

        let conflicts = detect_resource_conflicts(&dag).expect("resource accesses should derive");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].resource_id.0, "file");
    }

    #[test]
    fn test_detect_resource_conflicts_parallel_reads_ok() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![Port::resource("platform", "Platform", AccessMode::Read)],
            vec![],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![Port::resource("platform", "Platform", AccessMode::Read)],
            vec![],
            "op_b".to_string(),
        ));

        let conflicts = detect_resource_conflicts(&dag).expect("resource accesses should derive");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_resource_conflicts_coarse_file_vs_specific_file() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "a",
            vec![Port::resource(
                "file",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "b",
            vec![Port::resource(
                "file:shared.txt",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![],
            "op_b".to_string(),
        ));

        let conflicts = detect_resource_conflicts(&dag).expect("resource accesses should derive");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].resource_id.0, "file");
    }

    #[test]
    fn test_subdag_preserves_resource_access_mode() {
        // Inner DAG has a Write resource port
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![Port::resource(
                "file",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![Port::scalar("result", "String")],
            (),
        ));

        // SubDag auto-inference should preserve the Write mode
        let subdag_node = Node::subdag("wrapper", inner);
        let res_port = subdag_node
            .inputs
            .iter()
            .find(|p| p.name.0 == "res:file")
            .expect("res:file should be inferred");
        assert_eq!(
            res_port.resource_access,
            Some(AccessMode::Write),
            "SubDag auto-inference should preserve resource_access mode"
        );
    }

    #[test]
    fn test_derive_resource_accesses_respects_subdag_mode() {
        // Inner DAG with an Exclusive resource port
        let mut inner: Dag<()> = Dag::new();
        inner.add_node(Node::opaque(
            "worker",
            vec![Port::resource("lock", "Lock", AccessMode::Exclusive)],
            vec![Port::scalar("result", "String")],
            (),
        ));

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        let accesses = derive_resource_accesses(&dag).expect("resource accesses should derive");
        assert_eq!(accesses.len(), 1);
        assert_eq!(accesses[0].mode, AccessMode::Exclusive);
    }

    // ============ R2: Wildcard normalization and coarse conflict edge cases ============

    #[test]
    fn test_normalize_resource_id_various_wildcards() {
        // All wildcard forms must coarsen to "file".
        assert_eq!(normalize_resource_id("file:*"), "file");
        assert_eq!(normalize_resource_id("file:src/*"), "file");
        assert_eq!(normalize_resource_id("file:*.rs"), "file");
        assert_eq!(normalize_resource_id("file:src/main*"), "file");
        assert_eq!(normalize_resource_id("res:file:*"), "file");
        assert_eq!(normalize_resource_id("res:file:src/*"), "file");
    }

    #[test]
    fn test_normalize_resource_id_non_wildcard_preserved() {
        // Specific file paths must not be coarsened.
        assert_eq!(normalize_resource_id("file:Makefile"), "file:Makefile");
        assert_eq!(
            normalize_resource_id("file:src/main.rs"),
            "file:src/main.rs"
        );
        // Coarse `file` stays as `file`.
        assert_eq!(normalize_resource_id("file"), "file");
        // Non-file resources unaffected.
        assert_eq!(normalize_resource_id("api:network"), "api:network");
        assert_eq!(normalize_resource_id("tool:clippy"), "tool:clippy");
        assert_eq!(normalize_resource_id("repo"), "repo");
    }

    #[test]
    fn test_coarse_file_conflicts_with_multiple_specific_files() {
        let dag = parallel_dag();

        // Coarse `file` should conflict with any specific file path.
        let accesses = vec![
            ResourceAccess::write("a", "file"),
            ResourceAccess::write("b", "file:src/main.rs"),
        ];
        let conflicts = detect_conflicts(&dag, &accesses);
        assert_eq!(
            conflicts.len(),
            1,
            "coarse file must conflict with specific file path"
        );
    }

    #[test]
    fn test_two_specific_files_do_not_conflict() {
        let dag = parallel_dag();

        // Two different specific file paths should NOT conflict.
        let accesses = vec![
            ResourceAccess::write("a", "file:Makefile"),
            ResourceAccess::write("b", "file:Cargo.toml"),
        ];
        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(
            conflicts.is_empty(),
            "different specific file paths must not conflict"
        );
    }

    #[test]
    fn test_coarse_file_read_read_no_conflict() {
        let dag = parallel_dag();

        // Coarse file Read + specific file Read = no conflict.
        let accesses = vec![
            ResourceAccess::read("a", "file"),
            ResourceAccess::read("b", "file:Makefile"),
        ];
        let conflicts = detect_conflicts(&dag, &accesses);
        assert!(conflicts.is_empty(), "read-read should never conflict");
    }

    #[test]
    fn test_wildcard_port_construction_normalizes_in_dag_conflict_detection() {
        // Build a DAG where one node uses a wildcard-constructed port and
        // another uses a specific file — verify conflict detection sees them
        // as coarse vs specific and flags the conflict.
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(Node::opaque(
            "wildcard",
            vec![Port::resource(
                "file:*",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![],
            "op_a".to_string(),
        ));
        dag.add_node(Node::opaque(
            "specific",
            vec![Port::resource(
                "file:out.txt",
                "FilesystemHandle",
                AccessMode::Write,
            )],
            vec![],
            "op_b".to_string(),
        ));

        let conflicts = detect_resource_conflicts(&dag).expect("should derive accesses");
        assert_eq!(
            conflicts.len(),
            1,
            "normalized wildcard (coarse file) must conflict with specific file"
        );
        assert_eq!(conflicts[0].resource_id.0, "file");
    }

    #[test]
    fn test_resource_ids_conflict_symmetry() {
        let coarse = ResourceId::new("file");
        let specific = ResourceId::new("file:Makefile");

        // Conflict should be symmetric.
        assert!(resource_ids_conflict(&coarse, &specific));
        assert!(resource_ids_conflict(&specific, &coarse));
    }

    #[test]
    fn test_resource_ids_no_cross_kind_conflict() {
        // file vs api should never conflict.
        let file = ResourceId::new("file");
        let api = ResourceId::new("api:network");
        assert!(!resource_ids_conflict(&file, &api));

        // file vs tool should never conflict.
        let tool = ResourceId::new("tool:clippy");
        assert!(!resource_ids_conflict(&file, &tool));
    }

    // ============ M10: classify_effect + validate_resource_completeness ============

    #[test]
    fn test_classify_effect_transport_execution() {
        let node = Node::opaque(
            "execute_http",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            "op".to_string(),
        )
        .with_kind(NodeKind::TransportExecute);
        assert_eq!(classify_effect(&node), Some(NodeKind::TransportExecute));
    }

    #[test]
    fn test_classify_effect_tool_environment() {
        let node = Node::opaque(
            "tool_env",
            vec![],
            vec![Port::scalar("tool", "ToolHandle")],
            "op".to_string(),
        )
        .with_kind(NodeKind::ToolEnvironment);
        assert_eq!(classify_effect(&node), Some(NodeKind::ToolEnvironment));
    }

    #[test]
    fn test_classify_effect_resource_environment() {
        for type_id in &[
            "FilesystemHandle",
            "NetworkHandle",
            "Timestamp",
            "Credential",
            "Platform",
            "CloudSecretConfig",
        ] {
            let node = Node::opaque(
                "res_env",
                vec![],
                vec![Port::scalar("handle", *type_id)],
                "op".to_string(),
            )
            .with_kind(NodeKind::ResourceEnvironment);
            assert_eq!(
                classify_effect(&node),
                Some(NodeKind::ResourceEnvironment),
                "{type_id} should be classified as ResourceEnvironment"
            );
        }
    }

    #[test]
    fn test_classify_effect_tool_consumption() {
        let node = Node::opaque(
            "use_tool",
            vec![Port::scalar("tool", "ToolHandle")],
            vec![Port::scalar("result", "String")],
            "op".to_string(),
        )
        .with_kind(NodeKind::ToolConsumer);
        assert_eq!(classify_effect(&node), Some(NodeKind::ToolConsumer));
    }

    #[test]
    fn test_classify_effect_none_for_pure_node() {
        let node = Node::opaque(
            "pure",
            vec![Port::scalar("data", "String")],
            vec![Port::scalar("result", "String")],
            "op".to_string(),
        )
        .with_kind(NodeKind::Pure);
        assert_eq!(classify_effect(&node), None);
    }

    #[test]
    fn test_validate_resource_completeness_passes_for_declared() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(
            Node::opaque(
                "transport",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::resource("file", "FilesystemHandle", AccessMode::Write),
                ],
                vec![Port::scalar("response", "TransportResponse")],
                "op".to_string(),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let violations = validate_resource_completeness(&dag);
        assert!(violations.is_empty(), "properly declared node should pass");
    }

    #[test]
    fn test_validate_resource_completeness_and_conflicts_share_annotated_non_res_contract() {
        let shared_tool_port =
            Port::scalar("tool:clippy", "ToolHandle").with_resource_access(AccessMode::Exclusive);
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(
            Node::opaque(
                "tool_a",
                vec![shared_tool_port.clone()],
                vec![],
                "op_a".to_string(),
            )
            .with_kind(NodeKind::ToolEnvironment),
        );
        dag.add_node(
            Node::opaque("tool_b", vec![shared_tool_port], vec![], "op_b".to_string())
                .with_kind(NodeKind::ToolEnvironment),
        );

        let violations = validate_resource_completeness(&dag);
        assert!(
            violations.is_empty(),
            "resource_access should satisfy completeness even without a res: prefix"
        );

        let conflicts = detect_resource_conflicts(&dag)
            .expect("annotated non-res ports should participate in conflict detection");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].resource_id.0, "tool:clippy");
    }

    #[test]
    fn test_validate_resource_completeness_fails_for_undeclared() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(
            Node::opaque(
                "transport_no_res",
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                "op".to_string(),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let violations = validate_resource_completeness(&dag);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].node_id.0, "transport_no_res");
        assert_eq!(violations[0].effect_kind, NodeKind::TransportExecute);
    }

    #[test]
    fn test_validate_resource_completeness_ignores_pure_nodes() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(
            Node::opaque(
                "pure_transform",
                vec![Port::scalar("data", "String")],
                vec![Port::scalar("result", "String")],
                "op".to_string(),
            )
            .with_kind(NodeKind::Pure),
        );

        let violations = validate_resource_completeness(&dag);
        assert!(
            violations.is_empty(),
            "pure nodes should not require resource ports"
        );
    }

    #[test]
    fn test_validate_resource_completeness_recurses_into_subdags() {
        // Inner DAG has an effectful node without resource ports
        let mut inner: Dag<String> = Dag::new();
        inner.add_node(
            Node::opaque(
                "inner_transport",
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                "op".to_string(),
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let mut outer: Dag<String> = Dag::new();
        outer.add_node(Node::subdag("wrapper", inner));

        let violations = validate_resource_completeness(&outer);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].node_id.0, "inner_transport");
    }

    #[test]
    fn test_validate_resource_completeness_multiple_violations() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(
            Node::opaque(
                "transport_no_res",
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                "op_a".to_string(),
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "tool_env_no_res",
                vec![],
                vec![Port::scalar("tool", "ToolHandle")],
                "op_b".to_string(),
            )
            .with_kind(NodeKind::ToolEnvironment),
        );
        dag.add_node(
            Node::opaque(
                "pure_ok",
                vec![Port::scalar("x", "String")],
                vec![Port::scalar("y", "String")],
                "op_c".to_string(),
            )
            .with_kind(NodeKind::Pure),
        );

        let violations = validate_resource_completeness(&dag);
        assert_eq!(violations.len(), 2);
        let ids: Vec<&str> = violations.iter().map(|v| v.node_id.0.as_str()).collect();
        assert!(ids.contains(&"transport_no_res"));
        assert!(ids.contains(&"tool_env_no_res"));
    }

    #[test]
    fn test_missing_resource_declaration_labels() {
        let v = MissingResourceDeclaration {
            node_id: NodeId::from("n"),
            effect_kind: NodeKind::TransportExecute,
        };
        assert!(v.to_string().contains("transport execution"));
        let v2 = MissingResourceDeclaration {
            node_id: NodeId::from("n"),
            effect_kind: NodeKind::ToolEnvironment,
        };
        assert!(v2.to_string().contains("tool environment"));
        let v3 = MissingResourceDeclaration {
            node_id: NodeId::from("n"),
            effect_kind: NodeKind::ResourceEnvironment,
        };
        assert!(v3.to_string().contains("resource environment"));
        let v4 = MissingResourceDeclaration {
            node_id: NodeId::from("n"),
            effect_kind: NodeKind::ToolConsumer,
        };
        assert!(v4.to_string().contains("tool consumption"));
    }

    #[test]
    fn test_missing_resource_declaration_display() {
        let violation = MissingResourceDeclaration {
            node_id: NodeId::from("bad_node"),
            effect_kind: NodeKind::TransportExecute,
        };
        assert!(violation.to_string().contains("bad_node"));
        assert!(violation.to_string().contains("transport execution"));
    }

    #[test]
    fn test_classify_effect_priority_transport_over_tool() {
        // A node that has both TransportRequest input AND ToolHandle input
        // should classify as TransportExecution — NodeKind is the sole authority.
        let node = Node::opaque(
            "mixed",
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("tool", "ToolHandle"),
            ],
            vec![],
            "op".to_string(),
        )
        .with_kind(NodeKind::TransportExecute);
        assert_eq!(classify_effect(&node), Some(NodeKind::TransportExecute));
    }

    /// Regression: an effectful node that satisfies completeness (has a resource
    /// port) must also produce a ResourceAccess in derive_resource_accesses.
    /// Both functions use declared_resource_access() as the single contract;
    /// this test proves the coupling holds for every effectful NodeKind.
    #[test]
    fn test_effectful_node_satisfying_completeness_visible_to_conflict_derivation() {
        let effectful_kinds = [
            NodeKind::TransportExecute,
            NodeKind::TransportPrepare,
            NodeKind::TransportParse,
            NodeKind::ToolEnvironment,
            NodeKind::ToolConsumer,
            NodeKind::ResourceEnvironment,
            NodeKind::ResourceAcquire,
            NodeKind::ResourceRelease,
            NodeKind::ParamSource,
        ];

        for kind in &effectful_kinds {
            // Precondition: this kind is effectful.
            let probe = Node::opaque("probe", vec![], vec![], "op".to_string()).with_kind(*kind);
            assert!(
                classify_effect(&probe).is_some(),
                "{kind:?} must be classified as effectful"
            );

            // Build a single-node DAG with one resource port.
            let mut dag: Dag<String> = Dag::new();
            dag.add_node(
                Node::opaque(
                    "effectful",
                    vec![Port::resource("file", "FilesystemHandle", AccessMode::Write)],
                    vec![],
                    "op".to_string(),
                )
                .with_kind(*kind),
            );

            // Completeness must pass — the node declares a resource port.
            let violations = validate_resource_completeness(&dag);
            assert!(
                violations.is_empty(),
                "{kind:?}: node with resource port should satisfy completeness"
            );

            // Conflict derivation must see the same port — no invisible gap.
            let accesses = derive_resource_accesses(&dag).expect("should derive");
            assert!(
                accesses.iter().any(|a| a.node_id.0 == "effectful"),
                "{kind:?}: node satisfying completeness must be visible to conflict derivation"
            );
        }
    }
}
