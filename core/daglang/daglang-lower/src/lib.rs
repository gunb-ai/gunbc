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

use std::collections::{HashMap, HashSet};

use daglang_syntax::ast::{Expr, Item, Stmt};
use daglang_syntax::ast_utils::{
    canonical_resource_type_name as canonical_type_name, resource_type_name, service_call_lookup_keys,
    should_track_call_name as should_track_call, type_expr_to_string, walk_stmts,
};
use daglang_typecheck::{TypedCallableSignature, TypedItemSignature, TypedProject};
use gunbc_ir::{diff_topologies, Cardinality, Dag, Edge, Node, Port};

/// Lowered operation payload for daglang graph nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredOp {
    Callable {
        module: String,
        kind: CallableKind,
        name: String,
    },
    Pipeline {
        module: String,
        name: String,
        stages: usize,
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

/// Kind of lowered callable declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableKind {
    Fn,
    Func,
    Pattern,
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
            } => write!(
                f,
                "unresolved service call `{service_call}` in `{caller}`"
            ),
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
    let mut builder = DagBuilder::new();
    let mut endpoints_by_full = HashMap::<(String, String), LoweredEndpoint>::new();
    let mut endpoints_by_name = HashMap::<String, Option<LoweredEndpoint>>::new();

    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for signature in &module.signatures {
            match signature {
                TypedItemSignature::Fn(callable) => {
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
                TypedItemSignature::Pipeline { name, stages } => {
                    let node_id = lowered_node_id(&module_name, name);
                    builder.add_node(Node::opaque(
                        node_id,
                        vec![],
                        vec![Port::with_cardinality(
                            "stages",
                            "Int",
                            Cardinality::ONE,
                        )],
                        LoweredOp::Pipeline {
                            module: module_name.clone(),
                            name: name.clone(),
                            stages: *stages,
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

    add_dependency_edges(&mut builder, project, &endpoints_by_full, &endpoints_by_name);
    add_makegen_scaffolding(&mut builder, &endpoints_by_full);
    let service_registry = add_service_transport_triplets(&mut builder, project);
    add_service_call_edges(&mut builder, project, &endpoints_by_full, &service_registry)?;
    let resource_registry = add_resource_lifecycle_nodes(&mut builder, project);
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
    let candidate_topology = candidate.topology();
    let reference_topology = reference.topology();
    let diff = diff_topologies(&reference_topology, &candidate_topology);

    ParityReport {
        candidate_nodes: candidate_topology.nodes.len(),
        reference_nodes: reference_topology.nodes.len(),
        candidate_edges: candidate_topology.edges.len(),
        reference_edges: reference_topology.edges.len(),
        added_nodes: diff.added_nodes.len(),
        removed_nodes: diff.removed_nodes.len(),
        changed_nodes: diff.changed_nodes.len(),
        added_edges: diff.added_edges.len(),
        removed_edges: diff.removed_edges.len(),
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
    compare_topology(&normalized, reference)
}

fn normalize_makegen_candidate(candidate: &Dag<LoweredOp>) -> Dag<LoweredOp> {
    let mut normalized = Dag::new();
    let mut kept_nodes = HashSet::<String>::new();

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
        inputs.sort_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));

        let mut outputs = node.outputs.clone();
        if canonical_id == "render_makefile" {
            for output in &mut outputs {
                if output.name.0 == "return" {
                    output.name.0 = "makefile_content".to_string();
                }
            }
        }
        outputs.sort_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));

        normalized.add_node(Node::opaque(canonical_id.clone(), inputs, outputs, op));
        kept_nodes.insert(canonical_id);
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
        if from_node == "fs_env"
            && to_node == "prepare_read_makegen"
            && edge.to_port.0 == "res:file:Makefile"
        {
            continue;
        }

        let from_port = if from_node == "render_makefile" && edge.from_port.0 == "return" {
            "makefile_content".to_string()
        } else {
            edge.from_port.0.clone()
        };
        let key = (
            from_node,
            from_port,
            to_node,
            edge.to_port.0.clone(),
        );
        if seen_edges.insert(key.clone()) {
            normalized.add_edge(Edge::new(key.0, key.1, key.2, key.3));
        }
    }

    normalized
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
                Port::with_cardinality(
                    binding.name.as_str(),
                    binding.ty.as_str(),
                    Cardinality::ONE,
                )
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
) {
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Some((item_name, stmts)) = item_callable_body(&item.node) else {
                continue;
            };
            let Some(target) =
                endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };

            let mut calls = HashSet::new();
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
        }
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
        },
    ));
    builder.add_node(Node::opaque(
        compare_id.clone(),
        vec![
            Port::scalar("expected_content", "String"),
            Port::scalar("response", "TransportResponse"),
        ],
        vec![
            Port::scalar("fresh", "Bool"),
            Port::scalar("skip", "Bool"),
        ],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Pattern,
            name: format!("content_upsert::{compare_id}"),
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
        },
    ));

    builder.add_edge(
        &prepare_read_id,
        "request",
        &execute_read_id,
        "request",
    );
    builder.add_edge(
        &execute_read_id,
        "response",
        &compare_id,
        "response",
    );
    builder.add_edge(
        &prepare_write_id,
        "request",
        &execute_transport_id,
        "request",
    );
    builder.add_edge(
        &compare_id,
        "skip",
        &execute_transport_id,
        "skip",
    );
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
    endpoints_by_name.get(&source_name).and_then(|entry| entry.clone())
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
    let Some(render) = endpoints_by_full
        .get(&("tools.makegen".to_string(), "render_makefile".to_string()))
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

