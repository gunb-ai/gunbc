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
//! ```ignore
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
pub use handle::{HandleParseError, ResourceHandle};
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

use crate::dag::Dag;
use crate::types::NodeId;
use crate::{SecretString, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
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
/// Canonical repository resource port.
pub const RESOURCE_REPO: &str = "res:repo";
/// Canonical coarse target resource port.
pub const RESOURCE_TARGET: &str = "res:target";
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
    id.strip_prefix(RESOURCE_PORT_PREFIX)
        .unwrap_or(id)
        .to_string()
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
        Some(Value::Secret(s)) if s.expose() == CAPABILITY_MARKER => Ok(()),
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

    // Build a map of resource → accesses
    let mut resource_accesses: HashMap<&ResourceId, Vec<&ResourceAccess>> = HashMap::new();
    for access in accesses {
        resource_accesses
            .entry(&access.resource_id)
            .or_default()
            .push(access);
    }

    // Build a set of ordered pairs (nodes where one must execute before the other)
    let ordered_pairs = compute_ordered_pairs(dag);

    // Check each resource for conflicts
    for (resource_id, accesses) in resource_accesses {
        // Check all pairs of accesses to this resource
        for i in 0..accesses.len() {
            for j in (i + 1)..accesses.len() {
                let access_a = accesses[i];
                let access_b = accesses[j];

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
                        resource_id: resource_id.clone(),
                        mode_a: access_a.mode,
                        mode_b: access_b.mode,
                    });
                }
            }
        }
    }

    conflicts
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

/// Derive resource accesses from `res:*` input ports in a DAG.
///
/// Walks all nodes in the DAG and extracts `ResourceAccess` entries from
/// input ports whose names start with `res:`. Requires `port.resource_access`
/// to be explicitly set on all `res:*` ports.
pub fn derive_resource_accesses<T>(
    dag: &Dag<T>,
) -> Result<Vec<ResourceAccess>, Vec<ResourceAccessError>> {
    let mut accesses = Vec::new();
    let mut errors = Vec::new();
    for node in &dag.nodes {
        for port in &node.inputs {
            if let Some(res_name) = port.name.0.strip_prefix("res:") {
                let resource_id = ResourceId::new(normalize_resource_id(res_name));
                match port.resource_access {
                    Some(mode) => {
                        accesses.push(ResourceAccess::new(node.id.clone(), resource_id, mode))
                    }
                    None => errors.push(ResourceAccessError {
                        node_id: node.id.clone(),
                        port_name: port.name.0.clone(),
                        resource_id,
                    }),
                }
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
}
