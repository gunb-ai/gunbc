//! daglang-lower: Lowers typed .dag AST to gunbc GraphIR.
//!
//! Transforms the high-level typed AST into gunbc's existing IR types
//! (`Dag`, `Node`, `Port`, `Edge`). This is where:
//!
//! - Pattern expansion happens (`content_upsert` → read/compare/write chain)
//! - Service calls become transport triplets (prepare/execute/parse)
//! - Resource `acquire` blocks become acquisition DAG nodes
//! - `fn` body collection ops (`map`, `filter`, `fold`) become IR-level
//!   `MapNode`, `FilterNode`, `FoldNode` for data-parallel execution
//! - `interface` resolution replaces abstract types with concrete resources
//!
//! # Pipeline position
//!
//! ```text
//! TypedAST → [daglang-lower] → GraphIR (gunbc Dag/Node/Port/Edge)
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use daglang_syntax::ast::{Annotation, Expr, Item, Literal, OperationDef, ServiceDef, Stmt};
use daglang_syntax::ast_utils::{
    canonical_resource_type_name as canonical_type_name, resource_type_name,
    service_call_lookup_keys, should_track_call_name as should_track_call, type_expr_to_string,
    walk_stmts,
};
use daglang_typecheck::{TypedCallableSignature, TypedItemSignature, TypedProject};
use gunbc_ir::{Cardinality, Dag, Edge, Node, Port};
use serde::Serialize;

/// Lowered operation payload for daglang graph nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredOp {
    Callable {
        module: String,
        kind: CallableKind,
        name: String,
        obligation: ObligationCategory,
        service_metadata: Option<ServiceCallMetadata>,
    },
    Collection {
        module: String,
        callable: String,
        kind: CollectionOpKind,
    },
    Pipeline {
        module: String,
        name: String,
        stages: usize,
        stage_names: Vec<String>,
    },
}

/// Structural parity report between two DAGs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityReport {
    pub candidate_nodes: usize,
    pub reference_nodes: usize,
    pub candidate_edges: usize,
    pub reference_edges: usize,
    pub added_nodes: usize,
    pub removed_nodes: usize,
    pub changed_nodes: usize,
    pub added_edges: usize,
    pub removed_edges: usize,
    pub added_node_ids: Vec<String>,
    pub removed_node_ids: Vec<String>,
    pub changed_node_details: Vec<NodeDiff>,
    pub added_edge_ids: Vec<String>,
    pub removed_edge_ids: Vec<String>,
}

impl ParityReport {
    pub fn is_exact_match(&self) -> bool {
        self.added_nodes == 0
            && self.removed_nodes == 0
            && self.changed_nodes == 0
            && self.added_edges == 0
            && self.removed_edges == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDiff {
    pub node_id: String,
    pub differences: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalDag {
    pub nodes: Vec<CanonicalNode>,
    pub edges: Vec<CanonicalEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub inputs: Vec<CanonicalPort>,
    pub outputs: Vec<CanonicalPort>,
    pub subdag: Option<Box<CanonicalDag>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalPort {
    pub name: String,
    pub type_id: String,
    pub cardinality: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CanonicalEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

/// Kind of lowered callable declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableKind {
    Fn,
    Func,
    Pattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationCategory {
    None,
    ServiceTransportPrepare,
    ServiceTransportExecute,
    ServiceTransportParse,
    ServiceParamSource,
    ResourceProvide,
    ResourceAcquire,
    ResourceRelease,
    InterfaceContractVerification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTransportClass {
    Unknown,
    ShellLocal,
    RestNetwork,
    FileBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ServiceCallMetadata {
    pub service: String,
    pub operation: String,
    pub transport: ServiceTransportClass,
    pub idempotent: bool,
    pub readonly: bool,
    pub permissions: Vec<String>,
}

impl LoweredOp {
    pub fn obligation_category(&self) -> ObligationCategory {
        match self {
            Self::Callable { obligation, .. } => *obligation,
            Self::Collection { .. } | Self::Pipeline { .. } => ObligationCategory::None,
        }
    }

    pub fn service_call_metadata(&self) -> Option<&ServiceCallMetadata> {
        match self {
            Self::Callable {
                service_metadata, ..
            } => service_metadata.as_ref(),
            Self::Collection { .. } | Self::Pipeline { .. } => None,
        }
    }
}

pub fn classify_obligation(op: &LoweredOp) -> ObligationCategory {
    op.obligation_category()
}

pub fn classify_service_transport(op: &LoweredOp) -> Option<ServiceTransportClass> {
    op.service_call_metadata()
        .map(|metadata| metadata.transport)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoweredEndpoint {
    node_id: String,
    primary_output: String,
}

#[derive(Debug, Clone)]
struct EndpointRegistry<T> {
    by_key: HashMap<String, Option<T>>,
}

impl<T> Default for EndpointRegistry<T> {
    fn default() -> Self {
        Self {
            by_key: HashMap::new(),
        }
    }
}

impl<T: PartialEq> EndpointRegistry<T> {
    fn register(&mut self, key: String, endpoint: T) {
        self.by_key
            .entry(key)
            .and_modify(|existing| {
                if let Some(current) = existing {
                    if current != &endpoint {
                        *existing = None;
                    }
                }
            })
            .or_insert(Some(endpoint));
    }
}

type ServiceEndpointRegistry = EndpointRegistry<ServiceTransportEndpoint>;
type ResourceLifecycleRegistry = EndpointRegistry<ResourceLifecycleEndpoint>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceTransportEndpoint {
    parse: LoweredEndpoint,
    prepare_node_id: String,
    prepare_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceLifecycleEndpoint {
    acquire_node: Option<String>,
    release_node: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderHint {
    Gcp,
    Aws,
    Azure,
}

fn provider_hint_from_resource_type(resource_type: &str) -> Option<ProviderHint> {
    provider_hint_from_name(resource_type)
}

fn provider_hint_from_name(name: &str) -> Option<ProviderHint> {
    if name.contains("GcpConfig") {
        return Some(ProviderHint::Gcp);
    }
    if name.contains("AwsConfig") {
        return Some(ProviderHint::Aws);
    }
    if name.contains("AzureConfig") {
        return Some(ProviderHint::Azure);
    }
    None
}

fn provider_hint_from_expr(expr: &Expr) -> Option<ProviderHint> {
    match expr {
        Expr::Ident(name) | Expr::Call(name, _) => provider_hint_from_name(name),
        Expr::Record(name, _) => name.as_deref().and_then(provider_hint_from_name),
        Expr::FieldAccess(_, field) => provider_hint_from_name(field),
        _ => None,
    }
}

fn provider_hint_from_uses_config(config: Option<&[(String, Expr)]>) -> Option<ProviderHint> {
    let Some(config_entries) = config else {
        return None;
    };
    for (name, value) in config_entries {
        if name == "cloud" {
            if let Some(provider_hint) = provider_hint_from_expr(value) {
                return Some(provider_hint);
            }
        }
    }
    None
}

fn provider_hint_from_module_name(module_name: &str) -> Option<ProviderHint> {
    let normalized = format!(".{}.", module_name);
    if normalized.contains(".gcp.") {
        return Some(ProviderHint::Gcp);
    }
    if normalized.contains(".aws.") {
        return Some(ProviderHint::Aws);
    }
    if normalized.contains(".azure.") {
        return Some(ProviderHint::Azure);
    }
    None
}

fn provider_hint_from_resource_name(resource_name: &str) -> Option<ProviderHint> {
    let lower = resource_name.to_ascii_lowercase();
    if lower.contains("gcs") || lower.contains("gcp") {
        return Some(ProviderHint::Gcp);
    }
    if lower.contains("s3") || lower.contains("aws") {
        return Some(ProviderHint::Aws);
    }
    if lower.contains("blob") || lower.contains("azure") {
        return Some(ProviderHint::Azure);
    }
    None
}

fn insert_canonical_names(set: &mut HashSet<String>, name: &str) {
    let canonical = canonical_type_name(name);
    let short = canonical
        .rsplit('.')
        .next()
        .unwrap_or(canonical.as_str())
        .to_string();
    set.insert(canonical);
    set.insert(short);
}

fn is_known_uses_type(set: &HashSet<String>, name: &str) -> bool {
    let canonical = canonical_type_name(name);
    set.contains(&canonical)
        || set.contains(canonical.rsplit('.').next().unwrap_or(canonical.as_str()))
}

/// Wraps a `Dag` with O(1) deduplication tracking for nodes and edges.
struct DagBuilder {
    dag: Dag<LoweredOp>,
    seen_nodes: HashSet<String>,
    seen_edges: HashSet<(String, String, String, String)>,
}

impl DagBuilder {
    fn new() -> Self {
        Self {
            dag: Dag::new(),
            seen_nodes: HashSet::new(),
            seen_edges: HashSet::new(),
        }
    }

    fn add_node(&mut self, node: Node<LoweredOp>) {
        let node_id = node.id.0.clone();
        if self.seen_nodes.insert(node_id) {
            self.dag.add_node(node);
        }
    }

    fn add_edge(&mut self, from: &str, from_port: &str, to: &str, to_port: &str) {
        let key = (
            from.to_string(),
            from_port.to_string(),
            to.to_string(),
            to_port.to_string(),
        );
        if self.seen_edges.insert(key) {
            self.dag.add_edge(Edge::new(from, from_port, to, to_port));
        }
    }

    fn has_node(&self, id: &str) -> bool {
        self.seen_nodes.contains(id)
    }

    fn into_dag(self) -> Dag<LoweredOp> {
        self.dag
    }
}

/// Errors during lowering.
#[derive(Debug)]
pub enum LowerError {
    /// A pattern could not be expanded (e.g., unknown pattern name).
    UnknownPattern(String),
    /// A service operation has no transport annotation.
    MissingTransport { service: String, operation: String },
    /// Resource acquire block is malformed.
    InvalidAcquireBlock { resource: String, reason: String },
    /// Interface could not be resolved to a concrete resource.
    UnresolvedInterface { interface: String },
    /// Service call in a callable body could not be resolved to a transport endpoint.
    UnresolvedServiceCall {
        caller: String,
        service_call: String,
    },
    /// `uses` clause references an unknown resource/interface lifecycle source.
    UnresolvedUsedResource {
        caller: String,
        binding: String,
        resource_type: String,
    },
    /// `provides` clause references an unknown resource/interface source.
    UnresolvedProvidedResource {
        caller: String,
        binding: String,
        resource_type: String,
    },
    /// No executable declarations were available for lowering.
    NoLowerableItems,
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPattern(name) => write!(f, "unknown pattern `{name}`"),
            Self::MissingTransport { service, operation } => {
                write!(
                    f,
                    "service `{service}` operation `{operation}` has no transport annotation"
                )
            }
            Self::InvalidAcquireBlock { resource, reason } => {
                write!(f, "invalid acquire block for `{resource}`: {reason}")
            }
            Self::UnresolvedInterface { interface } => {
                write!(f, "unresolved interface `{interface}`")
            }
            Self::UnresolvedServiceCall {
                caller,
                service_call,
            } => write!(f, "unresolved service call `{service_call}` in `{caller}`"),
            Self::UnresolvedUsedResource {
                caller,
                binding,
                resource_type,
            } => write!(
                f,
                "unresolved used resource `{binding}: {resource_type}` in `{caller}`"
            ),
            Self::UnresolvedProvidedResource {
                caller,
                binding,
                resource_type,
            } => write!(
                f,
                "unresolved provided resource `{binding}: {resource_type}` in `{caller}`"
            ),
            Self::NoLowerableItems => write!(f, "no callable or pipeline declarations to lower"),
        }
    }
}

/// Lower a typed project into a structural GraphIR DAG.
///
/// Phase-1 lowering focuses on callable and pipeline signatures:
/// - `fn` / `func` / `pattern` become opaque callable nodes
/// - `pipeline` declarations become opaque pipeline nodes
/// - type/service/resource/interface declarations remain metadata and are not
///   lowered into executable graph nodes yet.
pub fn lower_typed_project(project: &TypedProject) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, false)
}

/// Lowers typed modules while emitting explicit collection pipeline nodes.
pub fn lower_typed_project_with_collection_nodes(
    project: &TypedProject,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, true)
}

pub fn lower_typed_project_for_modules(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), false)
}

/// Lowers only scoped modules while emitting explicit collection pipeline nodes.
pub fn lower_typed_project_for_modules_with_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), true)
}