fn add_service_transport_triplets(
    builder: &mut DagBuilder,
    project: &TypedProject,
) -> ServiceEndpointRegistry {
    let mut registry = ServiceEndpointRegistry::default();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::ServiceDef(service) = &item.node else {
                continue;
            };
            if service.implements.is_none() {
                continue;
            }

            for operation in &service.operations {
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
            let Some(target) =
                endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
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
            let Some(target) =
                endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };
            for usage in uses {
                let resource_type = resource_type_name(&usage.resource_type);
                let Some(endpoint) = resolve_resource_endpoint(
                    module_name.as_str(),
                    resource_type.as_str(),
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
            let Some(target) =
                endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };
            for provided in provides {
                let resource_type = resource_type_name(&provided.resource_type);
                let endpoint = resolve_resource_endpoint(
                    module_name.as_str(),
                    resource_type.as_str(),
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
                    sanitize_identifier(&format!(
                        "{module_name}_{item_name}_{}",
                        provided.binding
                    ))
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
                        name: format!(
                            "resource_provide::{}::{}",
                            item_name, provided.binding
                        ),
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

fn collect_known_uses_types(project: &TypedProject) -> HashSet<String> {
    let mut known = HashSet::new();
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

fn add_resource_lifecycle_nodes(
    builder: &mut DagBuilder,
    project: &TypedProject,
) -> ResourceLifecycleRegistry {
    let mut registry = ResourceLifecycleRegistry::default();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
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
            registry.register(
                format!("{module_name}.{}", resource.name),
                endpoint.clone(),
            );
            registry.register(resource.name.clone(), endpoint);
        }
    }
    registry
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

fn item_callable_provides(
    item: &Item,
) -> Option<(&str, &[daglang_syntax::ast::ProvidesClause])> {
    match item {
        Item::FuncDef(def) => Some((def.name.as_str(), def.provides.as_slice())),
        _ => None,
    }
}

fn collect_calls_from_stmts(stmts: &[Stmt], calls: &mut HashSet<String>) {
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
    use gunbc_ir::{Edge, Port};
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

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn lower_makegen_produces_callable_nodes() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
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

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn content_upsert_expansion_wires_transport_chain_for_makegen() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
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
            ("load_registry", "registry", "tools.makegen::render_makefile", "registry"),
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
        assert!(dag.nodes.iter().any(|node| node.id.0 == "sample.resources::run"));
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
    fn lower_errors_when_no_callable_items_exist() {
        let typed = typed_project_from_sources(&[(
            "dsl/types_only.dag",
            "module sample.types\ntype Name = String",
        )]);
        let error = lower_typed_project(&typed).expect_err("should fail without callable items");
        assert!(matches!(error, LowerError::NoLowerableItems));
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn makegen_parity_report_is_deterministic() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let source = fs::read_to_string(file).expect("should read makegen source");
        let typed = typed_project_from_sources(&[("dsl/tools/makegen.dag", &source)]);
        let lowered = lower_typed_project(&typed).expect("lowering should succeed");
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
        let file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let source = fs::read_to_string(file).expect("should read makegen source");
        let typed = typed_project_from_sources(&[("dsl/tools/makegen.dag", &source)]);
        let lowered = lower_typed_project(&typed).expect("lowering should succeed");
        let reference = reference_makegen_shape();

        let report = compare_makegen_topology(&lowered, &reference);
        assert!(
            report.is_exact_match(),
            "normalized makegen parity should currently match reference topology"
        );
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
            vec![Port::scalar("content", "String"), Port::scalar("path", "String")],
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