fn lower_typed_project_with_callable_scope(
    project: &TypedProject,
    callable_modules: Option<&HashSet<String>>,
    emit_collection_nodes: bool,
) -> Result<Dag<LoweredOp>, LowerError> {
    let mut builder = DagBuilder::new();
    let mut endpoints_by_full = HashMap::<(String, String), LoweredEndpoint>::new();
    let mut endpoints_by_name = HashMap::<String, Option<LoweredEndpoint>>::new();

    for module in &project.modules {
        let module_name = module.module_path.join(".");
        let include_callables = callable_modules
            .map(|scope| scope.contains(&module_name))
            .unwrap_or(true);
        for signature in &module.signatures {
            match signature {
                TypedItemSignature::Fn(callable) => {
                    if !include_callables {
                        continue;
                    }
                    let (node, endpoint) = lower_callable(callable, &module_name, CallableKind::Fn);
                    register_endpoint(
                        &mut endpoints_by_full,
                        &mut endpoints_by_name,
                        &module_name,
                        &callable.name,
                        endpoint,
                    );
                    builder.add_node(node);
                }
                TypedItemSignature::Func(callable) => {
                    if !include_callables {
                        continue;
                    }
                    let (node, endpoint) =
                        lower_callable(callable, &module_name, CallableKind::Func);
                    register_endpoint(
                        &mut endpoints_by_full,
                        &mut endpoints_by_name,
                        &module_name,
                        &callable.name,
                        endpoint,
                    );
                    builder.add_node(node);
                }
                TypedItemSignature::Pattern(callable) => {
                    if !include_callables {
                        continue;
                    }
                    let (node, endpoint) =
                        lower_callable(callable, &module_name, CallableKind::Pattern);
                    register_endpoint(
                        &mut endpoints_by_full,
                        &mut endpoints_by_name,
                        &module_name,
                        &callable.name,
                        endpoint,
                    );
                    builder.add_node(node);
                }
                TypedItemSignature::Pipeline {
                    name,
                    stages,
                    stage_names,
                } => {
                    if !include_callables {
                        continue;
                    }
                    let node_id = lowered_node_id(&module_name, name);
                    builder.add_node(Node::opaque(
                        node_id,
                        vec![],
                        vec![Port::with_cardinality("stages", "Int", Cardinality::ONE)],
                        LoweredOp::Pipeline {
                            module: module_name.clone(),
                            name: name.clone(),
                            stages: *stages,
                            stage_names: stage_names.clone(),
                        },
                    ));
                }
                TypedItemSignature::Type { .. }
                | TypedItemSignature::Service { .. }
                | TypedItemSignature::Resource { .. }
                | TypedItemSignature::Interface { .. } => {}
            }
        }
    }

    add_dependency_edges(
        &mut builder,
        project,
        &endpoints_by_full,
        &endpoints_by_name,
        emit_collection_nodes,
    );
    add_makegen_scaffolding(&mut builder, &endpoints_by_full);
    let service_registry = if callable_modules.is_some() {
        let required_service_calls = collect_required_service_call_keys(project, callable_modules);
        add_service_transport_triplets(&mut builder, project, Some(&required_service_calls))
    } else {
        add_service_transport_triplets(&mut builder, project, None)
    };
    add_service_call_edges(&mut builder, project, &endpoints_by_full, &service_registry)?;
    let resource_registry = add_resource_lifecycle_nodes(&mut builder, project, callable_modules);
    let known_uses_types = collect_known_uses_types(project);
    add_used_resource_edges(
        &mut builder,
        project,
        &endpoints_by_full,
        &resource_registry,
        &known_uses_types,
    )?;
    add_provided_resource_nodes(
        &mut builder,
        project,
        &endpoints_by_full,
        &resource_registry,
        &known_uses_types,
    )?;
    add_interface_contract_verification_nodes(&mut builder, project, &resource_registry);

    if builder.dag.nodes.is_empty() {
        return Err(LowerError::NoLowerableItems);
    }

    Ok(builder.into_dag())
}

/// Compare a lowered daglang graph against a reference graph topology.
///
/// This enables incremental parity harness adoption:
/// - exact parity: `report.is_exact_match() == true`
/// - scaffold mode: report still gives deterministic deltas while lowering
///   coverage grows.
pub fn compare_topology<T>(candidate: &Dag<LoweredOp>, reference: &Dag<T>) -> ParityReport {
    compare_ir(candidate, reference)
}

/// Compare a lowered daglang graph against a reference graph using canonical
/// structural IR (node kind, labels, ports, edges, and nested subdag shape).
pub fn compare_ir<T>(candidate: &Dag<LoweredOp>, reference: &Dag<T>) -> ParityReport {
    let candidate_ir = canonicalize_lowered_ir(candidate);
    let reference_ir = canonicalize_reference_ir(reference);
    compare_canonical_ir(&candidate_ir, &reference_ir)
}

/// Serialize lowered IR into deterministic canonical JSON.
pub fn canonical_ir_json(dag: &Dag<LoweredOp>) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&canonicalize_lowered_ir(dag))
}

fn compare_canonical_ir(candidate: &CanonicalDag, reference: &CanonicalDag) -> ParityReport {
    let candidate_nodes = candidate.nodes.len();
    let reference_nodes = reference.nodes.len();
    let candidate_edges = candidate.edges.len();
    let reference_edges = reference.edges.len();

    let candidate_by_id = candidate
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let reference_by_id = reference
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();

    let candidate_ids = candidate_by_id.keys().cloned().collect::<BTreeSet<_>>();
    let reference_ids = reference_by_id.keys().cloned().collect::<BTreeSet<_>>();

    let added_node_ids = candidate_ids
        .difference(&reference_ids)
        .cloned()
        .collect::<Vec<_>>();
    let removed_node_ids = reference_ids
        .difference(&candidate_ids)
        .cloned()
        .collect::<Vec<_>>();

    let changed_node_details = candidate_ids
        .intersection(&reference_ids)
        .filter_map(|node_id| {
            let candidate_node = candidate_by_id
                .get(node_id)
                .expect("intersection node must exist in candidate map");
            let reference_node = reference_by_id
                .get(node_id)
                .expect("intersection node must exist in reference map");
            let mut differences = Vec::new();
            if candidate_node.kind != reference_node.kind {
                differences.push(format!(
                    "kind differs: candidate=`{}` reference=`{}`",
                    candidate_node.kind, reference_node.kind
                ));
            }
            if candidate_node.label != reference_node.label {
                differences.push(format!(
                    "label differs: candidate=`{}` reference=`{}`",
                    candidate_node.label, reference_node.label
                ));
            }
            if candidate_node.inputs != reference_node.inputs {
                differences.push("input ports differ".to_string());
            }
            if candidate_node.outputs != reference_node.outputs {
                differences.push("output ports differ".to_string());
            }
            if candidate_node.subdag != reference_node.subdag {
                differences.push("subdag structure differs".to_string());
            }
            (!differences.is_empty()).then_some(NodeDiff {
                node_id: node_id.clone(),
                differences,
            })
        })
        .collect::<Vec<_>>();

    let candidate_edge_ids = candidate
        .edges
        .iter()
        .map(canonical_edge_id)
        .collect::<BTreeSet<_>>();
    let reference_edge_ids = reference
        .edges
        .iter()
        .map(canonical_edge_id)
        .collect::<BTreeSet<_>>();

    let added_edge_ids = candidate_edge_ids
        .difference(&reference_edge_ids)
        .cloned()
        .collect::<Vec<_>>();
    let removed_edge_ids = reference_edge_ids
        .difference(&candidate_edge_ids)
        .cloned()
        .collect::<Vec<_>>();

    ParityReport {
        candidate_nodes,
        reference_nodes,
        candidate_edges,
        reference_edges,
        added_nodes: added_node_ids.len(),
        removed_nodes: removed_node_ids.len(),
        changed_nodes: changed_node_details.len(),
        added_edges: added_edge_ids.len(),
        removed_edges: removed_edge_ids.len(),
        added_node_ids,
        removed_node_ids,
        changed_node_details,
        added_edge_ids,
        removed_edge_ids,
    }
}

/// Compare makegen topology with normalization rules for known scaffold deltas.
///
/// Normalization currently:
/// - strips `tools.makegen::` node-id prefixes
/// - drops the wrapper `makegen` callable node
/// - removes synthetic `__deps` ports/edges
/// - canonicalizes `render_makefile.return` output to `makefile_content`
/// - ignores environment-scaffold edge `fs_env -> prepare_read_makegen`
pub fn compare_makegen_topology<T>(candidate: &Dag<LoweredOp>, reference: &Dag<T>) -> ParityReport {
    let normalized = normalize_makegen_candidate(candidate);
    let normalized_reference = normalize_makegen_reference(reference);
    compare_topology(&normalized, &normalized_reference)
}

fn canonicalize_lowered_ir(dag: &Dag<LoweredOp>) -> CanonicalDag {
    canonicalize_dag(dag, canonical_kind_lowered, canonical_label_lowered)
}

fn canonicalize_reference_ir<T>(dag: &Dag<T>) -> CanonicalDag {
    canonicalize_dag(dag, canonical_kind_reference, canonical_label_reference)
}

fn canonicalize_dag<T>(
    dag: &Dag<T>,
    kind_of: fn(&Node<T>) -> String,
    label_of: fn(&Node<T>) -> String,
) -> CanonicalDag {
    let mut nodes = dag
        .nodes
        .iter()
        .map(|node| CanonicalNode {
            id: node.id.0.clone(),
            kind: kind_of(node),
            label: label_of(node),
            inputs: canonicalize_ports(&node.inputs),
            outputs: canonicalize_ports(&node.outputs),
            subdag: match &node.body {
                gunbc_ir::node::NodeBody::SubDag(inner) => {
                    Some(Box::new(canonicalize_dag(inner, kind_of, label_of)))
                }
                gunbc_ir::node::NodeBody::Opaque(_) => None,
            },
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

    let mut edges = dag
        .edges
        .iter()
        .map(|edge| CanonicalEdge {
            from_node: edge.from_node.0.clone(),
            from_port: edge.from_port.0.clone(),
            to_node: edge.to_node.0.clone(),
            to_port: edge.to_port.0.clone(),
        })
        .collect::<Vec<_>>();
    edges.sort();

    CanonicalDag { nodes, edges }
}

fn canonicalize_ports(ports: &[Port]) -> Vec<CanonicalPort> {
    let mut canonical = ports
        .iter()
        .map(|port| CanonicalPort {
            name: port.name.0.clone(),
            type_id: port.type_id.0.clone(),
            cardinality: port.cardinality.to_string(),
        })
        .collect::<Vec<_>>();
    canonical.sort_by(|lhs, rhs| {
        lhs.name
            .cmp(&rhs.name)
            .then_with(|| lhs.type_id.cmp(&rhs.type_id))
            .then_with(|| lhs.cardinality.cmp(&rhs.cardinality))
    });
    canonical
}

fn canonical_kind_lowered(node: &Node<LoweredOp>) -> String {
    match &node.body {
        gunbc_ir::node::NodeBody::SubDag(_) => "subdag".to_string(),
        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline { .. }) => {
            canonical_kind_from_shape(&node.id.0, &node.inputs, &node.outputs, true)
        }
        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection { kind, .. }) => {
            collection_kind_node_label(*kind).to_string()
        }
        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable { .. }) => {
            canonical_kind_from_shape(&node.id.0, &node.inputs, &node.outputs, false)
        }
    }
}

fn canonical_label_lowered(node: &Node<LoweredOp>) -> String {
    node.id.0.clone()
}

fn canonical_kind_reference<T>(node: &Node<T>) -> String {
    match &node.body {
        gunbc_ir::node::NodeBody::SubDag(_) => "subdag".to_string(),
        gunbc_ir::node::NodeBody::Opaque(_) => {
            canonical_kind_from_shape(&node.id.0, &node.inputs, &node.outputs, false)
        }
    }
}

fn canonical_label_reference<T>(node: &Node<T>) -> String {
    node.id.0.clone()
}

fn canonical_edge_id(edge: &CanonicalEdge) -> String {
    format!(
        "{}.{}->{}.{}",
        edge.from_node, edge.from_port, edge.to_node, edge.to_port
    )
}

fn canonical_kind_from_shape(
    node_id: &str,
    inputs: &[Port],
    outputs: &[Port],
    pipeline_hint: bool,
) -> String {
    if pipeline_hint
        || outputs
            .iter()
            .any(|port| port.name.0 == "stages" && port.type_id.0 == "Int")
    {
        return "pipeline".to_string();
    }
    if inputs
        .iter()
        .any(|port| port.type_id.0 == "TransportRequest")
    {
        return "transport".to_string();
    }
    let looks_expanded = node_id.starts_with("prepare_")
        || node_id.starts_with("compare_")
        || node_id.starts_with("execute_transport_")
        || node_id.starts_with("param_source_")
        || node_id.starts_with("acquire_resource_")
        || node_id.starts_with("release_resource_")
        || node_id.starts_with("provide_resource_")
        || node_id == "load_registry"
        || node_id == "fs_env";
    if looks_expanded {
        return "pattern-expanded".to_string();
    }
    "callable".to_string()
}

fn normalize_makegen_candidate(candidate: &Dag<LoweredOp>) -> Dag<LoweredOp> {
    let mut normalized = Dag::new();
    let mut kept_nodes = HashSet::<String>::new();
    let mut ports_by_node = HashMap::<String, (HashSet<String>, HashSet<String>)>::new();

    for node in &candidate.nodes {
        let Some(op) = node_body_as_opaque(&node.body).cloned() else {
            continue;
        };
        let canonical_id = canonical_makegen_node_id(&node.id.0);
        if canonical_id == "makegen" {
            continue;
        }

        let mut inputs = node
            .inputs
            .iter()
            .filter(|port| port.name.0 != "__deps")
            .cloned()
            .collect::<Vec<_>>();
        let mut outputs = node.outputs.clone();
        normalize_makegen_ports(&canonical_id, &mut inputs, &mut outputs);

        normalized.add_node(Node::opaque(canonical_id.clone(), inputs, outputs, op));
        kept_nodes.insert(canonical_id);
    }
    for node in &normalized.nodes {
        ports_by_node.insert(
            node.id.0.clone(),
            (
                node.inputs.iter().map(|port| port.name.0.clone()).collect(),
                node.outputs
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect(),
            ),
        );
    }

    let mut seen_edges = HashSet::<(String, String, String, String)>::new();
    for edge in &candidate.edges {
        let from_node = canonical_makegen_node_id(&edge.from_node.0);
        let to_node = canonical_makegen_node_id(&edge.to_node.0);
        if from_node == "makegen" || to_node == "makegen" {
            continue;
        }
        if !kept_nodes.contains(&from_node) || !kept_nodes.contains(&to_node) {
            continue;
        }
        if edge.from_port.0 == "__deps" || edge.to_port.0 == "__deps" {
            continue;
        }
        let from_port = canonical_makegen_port_name(&from_node, &edge.from_port.0);
        let to_port = canonical_makegen_port_name(&to_node, &edge.to_port.0);
        let Some((to_inputs, _)) = ports_by_node.get(&to_node) else {
            continue;
        };
        let Some((_, from_outputs)) = ports_by_node.get(&from_node) else {
            continue;
        };
        if !from_outputs.contains(&from_port) || !to_inputs.contains(&to_port) {
            continue;
        }
        let key = (from_node, from_port, to_node, to_port);
        if seen_edges.insert(key.clone()) {
            normalized.add_edge(Edge::new(key.0, key.1, key.2, key.3));
        }
    }

    normalized
}

fn normalize_makegen_reference<T>(reference: &Dag<T>) -> Dag<()> {
    let mut normalized = Dag::new();
    let mut kept_nodes = HashSet::<String>::new();
    let mut ports_by_node = HashMap::<String, (HashSet<String>, HashSet<String>)>::new();

    for node in &reference.nodes {
        let canonical_id = canonical_makegen_node_id(&node.id.0);
        if canonical_id == "makegen" {
            continue;
        }
        let mut inputs = node
            .inputs
            .iter()
            .filter(|port| port.name.0 != "__deps")
            .cloned()
            .collect::<Vec<_>>();
        let mut outputs = node.outputs.clone();
        normalize_makegen_ports(&canonical_id, &mut inputs, &mut outputs);
        normalized.add_node(Node::opaque(canonical_id.clone(), inputs, outputs, ()));
        kept_nodes.insert(canonical_id);
    }
    for node in &normalized.nodes {
        ports_by_node.insert(
            node.id.0.clone(),
            (
                node.inputs.iter().map(|port| port.name.0.clone()).collect(),
                node.outputs
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect(),
            ),
        );
    }

    let mut seen_edges = HashSet::<(String, String, String, String)>::new();
    for edge in &reference.edges {
        let from_node = canonical_makegen_node_id(&edge.from_node.0);
        let to_node = canonical_makegen_node_id(&edge.to_node.0);
        if from_node == "makegen" || to_node == "makegen" {
            continue;
        }
        if !kept_nodes.contains(&from_node) || !kept_nodes.contains(&to_node) {
            continue;
        }
        if edge.from_port.0 == "__deps" || edge.to_port.0 == "__deps" {
            continue;
        }
        let from_port = canonical_makegen_port_name(&from_node, &edge.from_port.0);
        let to_port = canonical_makegen_port_name(&to_node, &edge.to_port.0);
        let Some((to_inputs, _)) = ports_by_node.get(&to_node) else {
            continue;
        };
        let Some((_, from_outputs)) = ports_by_node.get(&from_node) else {
            continue;
        };
        if !from_outputs.contains(&from_port) || !to_inputs.contains(&to_port) {
            continue;
        }
        let key = (from_node, from_port, to_node, to_port);
        if seen_edges.insert(key.clone()) {
            normalized.add_edge(Edge::new(key.0, key.1, key.2, key.3));
        }
    }

    normalized
}

fn normalize_makegen_ports(node_id: &str, inputs: &mut Vec<Port>, outputs: &mut Vec<Port>) {
    match node_id {
        "fs_env" => {
            outputs
                .retain(|port| matches!(port.name.0.as_str(), "FilesystemHandle" | "file:write"));
            for output in outputs.iter_mut() {
                if output.name.0 == "file:write" {
                    output.name.0 = "FilesystemHandle".to_string();
                }
            }
        }
        "load_registry" => {
            outputs.retain(|port| port.name.0 == "registry");
            for output in outputs.iter_mut() {
                output.type_id.0 = "Json".to_string();
            }
        }
        "render_makefile" => {
            inputs.retain(|port| port.name.0 == "registry");
            for input in inputs.iter_mut() {
                input.type_id.0 = "Json".to_string();
            }
            for output in outputs.iter_mut() {
                if output.name.0 == "return" {
                    output.name.0 = "makefile_content".to_string();
                }
            }
            outputs.retain(|port| port.name.0 == "makefile_content");
        }
        "prepare_read_makegen" => {
            inputs.retain(|port| port.name.0 == "path");
            outputs.retain(|port| port.name.0 == "request");
        }
        "execute_read_makegen" => {
            inputs.retain(|port| port.name.0 == "request");
            outputs.retain(|port| port.name.0 == "response");
        }
        "compare_makegen_content" => {
            inputs.retain(|port| matches!(port.name.0.as_str(), "expected_content" | "response"));
            outputs.retain(|port| matches!(port.name.0.as_str(), "fresh" | "skip"));
        }
        "prepare_write_makegen" => {
            inputs.retain(|port| matches!(port.name.0.as_str(), "content" | "path"));
            outputs.retain(|port| port.name.0 == "request");
        }
        "execute_makegen_transport" => {
            inputs.retain(|port| matches!(port.name.0.as_str(), "request" | "skip"));
            outputs.retain(|port| port.name.0 == "makegen_response");
            for output in outputs.iter_mut() {
                output.cardinality = Cardinality::ZERO_OR_ONE;
            }
        }
        _ => {}
    }
    inputs.sort_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));
    dedup_ports_by_name_type_cardinality(inputs);
    outputs.sort_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));
    dedup_ports_by_name_type_cardinality(outputs);
}

fn dedup_ports_by_name_type_cardinality(ports: &mut Vec<Port>) {
    ports.dedup_by(|lhs, rhs| {
        lhs.name == rhs.name && lhs.type_id == rhs.type_id && lhs.cardinality == rhs.cardinality
    });
}

fn canonical_makegen_port_name(node_id: &str, port_name: &str) -> String {
    if node_id == "render_makefile" && port_name == "return" {
        return "makefile_content".to_string();
    }
    if node_id == "fs_env" && port_name == "file:write" {
        return "FilesystemHandle".to_string();
    }
    port_name.to_string()
}

fn canonical_makegen_node_id(node_id: &str) -> String {
    node_id
        .strip_prefix("tools.makegen::")
        .unwrap_or(node_id)
        .to_string()
}

fn node_body_as_opaque(body: &gunbc_ir::node::NodeBody<LoweredOp>) -> Option<&LoweredOp> {
    match body {
        gunbc_ir::node::NodeBody::Opaque(op) => Some(op),
        gunbc_ir::node::NodeBody::SubDag(_) => None,
    }
}

fn lower_callable(
    callable: &TypedCallableSignature,
    module_name: &str,
    kind: CallableKind,
) -> (Node<LoweredOp>, LoweredEndpoint) {
    let node_id = lowered_node_id(module_name, &callable.name);
    let mut inputs = callable
        .params
        .iter()
        .map(|binding| {
            Port::with_cardinality(binding.name.as_str(), binding.ty.as_str(), Cardinality::ONE)
        })
        .collect::<Vec<_>>();
    inputs.push(Port::with_cardinality(
        "__deps",
        "Any",
        Cardinality::ZERO_OR_MORE,
    ));
    let outputs = if callable.outputs.is_empty() {
        vec![Port::with_cardinality("return", "Unit", Cardinality::ONE)]
    } else {
        callable
            .outputs
            .iter()
            .map(|binding| {
                Port::with_cardinality(binding.name.as_str(), binding.ty.as_str(), Cardinality::ONE)
            })
            .collect()
    };
    let primary_output = outputs
        .first()
        .map(|port| port.name.0.clone())
        .unwrap_or_else(|| "return".to_string());
    (
        Node::opaque(
            node_id.clone(),
            inputs,
            outputs,
            LoweredOp::Callable {
                module: module_name.to_string(),
                kind,
                name: callable.name.clone(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ),
        LoweredEndpoint {
            node_id,
            primary_output,
        },
    )
}

fn lowered_node_id(module_name: &str, item_name: &str) -> String {
    format!("{module_name}::{item_name}").replace([' ', '/'], "_")
}

fn register_endpoint(
    by_full: &mut HashMap<(String, String), LoweredEndpoint>,
    by_name: &mut HashMap<String, Option<LoweredEndpoint>>,
    module_name: &str,
    callable_name: &str,
    endpoint: LoweredEndpoint,
) {
    by_full.insert(
        (module_name.to_string(), callable_name.to_string()),
        endpoint.clone(),
    );
    by_name
        .entry(callable_name.to_string())
        .and_modify(|existing| {
            if let Some(current) = existing {
                if current != &endpoint {
                    *existing = None;
                }
            }
        })
        .or_insert(Some(endpoint));
}

fn add_dependency_edges(
    builder: &mut DagBuilder,
    project: &TypedProject,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    emit_collection_nodes: bool,
) {
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Some((item_name, stmts)) = item_callable_body(&item.node) else {
                continue;
            };
            let Some(target) = endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };

            let mut calls = BTreeSet::new();
            collect_calls_from_stmts(stmts, &mut calls);
            for call in calls {
                let Some(Some(source)) = endpoints_by_name.get(&call) else {
                    continue;
                };
                if source.node_id == target.node_id {
                    continue;
                }
                builder.add_edge(
                    &source.node_id,
                    &source.primary_output,
                    &target.node_id,
                    "__deps",
                );
            }

            expand_content_upsert_patterns(
                builder,
                &module_name,
                item_name,
                stmts,
                target,
                endpoints_by_name,
            );
            if emit_collection_nodes {
                add_collection_pipeline_nodes(builder, &module_name, stmts, target);
            }
        }
    }
}

fn add_collection_pipeline_nodes(
    builder: &mut DagBuilder,
    module_name: &str,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
) {
    let specs = derive_collection_node_specs(&target.node_id, stmts);
    if specs.is_empty() {
        return;
    }
    let plan = build_collection_lowering_plan(module_name, &target.node_id, &specs);
    for node in plan.nodes {
        builder.add_node(node);
    }
    for (from_node, from_port, to_node, to_port) in plan.edges {
        builder.add_edge(&from_node, &from_port, &to_node, &to_port);
    }
}

fn expand_content_upsert_patterns(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
) {
    let mut bound_callables = HashMap::<String, String>::new();
    let mut expansion_count = 0usize;

    for stmt in stmts {
        let maybe_binding = match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => Some((name, expr)),
            Stmt::Expr(expr) => {
                if let Expr::Call(name, args) = expr {
                    if name == "content_upsert" {
                        expansion_count += 1;
                        expand_single_content_upsert(
                            builder,
                            module_name,
                            item_name,
                            expansion_count,
                            args,
                            target,
                            &bound_callables,
                            endpoints_by_name,
                        );
                    }
                }
                None
            }
            Stmt::Return(_) => None,
        };

        let Some((binding, expr)) = maybe_binding else {
            continue;
        };
        match expr {
            Expr::Call(name, args) => {
                if should_track_call(name) {
                    bound_callables.insert(binding.clone(), name.clone());
                }
                if name == "content_upsert" {
                    expansion_count += 1;
                    expand_single_content_upsert(
                        builder,
                        module_name,
                        item_name,
                        expansion_count,
                        args,
                        target,
                        &bound_callables,
                        endpoints_by_name,
                    );
                }
            }
            Expr::Ident(source) => {
                if let Some(origin) = bound_callables.get(source) {
                    bound_callables.insert(binding.clone(), origin.clone());
                }
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn expand_single_content_upsert(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    expansion_count: usize,
    args: &[(Option<String>, Expr)],
    target: &LoweredEndpoint,
    bound_callables: &HashMap<String, String>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
) {
    let suffix = expansion_suffix(item_name, expansion_count);
    let prepare_read_id = format!("prepare_read_{suffix}");
    let execute_read_id = format!("execute_read_{suffix}");
    let compare_id = format!("compare_{suffix}_content");
    let prepare_write_id = format!("prepare_write_{suffix}");
    let execute_transport_id = format!("execute_{suffix}_transport");

    builder.add_node(Node::opaque(
        prepare_read_id.clone(),
        vec![
            Port::scalar("path", "String"),
            Port::scalar("res:file:Makefile", "FilesystemHandle"),
        ],
        vec![Port::scalar("request", "TransportRequest")],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Pattern,
            name: format!("content_upsert::{prepare_read_id}"),
            obligation: ObligationCategory::None,
            service_metadata: None,
        },
    ));
    builder.add_node(Node::opaque(
        execute_read_id.clone(),
        vec![Port::scalar("request", "TransportRequest")],
        vec![Port::scalar("response", "TransportResponse")],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Pattern,
            name: format!("content_upsert::{execute_read_id}"),
            obligation: ObligationCategory::None,
            service_metadata: None,
        },
    ));
    builder.add_node(Node::opaque(
        compare_id.clone(),
        vec![
            Port::scalar("expected_content", "String"),
            Port::scalar("response", "TransportResponse"),
        ],
        vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Pattern,
            name: format!("content_upsert::{compare_id}"),
            obligation: ObligationCategory::None,
            service_metadata: None,
        },
    ));
    builder.add_node(Node::opaque(
        prepare_write_id.clone(),
        vec![
            Port::scalar("content", "String"),
            Port::scalar("path", "String"),
        ],
        vec![Port::scalar("request", "TransportRequest")],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Pattern,
            name: format!("content_upsert::{prepare_write_id}"),
            obligation: ObligationCategory::None,
            service_metadata: None,
        },
    ));
    builder.add_node(Node::opaque(
        execute_transport_id.clone(),
        vec![
            Port::scalar("request", "TransportRequest"),
            Port::scalar("skip", "Bool"),
        ],
        vec![Port::scalar("makegen_response", "TransportResponse")],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Pattern,
            name: format!("content_upsert::{execute_transport_id}"),
            obligation: ObligationCategory::None,
            service_metadata: None,
        },
    ));

    builder.add_edge(&prepare_read_id, "request", &execute_read_id, "request");
    builder.add_edge(&execute_read_id, "response", &compare_id, "response");
    builder.add_edge(
        &prepare_write_id,
        "request",
        &execute_transport_id,
        "request",
    );
    builder.add_edge(&compare_id, "skip", &execute_transport_id, "skip");
    builder.add_edge(
        &execute_transport_id,
        "makegen_response",
        &target.node_id,
        "__deps",
    );

    if let Some(source) = resolve_content_source(args, bound_callables, endpoints_by_name) {
        builder.add_edge(
            &source.node_id,
            &source.primary_output,
            &compare_id,
            "expected_content",
        );
        builder.add_edge(
            &source.node_id,
            &source.primary_output,
            &prepare_write_id,
            "content",
        );
    }
}

fn resolve_content_source(
    args: &[(Option<String>, Expr)],
    bound_callables: &HashMap<String, String>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
) -> Option<LoweredEndpoint> {
    let (_, content_expr) = args
        .iter()
        .find(|(name, _)| matches!(name.as_deref(), Some("content")))?;
    let source_name = match content_expr {
        Expr::Ident(name) => bound_callables
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.clone()),
        Expr::Call(name, _) => name.clone(),
        _ => return None,
    };
    endpoints_by_name
        .get(&source_name)
        .and_then(|entry| entry.clone())
}

fn expansion_suffix(item_name: &str, expansion_count: usize) -> String {
    let base = item_name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if expansion_count <= 1 {
        base
    } else {
        format!("{base}_{expansion_count}")
    }
}

fn sanitize_identifier(value: &str) -> String {
    let mut out = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if out.is_empty() {
        out.push('_');
    }
    out
}

fn add_makegen_scaffolding(
    builder: &mut DagBuilder,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
) {
    let Some(render) =
        endpoints_by_full.get(&("tools.makegen".to_string(), "render_makefile".to_string()))
    else {
        return;
    };
    let makegen = endpoints_by_full.get(&("tools.makegen".to_string(), "makegen".to_string()));

    if !builder.has_node("load_registry") {
        builder.add_node(Node::opaque(
            "load_registry",
            vec![],
            vec![Port::scalar("registry", "ToolRegistry")],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Pattern,
                name: "load_registry".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
    }
    builder.add_edge(
        "load_registry",
        "registry",
        render.node_id.as_str(),
        "registry",
    );
    if let Some(makegen_endpoint) = makegen {
        builder.add_edge(
            "load_registry",
            "registry",
            makegen_endpoint.node_id.as_str(),
            "registry",
        );
    }

    if builder.has_node("prepare_read_makegen") {
        if !builder.has_node("fs_env") {
            builder.add_node(Node::opaque(
                "fs_env",
                vec![],
                vec![Port::scalar("FilesystemHandle", "FilesystemHandle")],
                LoweredOp::Callable {
                    module: "tools.makegen".to_string(),
                    kind: CallableKind::Pattern,
                    name: "fs_env".to_string(),
                    obligation: ObligationCategory::None,
                    service_metadata: None,
                },
            ));
        }
        builder.add_edge(
            "fs_env",
            "FilesystemHandle",
            "prepare_read_makegen",
            "res:file:Makefile",
        );
    }
}

fn has_annotation(annotations: &[Annotation], target: &str) -> bool {
    annotations
        .iter()
        .any(|annotation| annotation.name == target)
}

fn annotation_transport_class(annotations: &[Annotation]) -> Option<ServiceTransportClass> {
    if has_annotation(annotations, "shell") {
        return Some(ServiceTransportClass::ShellLocal);
    }
    if has_annotation(annotations, "rest") {
        return Some(ServiceTransportClass::RestNetwork);
    }
    if has_annotation(annotations, "file") {
        return Some(ServiceTransportClass::FileBoundary);
    }
    None
}

fn annotation_permissions(annotations: &[Annotation]) -> Vec<String> {
    let mut permissions = BTreeSet::new();
    for annotation in annotations
        .iter()
        .filter(|annotation| annotation.name == "permissions")
    {
        for arg in &annotation.args {
            match arg {
                Expr::Literal(Literal::String(value)) => {
                    permissions.insert(value.clone());
                }
                Expr::Ident(path) => {
                    permissions.insert(path.clone());
                }
                _ => {}
            }
        }
    }
    permissions.into_iter().collect::<Vec<_>>()
}

fn derive_service_call_metadata(
    service: &ServiceDef,
    operation: &OperationDef,
) -> ServiceCallMetadata {
    let transport = annotation_transport_class(&operation.annotations)
        .or_else(|| annotation_transport_class(&service.annotations))
        .unwrap_or(ServiceTransportClass::Unknown);
    let mut permissions = annotation_permissions(&service.annotations);
    permissions.extend(annotation_permissions(&operation.annotations));
    permissions.sort();
    permissions.dedup();
    ServiceCallMetadata {
        service: service.name.clone(),
        operation: operation.name.clone(),
        transport,
        idempotent: has_annotation(&operation.annotations, "idempotent")
            || has_annotation(&service.annotations, "idempotent"),
        readonly: has_annotation(&operation.annotations, "readonly")
            || has_annotation(&service.annotations, "readonly"),
        permissions,
    }
}

fn collect_required_service_call_keys(
    project: &TypedProject,
    callable_modules: Option<&HashSet<String>>,
) -> HashSet<String> {
    let mut required = HashSet::new();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        if callable_modules
            .map(|scope| !scope.contains(&module_name))
            .unwrap_or(false)
        {
            continue;
        }
        for item in &module.ast.items {
            let Some((_, stmts)) = item_callable_body(&item.node) else {
                continue;
            };
            let mut calls = Vec::<ServiceCallSite>::new();
            collect_service_calls_from_stmts(stmts, &mut calls);
            for call in calls {
                if let Some(keys) = service_call_lookup_keys(&call.path) {
                    required.insert(keys[0].clone());
                    required.insert(keys[1].clone());
                    required.insert(keys[2].clone());
                }
            }
        }
    }
    required
}

fn add_service_transport_triplets(
    builder: &mut DagBuilder,
    project: &TypedProject,
    required_calls: Option<&HashSet<String>>,
) -> ServiceEndpointRegistry {
    let mut registry = ServiceEndpointRegistry::default();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::ServiceDef(service) = &item.node else {
                continue;
            };

            for operation in &service.operations {
                if let Some(required_calls) = required_calls {
                    let canonical = format!("{}.{}", service.name, operation.name);
                    let service_tail = service
                        .name
                        .rsplit('.')
                        .next()
                        .unwrap_or(service.name.as_str());
                    let short = format!("{service_tail}.{}", operation.name);
                    let module_scoped =
                        format!("{}.{}.{}", module_name, service.name, operation.name);
                    if !required_calls.contains(&canonical)
                        && !required_calls.contains(&short)
                        && !required_calls.contains(&module_scoped)
                    {
                        continue;
                    }
                }
                let service_metadata = derive_service_call_metadata(service, operation);
                let suffix = sanitize_identifier(&format!(
                    "{module_name}_{}_{}",
                    service.name, operation.name
                ));
                let prepare_id = format!("prepare_transport_{suffix}");
                let execute_id = format!("execute_transport_{suffix}");
                let parse_id = format!("parse_transport_{suffix}");

                builder.add_node(Node::opaque(
                    prepare_id.clone(),
                    operation
                        .inputs
                        .iter()
                        .map(|field| {
                            let ty = type_expr_to_string(&field.ty);
                            Port::with_cardinality(
                                field.name.as_str(),
                                ty.as_str(),
                                Cardinality::ONE,
                            )
                        })
                        .collect::<Vec<_>>(),
                    vec![Port::scalar("request", "TransportRequest")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::prepare::{}::{}",
                            service.name, operation.name
                        ),
                        obligation: ObligationCategory::ServiceTransportPrepare,
                        service_metadata: Some(service_metadata.clone()),
                    },
                ));
                builder.add_node(Node::opaque(
                    execute_id.clone(),
                    vec![Port::scalar("request", "TransportRequest")],
                    vec![Port::scalar("response", "TransportResponse")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::execute::{}::{}",
                            service.name, operation.name
                        ),
                        obligation: ObligationCategory::ServiceTransportExecute,
                        service_metadata: Some(service_metadata.clone()),
                    },
                ));
                let parse_outputs = if operation.outputs.is_empty() {
                    vec![Port::scalar("result", "Unit")]
                } else {
                    operation
                        .outputs
                        .iter()
                        .map(|field| {
                            let ty = type_expr_to_string(&field.ty);
                            Port::with_cardinality(
                                field.name.as_str(),
                                ty.as_str(),
                                Cardinality::ONE,
                            )
                        })
                        .collect::<Vec<_>>()
                };
                builder.add_node(Node::opaque(
                    parse_id.clone(),
                    vec![Port::scalar("response", "TransportResponse")],
                    parse_outputs,
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::parse::{}::{}",
                            service.name, operation.name
                        ),
                        obligation: ObligationCategory::ServiceTransportParse,
                        service_metadata: Some(service_metadata),
                    },
                ));

                builder.add_edge(
                    prepare_id.as_str(),
                    "request",
                    execute_id.as_str(),
                    "request",
                );
                builder.add_edge(
                    execute_id.as_str(),
                    "response",
                    parse_id.as_str(),
                    "response",
                );

                let parse_output = operation
                    .outputs
                    .first()
                    .map(|field| field.name.clone())
                    .unwrap_or_else(|| "result".to_string());
                let endpoint = ServiceTransportEndpoint {
                    parse: LoweredEndpoint {
                        node_id: parse_id,
                        primary_output: parse_output,
                    },
                    prepare_node_id: prepare_id,
                    prepare_inputs: operation
                        .inputs
                        .iter()
                        .map(|field| field.name.clone())
                        .collect::<Vec<_>>(),
                };
                registry.register(
                    format!("{}.{}", service.name, operation.name),
                    endpoint.clone(),
                );
                let service_tail = service
                    .name
                    .rsplit('.')
                    .next()
                    .unwrap_or(service.name.as_str());
                registry.register(
                    format!("{service_tail}.{}", operation.name),
                    endpoint.clone(),
                );
                registry.register(
                    format!("{}.{}.{}", module_name, service.name, operation.name),
                    endpoint,
                );
            }
        }
    }
    registry
}

fn add_service_call_edges(
    builder: &mut DagBuilder,
    project: &TypedProject,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    service_registry: &ServiceEndpointRegistry,
) -> Result<(), LowerError> {
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let (item_name, params, stmts, uses_bindings) = match &item.node {
                Item::FnDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    HashSet::new(),
                ),
                Item::FuncDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    def.uses
                        .iter()
                        .map(|usage| usage.binding.clone())
                        .collect::<HashSet<_>>(),
                ),
                Item::PatternDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    def.uses
                        .iter()
                        .map(|usage| usage.binding.clone())
                        .collect::<HashSet<_>>(),
                ),
                _ => continue,
            };
            let Some(target) = endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };
            let param_types = params
                .iter()
                .map(|param| (param.name.clone(), type_expr_to_string(&param.ty)))
                .collect::<HashMap<_, _>>();
            let mut service_calls = Vec::<ServiceCallSite>::new();
            collect_service_calls_from_stmts(stmts, &mut service_calls);
            for call in service_calls {
                let Some(source) = resolve_service_endpoint(&call.path, service_registry) else {
                    if call
                        .path
                        .first()
                        .is_some_and(|segment| uses_bindings.contains(segment))
                    {
                        continue;
                    }
                    return Err(LowerError::UnresolvedServiceCall {
                        caller: format!("{module_name}::{item_name}"),
                        service_call: call.path.join("."),
                    });
                };
                builder.add_edge(
                    source.parse.node_id.as_str(),
                    source.parse.primary_output.as_str(),
                    target.node_id.as_str(),
                    "__deps",
                );
                for (index, arg) in call.args.iter().enumerate() {
                    let Some(arg_ident) = arg.ident.as_deref() else {
                        continue;
                    };
                    let Some(param_ty) = param_types.get(arg_ident) else {
                        continue;
                    };
                    let Some(prepare_input) = arg
                        .name
                        .as_deref()
                        .or_else(|| source.prepare_inputs.get(index).map(String::as_str))
                    else {
                        continue;
                    };
                    let param_source = ensure_param_source_node(
                        builder,
                        module_name.as_str(),
                        item_name,
                        arg_ident,
                        param_ty.as_str(),
                    );
                    builder.add_edge(
                        param_source.as_str(),
                        arg_ident,
                        source.prepare_node_id.as_str(),
                        prepare_input,
                    );
                }
            }
        }
    }
    Ok(())
}

fn add_used_resource_edges(
    builder: &mut DagBuilder,
    project: &TypedProject,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    resource_registry: &ResourceLifecycleRegistry,
    known_uses_types: &HashSet<String>,
) -> Result<(), LowerError> {
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Some((item_name, uses)) = item_callable_uses(&item.node) else {
                continue;
            };
            let Some(target) = endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };
            for usage in uses {
                let resource_type = resource_type_name(&usage.resource_type);
                let resource_type_with_config = type_expr_to_string(&usage.resource_type);
                let provider_hint = provider_hint_from_uses_config(usage.config.as_deref());
                let Some(endpoint) = resolve_resource_endpoint(
                    module_name.as_str(),
                    resource_type.as_str(),
                    resource_type_with_config.as_str(),
                    provider_hint,
                    project,
                    resource_registry,
                ) else {
                    if is_known_uses_type(known_uses_types, &resource_type) {
                        continue;
                    }
                    return Err(LowerError::UnresolvedUsedResource {
                        caller: format!("{module_name}::{item_name}"),
                        binding: usage.binding.clone(),
                        resource_type,
                    });
                };
                if let Some(acquire_node) = endpoint.acquire_node {
                    builder.add_edge(
                        acquire_node.as_str(),
                        "resource_handle",
                        target.node_id.as_str(),
                        "__deps",
                    );
                }
                if let Some(release_node) = endpoint.release_node {
                    builder.add_edge(
                        target.node_id.as_str(),
                        target.primary_output.as_str(),
                        release_node.as_str(),
                        "resource_handle",
                    );
                }
            }
        }
    }
    Ok(())
}

fn add_provided_resource_nodes(
    builder: &mut DagBuilder,
    project: &TypedProject,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    resource_registry: &ResourceLifecycleRegistry,
    known_uses_types: &HashSet<String>,
) -> Result<(), LowerError> {
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Some((item_name, provides)) = item_callable_provides(&item.node) else {
                continue;
            };
            let Some(target) = endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };
            for provided in provides {
                let resource_type = resource_type_name(&provided.resource_type);
                let resource_type_with_config = type_expr_to_string(&provided.resource_type);
                let endpoint = resolve_resource_endpoint(
                    module_name.as_str(),
                    resource_type.as_str(),
                    resource_type_with_config.as_str(),
                    None,
                    project,
                    resource_registry,
                );
                if endpoint.is_none() && !is_known_uses_type(known_uses_types, &resource_type) {
                    return Err(LowerError::UnresolvedProvidedResource {
                        caller: format!("{module_name}::{item_name}"),
                        binding: provided.binding.clone(),
                        resource_type,
                    });
                }

                let provider_node_id = format!(
                    "provide_resource_{}",
                    sanitize_identifier(&format!("{module_name}_{item_name}_{}", provided.binding))
                );
                builder.add_node(Node::opaque(
                    provider_node_id.clone(),
                    vec![Port::scalar("trigger", "Any")],
                    vec![Port::with_cardinality(
                        provided.binding.as_str(),
                        resource_type.as_str(),
                        Cardinality::ONE,
                    )],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!("resource_provide::{}::{}", item_name, provided.binding),
                        obligation: ObligationCategory::ResourceProvide,
                        service_metadata: None,
                    },
                ));
                builder.add_edge(
                    target.node_id.as_str(),
                    target.primary_output.as_str(),
                    provider_node_id.as_str(),
                    "trigger",
                );
                if let Some(endpoint) = endpoint {
                    if let Some(release_node) = endpoint.release_node {
                        builder.add_edge(
                            provider_node_id.as_str(),
                            provided.binding.as_str(),
                            release_node.as_str(),
                            "resource_handle",
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn resolve_resource_endpoint(
    module_name: &str,
    resource_type: &str,
    resource_type_with_config: &str,
    provider_hint: Option<ProviderHint>,
    project: &TypedProject,
    registry: &ResourceLifecycleRegistry,
) -> Option<ResourceLifecycleEndpoint> {
    if let Some(endpoint) = resolve_concrete_resource_endpoint(module_name, resource_type, registry)
    {
        return Some(endpoint);
    }
    resolve_interface_resource_endpoint(
        resource_type,
        resource_type_with_config,
        provider_hint,
        project,
        registry,
    )
}

fn resolve_concrete_resource_endpoint(
    module_name: &str,
    resource_type: &str,
    registry: &ResourceLifecycleRegistry,
) -> Option<ResourceLifecycleEndpoint> {
    let keys = [
        format!("{module_name}.{resource_type}"),
        resource_type.to_string(),
    ];
    for key in keys {
        if let Some(Some(endpoint)) = registry.by_key.get(&key) {
            return Some(endpoint.clone());
        }
    }
    None
}

fn resolve_interface_resource_endpoint(
    resource_type: &str,
    resource_type_with_config: &str,
    provider_hint: Option<ProviderHint>,
    project: &TypedProject,
    registry: &ResourceLifecycleRegistry,
) -> Option<ResourceLifecycleEndpoint> {
    let target_canonical = canonical_type_name(resource_type);
    let target_short = target_canonical
        .rsplit('.')
        .next()
        .unwrap_or(target_canonical.as_str());
    let provider_hint =
        provider_hint.or_else(|| provider_hint_from_resource_type(resource_type_with_config));
    let mut candidates = Vec::<(Option<ProviderHint>, ResourceLifecycleEndpoint)>::new();

    for module in &project.modules {
        let candidate_module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::ResourceDef(resource) = &item.node else {
                continue;
            };
            let Some(implemented) = &resource.implements else {
                continue;
            };
            let implemented_canonical = canonical_type_name(implemented);
            let implemented_short = implemented_canonical
                .rsplit('.')
                .next()
                .unwrap_or(implemented_canonical.as_str());
            if implemented_canonical != target_canonical && implemented_short != target_short {
                continue;
            }
            let Some(endpoint) = resolve_concrete_resource_endpoint(
                candidate_module_name.as_str(),
                resource.name.as_str(),
                registry,
            ) else {
                continue;
            };
            candidates.push((
                provider_hint_from_module_name(candidate_module_name.as_str())
                    .or_else(|| provider_hint_from_resource_name(resource.name.as_str())),
                endpoint,
            ));
        }
    }

    if let Some(required_provider) = provider_hint {
        candidates.retain(|(candidate_provider, _)| {
            candidate_provider.is_some_and(|provider| provider == required_provider)
        });
    }

    if candidates.len() == 1 {
        return candidates.into_iter().next().map(|(_, endpoint)| endpoint);
    }
    None
}

fn collect_known_uses_types(project: &TypedProject) -> HashSet<String> {
    let mut known = HashSet::new();
    insert_default_known_resource_types(&mut known);
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            match &item.node {
                Item::InterfaceDef(def) => {
                    insert_canonical_names(&mut known, &def.name);
                    insert_canonical_names(&mut known, &format!("{module_name}.{}", def.name));
                }
                Item::ResourceDef(def) => {
                    insert_canonical_names(&mut known, &def.name);
                    insert_canonical_names(&mut known, &format!("{module_name}.{}", def.name));
                    if let Some(implemented) = &def.implements {
                        insert_canonical_names(&mut known, implemented);
                    }
                }
                Item::ServiceDef(def) => {
                    insert_canonical_names(&mut known, &def.name);
                    insert_canonical_names(&mut known, &format!("{module_name}.{}", def.name));
                    if let Some(implemented) = &def.implements {
                        insert_canonical_names(&mut known, implemented);
                    }
                }
                _ => {}
            }
        }
    }
    known
}

fn insert_default_known_resource_types(known: &mut HashSet<String>) {
    for resource_type in ["Filesystem", "Network", "Clock", "AuthContext"] {
        insert_canonical_names(known, resource_type);
        insert_canonical_names(known, &format!("std.resources.{resource_type}"));
    }
}

fn add_resource_lifecycle_nodes(
    builder: &mut DagBuilder,
    project: &TypedProject,
    callable_modules: Option<&HashSet<String>>,
) -> ResourceLifecycleRegistry {
    let mut registry = ResourceLifecycleRegistry::default();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        if callable_modules
            .map(|scope| !scope.contains(&module_name))
            .unwrap_or(false)
        {
            continue;
        }
        for item in &module.ast.items {
            let Item::ResourceDef(resource) = &item.node else {
                continue;
            };
            let suffix = sanitize_identifier(&format!("{module_name}_{}", resource.name));
            let acquire_id = format!("acquire_resource_{suffix}");
            let release_id = format!("release_resource_{suffix}");
            let mut has_acquire = false;
            let mut has_release = false;

            if resource.acquire.is_some() {
                builder.add_node(Node::opaque(
                    acquire_id.clone(),
                    vec![],
                    vec![Port::scalar("resource_handle", "ResourceHandle")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!("resource_lifecycle::acquire::{}", resource.name),
                        obligation: ObligationCategory::ResourceAcquire,
                        service_metadata: None,
                    },
                ));
                has_acquire = true;
            }
            if resource.release.is_some() {
                builder.add_node(Node::opaque(
                    release_id.clone(),
                    vec![Port::scalar("resource_handle", "ResourceHandle")],
                    vec![Port::scalar("released", "Bool")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!("resource_lifecycle::release::{}", resource.name),
                        obligation: ObligationCategory::ResourceRelease,
                        service_metadata: None,
                    },
                ));
                has_release = true;
            }
            if has_acquire && has_release {
                builder.add_edge(
                    acquire_id.as_str(),
                    "resource_handle",
                    release_id.as_str(),
                    "resource_handle",
                );
            }
            let endpoint = ResourceLifecycleEndpoint {
                acquire_node: has_acquire.then_some(acquire_id),
                release_node: has_release.then_some(release_id),
            };
            registry.register(format!("{module_name}.{}", resource.name), endpoint.clone());
            registry.register(resource.name.clone(), endpoint);
        }
    }
    registry
}

fn add_interface_contract_verification_nodes(
    builder: &mut DagBuilder,
    project: &TypedProject,
    resource_registry: &ResourceLifecycleRegistry,
) {
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::ResourceDef(resource) = &item.node else {
                continue;
            };
            let Some(interface_name) = &resource.implements else {
                continue;
            };
            let contract_count =
                resolve_interface_contract_count(project, module_name.as_str(), interface_name);
            if contract_count == 0 {
                continue;
            }
            let endpoint = resolve_concrete_resource_endpoint(
                module_name.as_str(),
                resource.name.as_str(),
                resource_registry,
            );
            for index in 0..contract_count {
                let node_id = format!(
                    "verify_contract_{}",
                    sanitize_identifier(&format!(
                        "{module_name}_{}_{}_{}",
                        resource.name,
                        canonical_type_name(interface_name),
                        index
                    ))
                );
                builder.add_node(Node::opaque(
                    node_id.clone(),
                    vec![Port::scalar("contract", "String")],
                    vec![Port::scalar("verified", "Bool")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "interface_contract::{}::{}::{}",
                            resource.name,
                            canonical_type_name(interface_name),
                            index
                        ),
                        obligation: ObligationCategory::InterfaceContractVerification,
                        service_metadata: None,
                    },
                ));
                if let Some(acquire_node) = endpoint
                    .as_ref()
                    .and_then(|entry| entry.acquire_node.as_ref())
                {
                    builder.add_edge(
                        acquire_node.as_str(),
                        "resource_handle",
                        node_id.as_str(),
                        "__deps",
                    );
                }
            }
        }
    }
}

fn resolve_interface_contract_count(
    project: &TypedProject,
    resource_module_name: &str,
    interface_name: &str,
) -> usize {
    let target = canonical_type_name(interface_name);
    let target_short = target.rsplit('.').next().unwrap_or(target.as_str());
    let mut counts = Vec::new();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::InterfaceDef(interface) = &item.node else {
                continue;
            };
            let qualified = format!("{module_name}.{}", interface.name);
            let qualified_canonical = canonical_type_name(&qualified);
            let interface_short = interface
                .name
                .rsplit('.')
                .next()
                .unwrap_or(interface.name.as_str());
            let matches_target = if target.contains('.') {
                qualified_canonical == target
            } else {
                interface_short == target_short
            };
            if !matches_target {
                continue;
            }
            if module_name == resource_module_name || target.contains('.') {
                counts.push(interface.contracts.len());
            } else {
                counts.push(interface.contracts.len());
            }
        }
    }
    if counts.len() == 1 {
        return counts[0];
    }
    0
}

fn resolve_service_endpoint(
    call_path: &[String],
    registry: &ServiceEndpointRegistry,
) -> Option<ServiceTransportEndpoint> {
    let keys = service_call_lookup_keys(call_path)?;
    for key in keys {
        if let Some(Some(endpoint)) = registry.by_key.get(&key) {
            return Some(endpoint.clone());
        }
    }
    None
}

fn ensure_param_source_node(
    builder: &mut DagBuilder,
    module_name: &str,
    callable: &str,
    param: &str,
    ty: &str,
) -> String {
    let node_id = format!(
        "param_source_{}",
        sanitize_identifier(&format!("{module_name}_{callable}_{param}"))
    );
    builder.add_node(Node::opaque(
        node_id.clone(),
        vec![],
        vec![Port::with_cardinality(param, ty, Cardinality::ONE)],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Pattern,
            name: format!("call_param_source::{callable}::{param}"),
            obligation: ObligationCategory::ServiceParamSource,
            service_metadata: None,
        },
    ));
    node_id
}

fn item_callable_body(item: &Item) -> Option<(&str, &[Stmt])> {
    match item {
        Item::FnDef(def) => Some((def.name.as_str(), def.body.stmts.as_slice())),
        Item::FuncDef(def) => Some((def.name.as_str(), def.body.stmts.as_slice())),
        Item::PatternDef(def) => Some((def.name.as_str(), def.body.stmts.as_slice())),
        _ => None,
    }
}

fn item_callable_uses(item: &Item) -> Option<(&str, &[daglang_syntax::ast::UsesClause])> {
    match item {
        Item::FuncDef(def) => Some((def.name.as_str(), def.uses.as_slice())),
        Item::PatternDef(def) => Some((def.name.as_str(), def.uses.as_slice())),
        _ => None,
    }
}

fn item_callable_provides(item: &Item) -> Option<(&str, &[daglang_syntax::ast::ProvidesClause])> {
    match item {
        Item::FuncDef(def) => Some((def.name.as_str(), def.provides.as_slice())),
        _ => None,
    }
}

fn collect_calls_from_stmts(stmts: &[Stmt], calls: &mut BTreeSet<String>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Call(name, _) = expr {
            if should_track_call(name) {
                calls.insert(name.clone());
            }
        }
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceCallArgSite {
    name: Option<String>,
    ident: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceCallSite {
    path: Vec<String>,
    args: Vec<ServiceCallArgSite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionOpKind {
    Map,
    Filter,
    Fold,
    Join,
    FlatMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CollectionOpSite {
    kind: CollectionOpKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CollectionNodeSpec {
    node_id: String,
    kind: CollectionOpKind,
}

fn collection_op_kind(name: &str) -> Option<CollectionOpKind> {
    match name {
        "map" => Some(CollectionOpKind::Map),
        "filter" => Some(CollectionOpKind::Filter),
        "fold" => Some(CollectionOpKind::Fold),
        "join" => Some(CollectionOpKind::Join),
        "flat_map" => Some(CollectionOpKind::FlatMap),
        _ => None,
    }
}

fn collect_collection_ops_from_stmts(stmts: &[Stmt], sites: &mut Vec<CollectionOpSite>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Pipe(_, rhs) = expr {
            let Expr::Call(name, _) = rhs.as_ref() else {
                return;
            };
            let Some(kind) = collection_op_kind(name) else {
                return;
            };
            sites.push(CollectionOpSite { kind });
        }
    });
}

fn derive_collection_node_specs(callable_node_id: &str, stmts: &[Stmt]) -> Vec<CollectionNodeSpec> {
    let mut sites = Vec::new();
    collect_collection_ops_from_stmts(stmts, &mut sites);
    sites.reverse();
    sites
        .into_iter()
        .enumerate()
        .map(|(index, site)| CollectionNodeSpec {
            node_id: format!(
                "{callable_node_id}::{}_{index}",
                collection_kind_node_label(site.kind)
            ),
            kind: site.kind,
        })
        .collect()
}

#[derive(Debug)]
struct CollectionLoweringPlan {
    nodes: Vec<Node<LoweredOp>>,
    edges: Vec<(String, String, String, String)>,
}

fn collection_kind_node_label(kind: CollectionOpKind) -> &'static str {
    match kind {
        CollectionOpKind::Map => "MapNode",
        CollectionOpKind::Filter => "FilterNode",
        CollectionOpKind::Fold => "FoldNode",
        CollectionOpKind::Join => "JoinNode",
        CollectionOpKind::FlatMap => "FlatMapNode",
    }
}

fn build_collection_lowering_plan(
    module_name: &str,
    callable_node_id: &str,
    specs: &[CollectionNodeSpec],
) -> CollectionLoweringPlan {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut previous_node_id: Option<String> = None;
    for spec in specs {
        let node = Node::opaque(
            spec.node_id.clone(),
            vec![
                Port::with_cardinality("items", "Any", Cardinality::ONE),
                Port::with_cardinality("__deps", "Any", Cardinality::ZERO_OR_MORE),
            ],
            vec![Port::with_cardinality("items", "Any", Cardinality::ONE)],
            LoweredOp::Collection {
                module: module_name.to_string(),
                callable: callable_node_id.to_string(),
                kind: spec.kind,
            },
        );
        if let Some(prev) = &previous_node_id {
            edges.push((
                prev.clone(),
                "items".to_string(),
                spec.node_id.clone(),
                "items".to_string(),
            ));
        }
        previous_node_id = Some(spec.node_id.clone());
        nodes.push(node);
    }
    if let Some(last) = previous_node_id {
        edges.push((
            last,
            "items".to_string(),
            callable_node_id.to_string(),
            "__deps".to_string(),
        ));
    }
    CollectionLoweringPlan { nodes, edges }
}

fn collect_service_calls_from_stmts(stmts: &[Stmt], calls: &mut Vec<ServiceCallSite>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::ServiceCall(path, args) = expr {
            calls.push(ServiceCallSite {
                path: path.clone(),
                args: args
                    .iter()
                    .map(|(name, arg)| ServiceCallArgSite {
                        name: name.clone(),
                        ident: match arg {
                            Expr::Ident(ident) => Some(ident.clone()),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>(),
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_resolve::{ModuleGraph, ResolvedModule};
    use daglang_syntax::parser;
    use daglang_typecheck::typecheck_module_graph;
    use gunbc_clippy::build_clippy_graph_lint_all;
    use gunbc_dag::{build_bootstrap_graph, build_ci_graph, build_makegen_graph};
    use gunbc_deps::build_deps_graph;
    use gunbc_gist::{build_gist_graph, GistMode};
    use gunbc_ir::{Edge, Port};
    use gunbc_lib_aws_ops::build_aws_secrets_manager_credential_graph;
    use gunbc_lib_azure_ops::build_azure_key_vault_credential_graph;
    use gunbc_lib_gcp_ops::build_gcp_secret_manager_credential_graph_github;
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn typed_project_from_sources(sources: &[(&str, &str)]) -> TypedProject {
        let modules = sources
            .iter()
            .map(|(path, source)| {
                let ast = parser::parse(source).expect("source should parse");
                let module_path = ast
                    .module_path
                    .as_ref()
                    .map(|module| module.node.segments.clone())
                    .expect("module declaration is required");
                ResolvedModule {
                    path: PathBuf::from(path),
                    ast,
                    module_path,
                    dependencies: Vec::new(),
                }
            })
            .collect();
        typecheck_module_graph(ModuleGraph { modules }).expect("typecheck should succeed")
    }

    fn callable_stmts_from_source(source: &str) -> Vec<Stmt> {
        let ast = parser::parse(source).expect("source should parse");
        let item = ast.items.first().expect("source should contain one item");
        match &item.node {
            Item::FnDef(def) => def.body.stmts.clone(),
            Item::FuncDef(def) => def.body.stmts.clone(),
            Item::PatternDef(def) => def.body.stmts.clone(),
            other => panic!("expected callable item, got {other:?}"),
        }
    }

    // Test infrastructure: filesystem access for real DSL corpus fixtures.
    #[allow(clippy::disallowed_methods)]
    fn typed_project_for_module_with_dependency_closure(module_name: &str) -> TypedProject {
        let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let graph =
            ModuleGraph::discover(&[dsl_root]).expect("module graph discovery should succeed");
        let target_index = graph
            .modules
            .iter()
            .position(|module| module.module_path.join(".") == module_name)
            .expect("target module should exist in graph");

        let mut required_indices = HashSet::<usize>::new();
        let mut queue = VecDeque::from([target_index]);
        while let Some(module_index) = queue.pop_front() {
            if !required_indices.insert(module_index) {
                continue;
            }
            let Some(module) = graph.modules.get(module_index) else {
                continue;
            };
            for dependency in &module.dependencies {
                queue.push_back(*dependency);
            }
        }

        let mut ordered_indices = required_indices.iter().copied().collect::<Vec<_>>();
        ordered_indices.sort_unstable();
        let index_map = ordered_indices
            .iter()
            .enumerate()
            .map(|(new_index, old_index)| (*old_index, new_index))
            .collect::<HashMap<_, _>>();

        let mut modules = Vec::new();
        for (old_index, mut module) in graph.modules.into_iter().enumerate() {
            if !required_indices.contains(&old_index) {
                continue;
            }
            module.dependencies = module
                .dependencies
                .into_iter()
                .filter_map(|dependency| index_map.get(&dependency).copied())
                .collect::<Vec<_>>();
            modules.push(module);
        }
        typecheck_module_graph(ModuleGraph { modules }).expect("typecheck should succeed")
    }

    fn lower_target_module(typed: &TypedProject, module_name: &str) -> Dag<LoweredOp> {
        let mut scope = HashSet::new();
        scope.insert(module_name.to_string());
        lower_typed_project_for_modules(typed, &scope).expect("lowering should succeed")
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn lower_makegen_produces_callable_nodes() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl/tools/makegen.dag");
        let source = fs::read_to_string(file).expect("should read makegen source");
        let typed = typed_project_from_sources(&[("dsl/tools/makegen.dag", &source)]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");

        let node_ids = dag
            .nodes
            .iter()
            .map(|node| node.id.0.as_str())
            .collect::<Vec<_>>();
        assert!(node_ids.contains(&"tools.makegen::render_makefile"));
        assert!(node_ids.contains(&"tools.makegen::makegen"));
        assert!(node_ids.contains(&"prepare_read_makegen"));
        assert!(node_ids.contains(&"execute_read_makegen"));
        assert!(node_ids.contains(&"compare_makegen_content"));
        assert!(node_ids.contains(&"prepare_write_makegen"));
        assert!(node_ids.contains(&"execute_makegen_transport"));
        assert!(node_ids.contains(&"load_registry"));
        assert!(node_ids.contains(&"fs_env"));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "tools.makegen::render_makefile"
                && edge.to_node.0 == "tools.makegen::makegen"
                && edge.to_port.0 == "__deps"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "tools.makegen::render_makefile"
                && edge.to_node.0 == "compare_makegen_content"
                && edge.to_port.0 == "expected_content"
        }));
    }

    #[test]
    fn collect_collection_ops_detects_pipe_map_filter_join_chain() {
        let stmts = callable_stmts_from_source(
            r#"
module sample.collections
fn run(values: List<String>) -> { out: String } {
  rendered = values
    |> map(v => v)
    |> filter(v => v != "")
    |> join(",")
  return { out: rendered }
}
"#,
        );
        let mut sites = Vec::new();
        collect_collection_ops_from_stmts(&stmts, &mut sites);
        let kinds = sites.iter().map(|site| site.kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                CollectionOpKind::Join,
                CollectionOpKind::Filter,
                CollectionOpKind::Map,
            ]
        );
    }

    #[test]
    fn collect_collection_ops_ignores_non_pipe_collection_calls() {
        let stmts = callable_stmts_from_source(
            r#"
module sample.collections
fn run(values: List<String>) -> { out: String } {
  rendered = map(values, v => v)
  return { out: rendered }
}
"#,
        );
        let mut sites = Vec::new();
        collect_collection_ops_from_stmts(&stmts, &mut sites);
        assert!(sites.is_empty());
    }

    #[test]
    fn derive_collection_node_specs_orders_pipeline_left_to_right() {
        let stmts = callable_stmts_from_source(
            r#"
module sample.collections
fn run(values: List<String>) -> { out: String } {
  rendered = values
    |> map(v => v)
    |> filter(v => v != "")
    |> join(",")
  return { out: rendered }
}
"#,
        );
        let specs = derive_collection_node_specs("sample.collections::run", &stmts);
        assert_eq!(
            specs,
            vec![
                CollectionNodeSpec {
                    node_id: "sample.collections::run::MapNode_0".to_string(),
                    kind: CollectionOpKind::Map,
                },
                CollectionNodeSpec {
                    node_id: "sample.collections::run::FilterNode_1".to_string(),
                    kind: CollectionOpKind::Filter,
                },
                CollectionNodeSpec {
                    node_id: "sample.collections::run::JoinNode_2".to_string(),
                    kind: CollectionOpKind::Join,
                },
            ]
        );
    }

    #[test]
    fn build_collection_lowering_plan_chains_nodes_and_wires_target_dependency() {
        let specs = vec![
            CollectionNodeSpec {
                node_id: "sample.collections::run::MapNode_0".to_string(),
                kind: CollectionOpKind::Map,
            },
            CollectionNodeSpec {
                node_id: "sample.collections::run::FilterNode_1".to_string(),
                kind: CollectionOpKind::Filter,
            },
            CollectionNodeSpec {
                node_id: "sample.collections::run::JoinNode_2".to_string(),
                kind: CollectionOpKind::Join,
            },
        ];
        let plan =
            build_collection_lowering_plan("sample.collections", "sample.collections::run", &specs);
        assert_eq!(plan.nodes.len(), 3);
        assert_eq!(plan.edges.len(), 3);
        let kinds = plan
            .nodes
            .iter()
            .map(|node| match &node.body {
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection { kind, .. }) => *kind,
                _ => panic!("expected collection lowered op"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                CollectionOpKind::Map,
                CollectionOpKind::Filter,
                CollectionOpKind::Join,
            ]
        );
        assert_eq!(
            plan.edges,
            vec![
                (
                    "sample.collections::run::MapNode_0".to_string(),
                    "items".to_string(),
                    "sample.collections::run::FilterNode_1".to_string(),
                    "items".to_string(),
                ),
                (
                    "sample.collections::run::FilterNode_1".to_string(),
                    "items".to_string(),
                    "sample.collections::run::JoinNode_2".to_string(),
                    "items".to_string(),
                ),
                (
                    "sample.collections::run::JoinNode_2".to_string(),
                    "items".to_string(),
                    "sample.collections::run".to_string(),
                    "__deps".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn lower_typed_project_emits_collection_nodes_for_pipe_chain() {
        let typed = typed_project_from_sources(&[(
            "sample.collections",
            r#"
module sample.collections

fn run(values: List<String>) -> String {
  rendered = values
    |> map(v => v)
    |> filter(v => v != "")
    |> join(",")
  return rendered
}
"#,
        )]);
        let dag =
            lower_typed_project_with_collection_nodes(&typed).expect("lowering should succeed");
        let node_ids = dag
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        assert!(node_ids.contains("sample.collections::run::MapNode_0"));
        assert!(node_ids.contains("sample.collections::run::FilterNode_1"));
        assert!(node_ids.contains("sample.collections::run::JoinNode_2"));
        let mut collection_kinds = dag
            .nodes
            .iter()
            .filter_map(|node| match &node.body {
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection { kind, .. }) => Some(*kind),
                _ => None,
            })
            .collect::<Vec<_>>();
        collection_kinds.sort_by_key(|kind| match kind {
            CollectionOpKind::Map => 0,
            CollectionOpKind::Filter => 1,
            CollectionOpKind::Fold => 2,
            CollectionOpKind::Join => 3,
            CollectionOpKind::FlatMap => 4,
        });
        assert_eq!(
            collection_kinds,
            vec![
                CollectionOpKind::Map,
                CollectionOpKind::Filter,
                CollectionOpKind::Join,
            ]
        );

        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.collections::run::MapNode_0"
                && edge.from_port.0 == "items"
                && edge.to_node.0 == "sample.collections::run::FilterNode_1"
                && edge.to_port.0 == "items"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.collections::run::FilterNode_1"
                && edge.from_port.0 == "items"
                && edge.to_node.0 == "sample.collections::run::JoinNode_2"
                && edge.to_port.0 == "items"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.collections::run::JoinNode_2"
                && edge.from_port.0 == "items"
                && edge.to_node.0 == "sample.collections::run"
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn gcp_credential_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.gcp.credential");
        let dag = lower_target_module(&typed, "cloud.gcp.credential");
        let reference = build_gcp_secret_manager_credential_graph_github();

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn clippy_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.clippy");
        let dag = lower_target_module(&typed, "tools.clippy");
        let reference = build_clippy_graph_lint_all();

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn deps_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.deps");
        let dag = lower_target_module(&typed, "tools.deps");
        let reference = build_deps_graph().expect("deps builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn gist_snapshot_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module(&typed, "tools.gist");
        let reference = build_gist_graph(GistMode::Snapshot, Vec::new(), false)
            .expect("gist builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn gist_diff_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module(&typed, "tools.gist");
        let reference = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            Vec::new(),
            false,
        )
        .expect("gist builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn gist_recent_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module(&typed, "tools.gist");
        let reference = build_gist_graph(GistMode::Recent, Vec::new(), false)
            .expect("gist builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn aws_credential_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.aws.credential");
        let dag = lower_target_module(&typed, "cloud.aws.credential");
        let reference = build_aws_secrets_manager_credential_graph();

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn azure_credential_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.azure.credential");
        let dag = lower_target_module(&typed, "cloud.azure.credential");
        let reference = build_azure_key_vault_credential_graph();

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn ci_pipeline_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("pipelines.ci");
        let dag = lower_target_module(&typed, "pipelines.ci");
        let reference = build_ci_graph().expect("ci builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn bootstrap_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.bootstrap");
        let dag = lower_target_module(&typed, "tools.bootstrap");
        let reference =
            build_bootstrap_graph().expect("bootstrap builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn content_upsert_expansion_wires_transport_chain_for_makegen() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl/tools/makegen.dag");
        let source = fs::read_to_string(file).expect("should read makegen source");
        let typed = typed_project_from_sources(&[("dsl/tools/makegen.dag", &source)]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");

        assert_eq!(
            dag.nodes.len(),
            9,
            "expected callable + content_upsert chain + source scaffold nodes"
        );
        let required_edges = [
            (
                "prepare_read_makegen",
                "request",
                "execute_read_makegen",
                "request",
            ),
            (
                "execute_read_makegen",
                "response",
                "compare_makegen_content",
                "response",
            ),
            (
                "prepare_write_makegen",
                "request",
                "execute_makegen_transport",
                "request",
            ),
            (
                "compare_makegen_content",
                "skip",
                "execute_makegen_transport",
                "skip",
            ),
            (
                "execute_makegen_transport",
                "makegen_response",
                "tools.makegen::makegen",
                "__deps",
            ),
            (
                "load_registry",
                "registry",
                "tools.makegen::render_makefile",
                "registry",
            ),
            (
                "fs_env",
                "FilesystemHandle",
                "prepare_read_makegen",
                "res:file:Makefile",
            ),
        ];
        for (from_node, from_port, to_node, to_port) in required_edges {
            assert!(
                dag.edges.iter().any(|edge| {
                    edge.from_node.0 == from_node
                        && edge.from_port.0 == from_port
                        && edge.to_node.0 == to_node
                        && edge.to_port.0 == to_port
                }),
                "missing edge {from_node}.{from_port} -> {to_node}.{to_port}"
            );
        }
    }

    #[test]
    fn interface_backed_service_lowers_transport_triplet_nodes() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/storage.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let node_ids = dag
            .nodes
            .iter()
            .map(|node| node.id.0.as_str())
            .collect::<Vec<_>>();
        let suffix = "sample_services_FsStorage_read";
        let prepare = format!("prepare_transport_{suffix}");
        let execute = format!("execute_transport_{suffix}");
        let parse = format!("parse_transport_{suffix}");

        assert!(node_ids.contains(&prepare.as_str()));
        assert!(node_ids.contains(&execute.as_str()));
        assert!(node_ids.contains(&parse.as_str()));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == prepare.as_str()
                && edge.from_port.0 == "request"
                && edge.to_node.0 == execute.as_str()
                && edge.to_port.0 == "request"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == execute.as_str()
                && edge.from_port.0 == "response"
                && edge.to_node.0 == parse.as_str()
                && edge.to_port.0 == "response"
        }));
    }

    #[test]
    fn concrete_service_without_interface_lowers_transport_triplet_nodes() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/shell.dag",
            r#"module sample.services
service shell.Tools {
  operation Echo(message: String) -> { output: String }
}
func run() -> { output: String } {
  result = shell.Tools.Echo(message: "hello")
  return { output: result.output }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let node_ids = dag
            .nodes
            .iter()
            .map(|node| node.id.0.as_str())
            .collect::<Vec<_>>();
        let suffix = "sample_services_shell_Tools_Echo";
        let prepare = format!("prepare_transport_{suffix}");
        let execute = format!("execute_transport_{suffix}");
        let parse = format!("parse_transport_{suffix}");

        assert!(node_ids.contains(&prepare.as_str()));
        assert!(node_ids.contains(&execute.as_str()));
        assert!(node_ids.contains(&parse.as_str()));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == parse.as_str()
                && edge.to_node.0 == "sample.services::run"
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn service_transport_metadata_preserves_operation_annotations() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/metadata.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service RemoteStorage implements Storage {
  @rest
  @idempotent
  @permissions("storage.read", "storage.inspect")
  operation read(path: String) -> { body: String }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let execute_node = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == "execute_transport_sample_services_RemoteStorage_read")
            .expect("execute transport node should exist");
        let metadata = match &execute_node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => op
                .service_call_metadata()
                .expect("service metadata should be preserved"),
            gunbc_ir::node::NodeBody::SubDag(_) => {
                panic!("expected opaque lowered node for execute transport")
            }
        };
        assert_eq!(metadata.service, "RemoteStorage");
        assert_eq!(metadata.operation, "read");
        assert_eq!(metadata.transport, ServiceTransportClass::RestNetwork);
        assert!(metadata.idempotent);
        assert!(!metadata.readonly);
        assert_eq!(
            metadata.permissions,
            vec!["storage.inspect".to_string(), "storage.read".to_string()]
        );
    }

    #[test]
    fn service_transport_metadata_uses_service_level_annotations_as_fallback() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/service_annotations.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
@shell
@readonly
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let prepare_node = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == "prepare_transport_sample_services_FsStorage_read")
            .expect("prepare transport node should exist");
        let (transport_class, readonly) = match &prepare_node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => (
                classify_service_transport(op).expect("service transport class should be present"),
                op.service_call_metadata()
                    .expect("service metadata should be present")
                    .readonly,
            ),
            gunbc_ir::node::NodeBody::SubDag(_) => {
                panic!("expected opaque lowered node for prepare transport")
            }
        };
        assert_eq!(transport_class, ServiceTransportClass::ShellLocal);
        assert!(
            readonly,
            "service-level readonly annotation should be preserved"
        );
    }

    #[test]
    fn service_calls_link_parse_triplet_output_into_caller_dependencies() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/storage_calls.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let parse_node = "parse_transport_sample_services_FsStorage_read";
        let prepare_node = "prepare_transport_sample_services_FsStorage_read";
        let param_source = "param_source_sample_services_run_path";
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == parse_node
                && edge.to_node.0 == "sample.services::run"
                && edge.to_port.0 == "__deps"
        }));
        assert!(dag.nodes.iter().any(|node| node.id.0 == param_source));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == param_source
                && edge.from_port.0 == "path"
                && edge.to_node.0 == prepare_node
                && edge.to_port.0 == "path"
        }));
    }

    #[test]
    fn resource_bound_capability_calls_do_not_raise_unresolved_service_call_errors() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/resource_capability_calls.dag",
            r#"module sample.resources
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "sample.resources::run"));
    }

    #[test]
    fn unresolved_service_call_reports_lower_error() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/unresolved_call.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let error = lower_typed_project(&typed).expect_err("lowering should fail");
        assert!(matches!(
            error,
            LowerError::UnresolvedServiceCall { caller, service_call }
                if caller == "sample.services::run" && service_call == "MissingStorage.read"
        ));
    }

    #[test]
    fn positional_service_call_args_wire_by_operation_input_order() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/storage_calls_positional.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path)
  return { body: response.body }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "param_source_sample_services_run_path"
                && edge.from_port.0 == "path"
                && edge.to_node.0 == "prepare_transport_sample_services_FsStorage_read"
                && edge.to_port.0 == "path"
        }));
    }

    #[test]
    fn resource_acquire_release_lower_to_lifecycle_nodes() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/fs.dag",
            r#"module sample.resources
resource TempFile {
  acquire {
    let path = "/tmp/file"
  }
  release {
    let done = true
  }
}
func run() -> { ok: Bool } {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let acquire = "acquire_resource_sample_resources_TempFile";
        let release = "release_resource_sample_resources_TempFile";
        assert!(dag.nodes.iter().any(|node| node.id.0 == acquire));
        assert!(dag.nodes.iter().any(|node| node.id.0 == release));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == acquire
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == release
                && edge.to_port.0 == "resource_handle"
        }));
    }

    #[test]
    fn uses_clause_wires_acquire_and_release_lifecycle_edges() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/uses_wiring.dag",
            r#"module sample.resources
resource TempFile {
  acquire { let path = "/tmp/file" }
  release { let done = true }
}
func run() -> { ok: Bool } uses fs: TempFile {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let acquire = "acquire_resource_sample_resources_TempFile";
        let release = "release_resource_sample_resources_TempFile";
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == acquire
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == "sample.resources::run"
                && edge.to_port.0 == "__deps"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.resources::run"
                && edge.from_port.0 == "ok"
                && edge.to_node.0 == release
                && edge.to_port.0 == "resource_handle"
        }));
    }

    #[test]
    fn unresolved_uses_clause_reports_lower_error() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/unresolved_uses.dag",
            r#"module sample.resources
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}"#,
        )]);
        let error = lower_typed_project(&typed).expect_err("lowering should fail");
        assert!(matches!(
            error,
            LowerError::UnresolvedUsedResource {
                caller,
                binding,
                resource_type,
            } if caller == "sample.resources::run"
                && binding == "fs"
                && resource_type == "MissingResource"
        ));
    }

    #[test]
    fn uses_clause_with_runtime_config_suffix_resolves_resource_type() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/configured_uses_wiring.dag",
            r#"module sample.resources
resource TempFile {
  acquire { let path = "/tmp/file" }
  release { let done = true }
}
func run() -> { ok: Bool } uses fs: TempFile(mode: ReadWrite) {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_sample_resources_TempFile"
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == "sample.resources::run"
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn uses_interface_with_provider_hint_resolves_matching_resource_lifecycle() {
        let typed = typed_project_from_sources(&[(
            "dsl/infra/providers.dag",
            r#"module infra.providers
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource GcsBucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses store: ObjectStorage(cloud: GcpConfig) {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_providers_GcsBucket"
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == "infra.providers::run"
                && edge.to_port.0 == "__deps"
        }));
        assert!(
            !dag.edges.iter().any(|edge| {
                edge.from_node.0 == "acquire_resource_infra_providers_S3Bucket"
                    && edge.to_node.0 == "infra.providers::run"
                    && edge.to_port.0 == "__deps"
            }),
            "provider hint should avoid wiring non-matching provider resources"
        );
    }

    #[test]
    fn uses_interface_without_provider_hint_stays_unwired_when_multiple_providers_exist() {
        let typed = typed_project_from_sources(&[(
            "dsl/infra/providers_ambiguous.dag",
            r#"module infra.providers
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource GcsBucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses store: ObjectStorage {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(
            !dag.edges.iter().any(|edge| {
                edge.to_node.0 == "infra.providers::run" && edge.to_port.0 == "__deps"
            }),
            "ambiguous interface uses should not arbitrarily wire lifecycle edges"
        );
    }

    #[test]
    fn interface_contract_annotations_lower_to_verification_nodes_for_implementors() {
        let typed = typed_project_from_sources(&[(
            "dsl/infra/contracts.dag",
            r#"module infra.contracts
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
  @contract: read("k") after write("k", "v") => { body: "v" }
  @contract: read("missing") => { body: "" }
}
resource GcsBucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let verify_0 = "verify_contract_infra_contracts_GcsBucket_ObjectStorage_0";
        let verify_1 = "verify_contract_infra_contracts_GcsBucket_ObjectStorage_1";
        assert!(dag.nodes.iter().any(|node| node.id.0 == verify_0));
        assert!(dag.nodes.iter().any(|node| node.id.0 == verify_1));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_contracts_GcsBucket"
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == verify_0
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn provides_clause_adds_provider_node_and_edge() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/provides_wiring.dag",
            r#"module sample.resources
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let provider = "provide_resource_sample_resources_run_out";
        assert!(dag.nodes.iter().any(|node| node.id.0 == provider));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.resources::run"
                && edge.from_port.0 == "ok"
                && edge.to_node.0 == provider
                && edge.to_port.0 == "trigger"
        }));
    }

    #[test]
    fn unresolved_provides_clause_reports_lower_error() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/unresolved_provides.dag",
            r#"module sample.resources
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}"#,
        )]);
        let error = lower_typed_project(&typed).expect_err("lowering should fail");
        assert!(matches!(
            error,
            LowerError::UnresolvedProvidedResource {
                caller,
                binding,
                resource_type,
            } if caller == "sample.resources::run"
                && binding == "out"
                && resource_type == "MissingResource"
        ));
    }

    #[test]
    fn provides_clause_with_runtime_config_suffix_resolves_resource_type() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/configured_provides_wiring.dag",
            r#"module sample.resources
resource TempFile {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides file: TempFile(mode: ReadWrite) {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "provide_resource_sample_resources_run_file"
                && edge.from_port.0 == "file"
                && edge.to_node.0 == "release_resource_sample_resources_TempFile"
                && edge.to_port.0 == "resource_handle"
        }));
    }

    #[test]
    fn provides_resource_with_release_wires_provider_output_to_lifecycle_release() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/provides_release_wiring.dag",
            r#"module sample.resources
resource TempFile {
  release {
    let done = true
  }
}
func run() -> { ok: Bool } provides file: TempFile {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "provide_resource_sample_resources_run_file"
                && edge.from_port.0 == "file"
                && edge.to_node.0 == "release_resource_sample_resources_TempFile"
                && edge.to_port.0 == "resource_handle"
        }));
    }

    #[test]
    fn known_interface_uses_without_lifecycle_are_tolerated() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/interface_uses.dag",
            r#"module sample.resources
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses store: Storage {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "sample.resources::run"));
        assert!(
            !dag.edges.iter().any(|edge| edge.to_node.0 == "sample.resources::run"
                && edge.to_port.0 == "__deps"),
            "interface-only uses should not fabricate lifecycle dependency edges"
        );
    }

    #[test]
    fn known_interface_provides_without_lifecycle_are_tolerated() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/interface_provides.dag",
            r#"module sample.resources
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "provide_resource_sample_resources_run_out"));
        assert!(
            !dag.edges.iter().any(|edge| {
                edge.from_node.0 == "provide_resource_sample_resources_run_out"
                    && edge.to_port.0 == "resource_handle"
            }),
            "interface-only provides should not fabricate lifecycle release edges"
        );
    }

    #[test]
    fn known_std_resource_provides_without_lifecycle_are_tolerated() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/std_resource_provides.dag",
            r#"module sample.resources
func run() -> { ok: Bool } provides auth: AuthContext {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "provide_resource_sample_resources_run_auth"));
        assert!(
            !dag.edges.iter().any(|edge| {
                edge.from_node.0 == "provide_resource_sample_resources_run_auth"
                    && edge.to_port.0 == "resource_handle"
            }),
            "std resource provides should not fabricate lifecycle release edges when lifecycle module is absent"
        );
    }

    #[test]
    fn lower_errors_when_no_callable_items_exist() {
        let typed = typed_project_from_sources(&[(
            "dsl/types_only.dag",
            "module sample.types\ntype Name = String",
        )]);
        let error = lower_typed_project(&typed).expect_err("should fail without callable items");
        assert!(matches!(error, LowerError::NoLowerableItems));
    }

    #[test]
    fn classify_obligation_uses_structural_lowered_metadata() {
        let op = LoweredOp::Callable {
            module: "sample.services".to_string(),
            kind: CallableKind::Pattern,
            name: "prepare_transport_sample".to_string(),
            obligation: ObligationCategory::ServiceTransportPrepare,
            service_metadata: None,
        };
        assert_eq!(
            classify_obligation(&op),
            ObligationCategory::ServiceTransportPrepare
        );

        let pipeline = LoweredOp::Pipeline {
            module: "pipelines.ci".to_string(),
            name: "ci".to_string(),
            stages: 4,
            stage_names: vec![
                "cloud_env".to_string(),
                "codegen_stage".to_string(),
                "deps_check".to_string(),
                "generate".to_string(),
            ],
        };
        assert_eq!(classify_obligation(&pipeline), ObligationCategory::None);
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn makegen_parity_report_is_deterministic() {
        let lowered = load_makegen_lowered();
        let reference = reference_makegen_shape();

        let report_a = compare_topology(&lowered, &reference);
        let report_b = compare_topology(&lowered, &reference);
        assert_eq!(report_a, report_b, "parity reports must be deterministic");
        assert!(
            !report_a.is_exact_match(),
            "phase-1 scaffold should still report makegen parity deltas"
        );
        assert!(
            report_a.added_nodes + report_a.removed_nodes + report_a.changed_nodes > 0,
            "parity report should continue surfacing remaining topology differences"
        );
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn makegen_normalized_parity_can_reach_exact_match() {
        let lowered = load_makegen_lowered();
        let reference = reference_makegen_shape();

        let report = compare_makegen_topology(&lowered, &reference);
        assert!(
            report.is_exact_match(),
            "normalized makegen parity should currently match reference topology"
        );
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn makegen_parity_matches_builder_graph() {
        let lowered = load_makegen_lowered();
        let builder = build_makegen_graph().expect("builder makegen graph should construct");
        let report = compare_makegen_topology(&lowered, &builder);
        assert!(
            report.is_exact_match(),
            "compiled makegen graph should match builder graph topology: {report:?}"
        );
    }

    #[test]
    fn makegen_builder_parity_report_is_deterministic() {
        let lowered = load_makegen_lowered();
        let builder = build_makegen_graph().expect("builder makegen graph should construct");
        let report_a = compare_makegen_topology(&lowered, &builder);
        let report_b = compare_makegen_topology(&lowered, &builder);
        assert_eq!(
            report_a, report_b,
            "builder parity report must be deterministic across runs"
        );
        assert!(
            report_a.is_exact_match(),
            "builder parity report should remain exact-match for makegen"
        );
    }

    #[test]
    fn makegen_builder_and_compiled_ascii_viz_match_after_normalization() {
        let lowered = load_makegen_lowered();
        let builder = build_makegen_graph().expect("builder makegen graph should construct");
        let candidate = normalize_makegen_candidate(&lowered);
        let reference = normalize_makegen_reference(&builder);
        let candidate_ascii = candidate.to_ascii("compiled_makegen");
        let reference_ascii = reference.to_ascii("builder_makegen");
        let normalized_candidate_ascii = candidate_ascii
            .replace("compiled_makegen", "makegen_parity")
            .trim()
            .to_string();
        let normalized_reference_ascii = reference_ascii
            .replace("builder_makegen", "makegen_parity")
            .trim()
            .to_string();
        assert_eq!(
            normalized_candidate_ascii, normalized_reference_ascii,
            "normalized compiled and builder ASCII DAG views should match"
        );
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn makegen_canonical_ir_json_matches_snapshot() {
        let lowered = load_makegen_lowered();
        let canonical = canonical_ir_json(&lowered).expect("canonical json should serialize");
        let expected = include_str!("../tests/snapshots/makegen_canonical_ir.json");
        assert_eq!(canonical.trim(), expected.trim());
    }

    #[test]
    fn canonical_ir_json_is_deterministic_for_makegen() {
        let lowered = load_makegen_lowered();
        let first = canonical_ir_json(&lowered).expect("first canonical json run should serialize");
        let second =
            canonical_ir_json(&lowered).expect("second canonical json run should serialize");
        assert_eq!(first, second, "canonical IR json must be byte-stable");
    }

    #[test]
    fn parity_report_includes_changed_node_details() {
        let mut candidate = Dag::new();
        candidate.add_node(Node::opaque(
            "render",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("content", "String")],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Fn,
                name: "render".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        let mut reference = Dag::new();
        reference.add_node(Node::opaque(
            "render",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("makefile", "String")],
            (),
        ));

        let report = compare_ir(&candidate, &reference);
        assert_eq!(report.changed_nodes, 1);
        assert_eq!(report.changed_node_details.len(), 1);
        assert_eq!(report.changed_node_details[0].node_id, "render");
        assert!(report.changed_node_details[0]
            .differences
            .iter()
            .any(|difference| difference.contains("output ports differ")));
    }

    #[test]
    fn parity_report_lists_added_and_removed_items_in_sorted_order() {
        let mut candidate = Dag::new();
        candidate.add_node(Node::opaque(
            "b",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "sample".to_string(),
                kind: CallableKind::Fn,
                name: "b".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        candidate.add_node(Node::opaque(
            "a",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "sample".to_string(),
                kind: CallableKind::Fn,
                name: "a".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        candidate.add_edge(Edge::new("a", "out", "b", "in"));

        let mut reference = Dag::new();
        reference.add_node(Node::opaque(
            "c",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            (),
        ));
        reference.add_node(Node::opaque(
            "d",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            (),
        ));
        reference.add_edge(Edge::new("c", "out", "d", "in"));

        let report = compare_ir(&candidate, &reference);
        assert_eq!(
            report.added_node_ids,
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            report.removed_node_ids,
            vec!["c".to_string(), "d".to_string()]
        );
        assert_eq!(
            report.added_edge_ids,
            vec!["a.out->b.in".to_string()],
            "added edges should be deterministic and sorted"
        );
        assert_eq!(
            report.removed_edge_ids,
            vec!["c.out->d.in".to_string()],
            "removed edges should be deterministic and sorted"
        );
    }

    #[allow(clippy::disallowed_methods)]
    fn load_makegen_lowered() -> Dag<LoweredOp> {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl/tools/makegen.dag");
        let source = fs::read_to_string(file).expect("should read makegen source");
        let typed = typed_project_from_sources(&[("dsl/tools/makegen.dag", &source)]);
        lower_typed_project(&typed).expect("lowering should succeed")
    }

    fn reference_makegen_shape() -> Dag<()> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "fs_env",
            vec![],
            vec![Port::scalar("FilesystemHandle", "FilesystemHandle")],
            (),
        ));
        dag.add_node(Node::opaque(
            "load_registry",
            vec![],
            vec![Port::scalar("registry", "ToolRegistry")],
            (),
        ));
        dag.add_node(Node::opaque(
            "render_makefile",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("makefile_content", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "prepare_read_makegen",
            vec![
                Port::scalar("path", "String"),
                Port::scalar("res:file:Makefile", "FilesystemHandle"),
            ],
            vec![Port::scalar("request", "TransportRequest")],
            (),
        ));
        dag.add_node(Node::opaque(
            "execute_read_makegen",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            (),
        ));
        dag.add_node(Node::opaque(
            "compare_makegen_content",
            vec![
                Port::scalar("expected_content", "String"),
                Port::scalar("response", "TransportResponse"),
            ],
            vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
            (),
        ));
        dag.add_node(Node::opaque(
            "prepare_write_makegen",
            vec![
                Port::scalar("content", "String"),
                Port::scalar("path", "String"),
            ],
            vec![Port::scalar("request", "TransportRequest")],
            (),
        ));
        dag.add_node(Node::opaque(
            "execute_makegen_transport",
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("skip", "Bool"),
            ],
            vec![Port::scalar("makegen_response", "TransportResponse")],
            (),
        ));

        dag.add_edge(Edge::new(
            "load_registry",
            "registry",
            "render_makefile",
            "registry",
        ));
        dag.add_edge(Edge::new(
            "render_makefile",
            "makefile_content",
            "compare_makegen_content",
            "expected_content",
        ));
        dag.add_edge(Edge::new(
            "render_makefile",
            "makefile_content",
            "prepare_write_makegen",
            "content",
        ));
        dag.add_edge(Edge::new(
            "prepare_read_makegen",
            "request",
            "execute_read_makegen",
            "request",
        ));
        dag.add_edge(Edge::new(
            "execute_read_makegen",
            "response",
            "compare_makegen_content",
            "response",
        ));
        dag.add_edge(Edge::new(
            "prepare_write_makegen",
            "request",
            "execute_makegen_transport",
            "request",
        ));
        dag.add_edge(Edge::new(
            "compare_makegen_content",
            "skip",
            "execute_makegen_transport",
            "skip",
        ));
        dag
    }
}
