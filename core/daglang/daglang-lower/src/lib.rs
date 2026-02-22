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
    canonical_resource_type_name, resource_type_name,
    service_call_lookup_keys, should_track_call_name as should_track_call, type_expr_to_string,
    walk_stmts,
};
use daglang_typecheck::{TypedCallableSignature, TypedItemSignature, TypedProject};
use gunbc_ir::patterns::branch::IfBuilder;
use gunbc_ir::patterns::{BranchBuilder, LoopBuilder, PatternOp};
use gunbc_ir::resource::AccessMode;
use gunbc_ir::{Cardinality, Dag, DagTopology, Edge, Node, Port, TypeId};
use serde::Serialize;

/// Lowered operation payload for daglang graph nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredOp {
    Callable {
        module: String,
        kind: CallableKind,
        name: String,
        obligation: ObligationCategory,
        service_metadata: Option<Box<ServiceCallMetadata>>,
        is_interactive: bool,
        resource_target: Option<String>,
    },
    Primitive {
        module: String,
        name: String,
        kind: PrimitiveOpKind,
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
    LoopUnpack {
        input_port: String,
        element_port: String,
    },
    LoopPack {
        output_port: String,
    },
    BranchMerge {
        output_port: String,
    },
}

impl From<PatternOp> for LoweredOp {
    fn from(op: PatternOp) -> Self {
        match op {
            PatternOp::LoopUnpack {
                input_port,
                element_port,
            } => LoweredOp::LoopUnpack {
                input_port,
                element_port,
            },
            PatternOp::LoopPack { output_port } => LoweredOp::LoopPack { output_port },
            PatternOp::BranchMerge { output_port } => LoweredOp::BranchMerge { output_port },
            other => panic!("unsupported PatternOp in daglang lowering: {other:?}"),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveOpKind {
    FsEnv,
    CallParamSource { callable: String, param: String },
    CallLiteralSource { literal: PrimitiveLiteral },
    IoPrepareFileRead,
    IoExecuteFileRead,
    CompareEquality,
    IoPrepareFileWrite,
    IoExecuteFileWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveLiteral {
    String(String),
    Int(i64),
    Bool(bool),
    Unit,
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
    /// Full protocol spec extracted from DSL annotations.
    /// Used by generic protocol interpreters to replace per-service adapters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<ServiceOperationSpec>,
}

// ============================================================================
// Service Operation Spec — protocol interface parameterization
// ============================================================================

/// Complete specification for a service operation, extracted from `.dag` annotations.
/// Each variant parameterizes a generic protocol interpreter (REST, Shell, File).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ServiceOperationSpec {
    Rest(RestOperationSpec),
    Shell(ShellOperationSpec),
}

/// REST protocol specification: endpoint + method + path + body + response.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RestOperationSpec {
    /// Base URL from `@endpoint("https://...")` on the service.
    pub endpoint: String,
    /// HTTP method from `@rest(METHOD, ...)`.
    pub method: String,
    /// URL path template from `@rest(..., "/path/{param}")`.
    pub path_template: String,
    /// Input fields from `input { ... }`.
    pub input_fields: Vec<FieldSpec>,
    /// Output fields from `output { ... @json("key") }`.
    pub output_fields: Vec<OutputFieldSpec>,
    /// Explicit body template from `@body_template({...})`, if present.
    /// When None, body is built from all non-path input fields.
    pub body_template: Option<Vec<BodyEntry>>,
    /// Extra HTTP headers from `@headers({...})`.
    pub headers: Vec<(String, String)>,
}

/// Shell protocol specification: argv template + output parsing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ShellOperationSpec {
    /// Command + args template from `@shell(["cmd", "arg", "{param}"])`.
    pub argv_template: Vec<ArgvSegment>,
    /// Input fields from `input { ... }`.
    pub input_fields: Vec<FieldSpec>,
    /// Output fields from `output { ... }`.
    pub output_fields: Vec<OutputFieldSpec>,
    /// How to parse the shell response.
    pub output_parsing: ShellOutputParsing,
    /// Whether this shell operation should run in passthrough mode.
    ///
    /// Derived from `@external`/`@interactive` annotations and used to
    /// preserve interactive terminal handoff semantics at runtime.
    pub passthrough: bool,
}

/// Specification for an input field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FieldSpec {
    pub name: String,
    pub type_id: String,
    pub default: Option<String>,
    pub is_secret: bool,
    /// True if this field appears as `{name}` in the path/argv template.
    pub is_path_param: bool,
}

/// Specification for an output field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct OutputFieldSpec {
    pub name: String,
    pub type_id: String,
    /// JSON pointer path for extraction (from `@json("key")` or field name).
    pub json_path: String,
    pub is_secret: bool,
    /// True if this field uses `@raw_body` (response body as raw string).
    pub is_raw_body: bool,
}

/// Body template entry: either a literal constant or a reference to an input field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum BodyEntry {
    /// Literal JSON key-value: `"grant_type": "urn:ietf:..."`.
    Literal(String, String),
    /// Reference to an input field: `"audience": audience`.
    InputRef(String, String),
}

/// Argv segment in a shell command template.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ArgvSegment {
    /// Literal string: `"cargo"`, `"--all-targets"`.
    Literal(String),
    /// Input field interpolation: `"{package}"`.
    InputRef(String),
}

/// How to parse shell command output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ShellOutputParsing {
    /// Single string: `trim(stdout)`.
    TrimStdout,
    /// List of strings: `split(trim(stdout), "\n")`.
    SplitLines,
    /// Standard triple: `(success: Bool, stdout: String, stderr: String)`.
    SuccessStdoutStderr,
    /// Bool from exit code: `success = exit_code == 0`.
    ExitCodeBool,
}

impl LoweredOp {
    pub fn obligation_category(&self) -> ObligationCategory {
        match self {
            Self::Callable { obligation, .. } => *obligation,
            Self::Primitive { kind, .. } => kind.obligation_category(),
            Self::Collection { .. }
            | Self::Pipeline { .. }
            | Self::LoopUnpack { .. }
            | Self::LoopPack { .. }
            | Self::BranchMerge { .. } => ObligationCategory::None,
        }
    }

    pub fn service_call_metadata(&self) -> Option<&ServiceCallMetadata> {
        match self {
            Self::Callable {
                service_metadata, ..
            } => service_metadata.as_deref(),
            Self::Primitive { .. }
            | Self::Collection { .. }
            | Self::Pipeline { .. }
            | Self::LoopUnpack { .. }
            | Self::LoopPack { .. }
            | Self::BranchMerge { .. } => None,
        }
    }
}

impl PrimitiveOpKind {
    pub fn obligation_category(&self) -> ObligationCategory {
        match self {
            Self::FsEnv => ObligationCategory::ResourceProvide,
            Self::CallParamSource { .. } | Self::CallLiteralSource { .. } => {
                ObligationCategory::ServiceParamSource
            }
            Self::IoPrepareFileRead | Self::IoPrepareFileWrite => {
                ObligationCategory::ServiceTransportPrepare
            }
            Self::IoExecuteFileRead | Self::IoExecuteFileWrite => {
                ObligationCategory::ServiceTransportExecute
            }
            Self::CompareEquality => ObligationCategory::InterfaceContractVerification,
        }
    }
}

pub fn classify_obligation(op: &LoweredOp) -> ObligationCategory {
    op.obligation_category()
}

/// Map lowered obligation categories to canonical parity-kind strings.
///
/// Returns `None` for unconstrained callables; callers can fall back to
/// shape/name-derived classification in those cases.
pub fn canonical_kind_for_obligation(obligation: ObligationCategory) -> Option<&'static str> {
    match obligation {
        ObligationCategory::None => None,
        ObligationCategory::ServiceTransportExecute => Some("transport"),
        ObligationCategory::ServiceTransportPrepare
        | ObligationCategory::ServiceTransportParse
        | ObligationCategory::ServiceParamSource
        | ObligationCategory::ResourceProvide
        | ObligationCategory::ResourceAcquire
        | ObligationCategory::ResourceRelease
        | ObligationCategory::InterfaceContractVerification => Some("pattern-expanded"),
    }
}

pub fn classify_service_transport(op: &LoweredOp) -> Option<ServiceTransportClass> {
    op.service_call_metadata()
        .map(|metadata| metadata.transport)
}

/// Extract DAG topology with canonical obligation-kind metadata on each node.
///
/// This is the preferred topology form for renderers (e.g. Mermaid) that need
/// stable semantic classes without depending on fragile node-id prefixes.
pub fn topology_with_obligation_kinds(dag: &Dag<LoweredOp>) -> DagTopology {
    dag.topology_with_kind(|node| match &node.body {
        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable { obligation, .. }) => {
            canonical_kind_for_obligation(*obligation).map(str::to_string)
        }
        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive { kind, .. }) => {
            canonical_kind_for_obligation(kind.obligation_category()).map(str::to_string)
        }
        gunbc_ir::node::NodeBody::Opaque(_) | gunbc_ir::node::NodeBody::SubDag(_) => None,
    })
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

#[derive(Debug, Clone, Default)]
struct ProfileBindingRegistry {
    by_full_name: HashMap<String, HashMap<String, String>>,
    by_short_name: HashMap<String, Option<String>>,
}

type ActiveProfileBindings = (String, HashMap<String, String>);

/// Cloud provider classification for resource/interface resolution.
///
/// Provider hints come from explicit DSL structure:
/// - `uses ... (cloud: GcpConfig|AwsConfig|AzureConfig)`
/// - optional resource properties (`provider: Gcp|Aws|Azure`)
/// - exact module path segments (`.gcp.`, `.aws.`, `.azure.`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderHint {
    Gcp,
    Aws,
    Azure,
}

fn provider_hint_from_symbol(name: &str) -> Option<ProviderHint> {
    let tail = name.rsplit('.').next().unwrap_or(name);
    match tail {
        "Gcp" | "GcpConfig" => Some(ProviderHint::Gcp),
        "Aws" | "AwsConfig" => Some(ProviderHint::Aws),
        "Azure" | "AzureConfig" => Some(ProviderHint::Azure),
        _ => None,
    }
}

fn provider_hint_from_expr(expr: &Expr) -> Option<ProviderHint> {
    match expr {
        Expr::Ident(name) | Expr::Call(name, _) => provider_hint_from_symbol(name),
        Expr::Record(name, _) => name.as_deref().and_then(provider_hint_from_symbol),
        Expr::FieldAccess(_, field) => provider_hint_from_symbol(field),
        _ => None,
    }
}

fn provider_hint_from_resource_type_config(resource_type: &str) -> Option<ProviderHint> {
    let open = resource_type.find('(')?;
    let close = resource_type.rfind(')')?;
    if close <= open {
        return None;
    }
    let config = &resource_type[open + 1..close];
    for entry in split_top_level_csv(config) {
        let Some((name, value)) = entry.split_once(':') else {
            continue;
        };
        if name.trim() != "cloud" {
            continue;
        }
        if let Some(provider) = provider_hint_from_symbol(parse_leading_symbol(value.trim())) {
            return Some(provider);
        }
    }
    None
}

fn split_top_level_csv(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut angle_depth = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            ',' if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && angle_depth == 0 =>
            {
                parts.push(input[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if start <= input.len() {
        parts.push(input[start..].trim());
    }
    parts
}

fn parse_leading_symbol(text: &str) -> &str {
    let end = text
        .char_indices()
        .find_map(|(index, ch)| {
            (ch.is_whitespace() || ch == '(' || ch == '{' || ch == '[' || ch == ',')
                .then_some(index)
        })
        .unwrap_or(text.len());
    &text[..end]
}

fn provider_hint_from_uses_config(config: Option<&[(String, Expr)]>) -> Option<ProviderHint> {
    let config_entries = config?;
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
    let mut found = None;
    for segment in module_name.split('.') {
        let hint = match segment {
            "gcp" => Some(ProviderHint::Gcp),
            "aws" => Some(ProviderHint::Aws),
            "azure" => Some(ProviderHint::Azure),
            _ => None,
        };
        if let Some(hint) = hint {
            if found.is_some_and(|existing| existing != hint) {
                return None;
            }
            found = Some(hint);
        }
    }
    found
}

fn provider_hint_from_resource_properties(properties: &[(String, Expr)]) -> Option<ProviderHint> {
    for (name, value) in properties {
        if name == "provider" || name == "cloud" {
            if let Some(provider) = provider_hint_from_expr(value) {
                return Some(provider);
            }
        }
    }
    None
}

fn canonical_type_id(name: &str) -> TypeId {
    let canonical = canonical_resource_type_name(name);
    TypeId::from(canonical.as_str())
}

fn short_type_name(type_id: &TypeId) -> &str {
    type_id
        .0
        .rsplit('.')
        .next()
        .unwrap_or(type_id.0.as_str())
}

fn insert_canonical_names(set: &mut HashSet<String>, name: &str) {
    let canonical = canonical_type_id(name);
    set.insert(canonical.0.clone());
    let short = short_type_name(&canonical).to_string();
    set.insert(short);
}

fn collect_profile_binding_registry(project: &TypedProject) -> ProfileBindingRegistry {
    let mut registry = ProfileBindingRegistry::default();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::ProfileDef(profile) = &item.node else {
                continue;
            };
            let full_name = format!("{module_name}.{}", profile.name);
            let mut binds = HashMap::<String, String>::new();
            for bind in &profile.binds {
                let canonical_interface = canonical_type_id(bind.interface.as_str());
                binds.insert(canonical_interface.0.clone(), bind.implementation.clone());
                let short_interface = short_type_name(&canonical_interface).to_string();
                binds.insert(short_interface, bind.implementation.clone());
            }
            registry.by_full_name.insert(full_name.clone(), binds);
            registry
                .by_short_name
                .entry(profile.name.clone())
                .and_modify(|existing| {
                    if existing.is_some() {
                        *existing = None;
                    }
                })
                .or_insert_with(|| Some(full_name));
        }
    }
    registry
}

fn resolve_active_profile_bindings(
    active_profile: Option<&str>,
    registry: &ProfileBindingRegistry,
) -> Result<Option<ActiveProfileBindings>, LowerError> {
    let Some(profile_name) = active_profile else {
        return Ok(None);
    };
    if let Some(bindings) = registry.by_full_name.get(profile_name) {
        return Ok(Some((profile_name.to_string(), bindings.clone())));
    }
    match registry.by_short_name.get(profile_name) {
        Some(Some(full_name)) => registry
            .by_full_name
            .get(full_name)
            .cloned()
            .map(|bindings| Some((full_name.clone(), bindings)))
            .ok_or_else(|| LowerError::UnknownProfile {
                profile: profile_name.to_string(),
            }),
        Some(None) => Err(LowerError::AmbiguousProfile {
            profile: profile_name.to_string(),
        }),
        None => Err(LowerError::UnknownProfile {
            profile: profile_name.to_string(),
        }),
    }
}

fn is_known_uses_type(set: &HashSet<String>, name: &str) -> bool {
    let canonical = canonical_type_id(name);
    set.contains(canonical.0.as_str()) || set.contains(short_type_name(&canonical))
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
    /// `uses` clause resolves to multiple resource/interface lifecycle sources.
    AmbiguousUsedResource {
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
    /// `provides` clause resolves to multiple resource/interface lifecycle sources.
    AmbiguousProvidedResource {
        caller: String,
        binding: String,
        resource_type: String,
    },
    /// The requested active profile was not found.
    UnknownProfile { profile: String },
    /// The requested active profile name resolved ambiguously.
    AmbiguousProfile { profile: String },
    /// A `uses` interface binding has no mapping in the active profile.
    MissingProfileBinding {
        caller: String,
        binding: String,
        interface: String,
        profile: String,
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
            Self::AmbiguousUsedResource {
                caller,
                binding,
                resource_type,
            } => write!(
                f,
                "ambiguous used resource `{binding}: {resource_type}` in `{caller}`; add explicit `cloud: GcpConfig|AwsConfig|AzureConfig`"
            ),
            Self::UnresolvedProvidedResource {
                caller,
                binding,
                resource_type,
            } => write!(
                f,
                "unresolved provided resource `{binding}: {resource_type}` in `{caller}`"
            ),
            Self::AmbiguousProvidedResource {
                caller,
                binding,
                resource_type,
            } => write!(
                f,
                "ambiguous provided resource `{binding}: {resource_type}` in `{caller}`; use a concrete resource type"
            ),
            Self::UnknownProfile { profile } => {
                write!(f, "unknown active profile `{profile}`")
            }
            Self::AmbiguousProfile { profile } => {
                write!(f, "ambiguous active profile `{profile}`")
            }
            Self::MissingProfileBinding {
                caller,
                binding,
                interface,
                profile,
            } => write!(
                f,
                "missing profile binding for `{binding}: {interface}` in `{caller}` (active profile `{profile}`)"
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
    lower_typed_project_with_callable_scope(project, None, false, None)
}

/// Lowers typed modules while emitting explicit collection pipeline nodes.
pub fn lower_typed_project_with_collection_nodes(
    project: &TypedProject,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, true, None)
}

pub fn lower_typed_project_for_modules(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), false, None)
}

/// Lowers only scoped modules while emitting explicit collection pipeline nodes.
pub fn lower_typed_project_for_modules_with_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), true, None)
}

pub fn lower_typed_project_with_profile(
    project: &TypedProject,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, false, active_profile)
}

pub fn lower_typed_project_with_collection_nodes_and_profile(
    project: &TypedProject,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, true, active_profile)
}

pub fn lower_typed_project_for_modules_with_profile(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), false, active_profile)
}

pub fn lower_typed_project_for_modules_with_collection_nodes_and_profile(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), true, active_profile)
}

fn lower_typed_project_with_callable_scope(
    project: &TypedProject,
    callable_modules: Option<&HashSet<String>>,
    emit_collection_nodes: bool,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    let mut builder = DagBuilder::new();
    let mut endpoints_by_full = HashMap::<(String, String), LoweredEndpoint>::new();
    let mut endpoints_by_name = HashMap::<String, Option<LoweredEndpoint>>::new();

    for module in &project.modules {
        let module_name = module.module_path.join(".");
        let include_callables = callable_modules
            .map(|scope| scope.contains(&module_name))
            .unwrap_or(true);
        let interactive_by_callable = module
            .ast
            .items
            .iter()
            .filter_map(|item| item_callable_interactive_flag(&item.node))
            .map(|(name, is_interactive)| (name.to_string(), is_interactive))
            .collect::<HashMap<_, _>>();
        for signature in &module.signatures {
            match signature {
                TypedItemSignature::Fn(callable) => {
                    if !include_callables {
                        continue;
                    }
                    let (node, endpoint) = lower_callable(
                        callable,
                        &module_name,
                        CallableKind::Fn,
                        *interactive_by_callable
                            .get(callable.name.as_str())
                            .unwrap_or(&false),
                    );
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
                    let (node, endpoint) = lower_callable(
                        callable,
                        &module_name,
                        CallableKind::Func,
                        *interactive_by_callable
                            .get(callable.name.as_str())
                            .unwrap_or(&false),
                    );
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
                    let (node, endpoint) = lower_callable(
                        callable,
                        &module_name,
                        CallableKind::Pattern,
                        *interactive_by_callable
                            .get(callable.name.as_str())
                            .unwrap_or(&false),
                    );
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
                | TypedItemSignature::Interface { .. }
                | TypedItemSignature::Profile { .. } => {}
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
    let service_registry = if callable_modules.is_some() && active_profile.is_none() {
        let required_service_calls = collect_required_service_call_keys(project, callable_modules);
        add_service_transport_triplets(&mut builder, project, Some(&required_service_calls))
    } else {
        add_service_transport_triplets(&mut builder, project, None)
    };
    let profile_registry = collect_profile_binding_registry(project);
    let active_profile_bindings =
        resolve_active_profile_bindings(active_profile, &profile_registry)?;
    let known_interface_types = collect_interface_type_names(project);
    add_service_call_edges(
        &mut builder,
        project,
        &endpoints_by_full,
        &service_registry,
        active_profile_bindings.as_ref(),
        &known_interface_types,
    )?;
    let resource_registry = add_resource_lifecycle_nodes(&mut builder, project, callable_modules);
    let known_uses_types = collect_known_uses_types(project);
    let mut wired_release_targets = HashSet::new();
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
        &mut wired_release_targets,
    )?;
    add_interface_contract_verification_nodes(&mut builder, project, &resource_registry);

    if builder.dag.nodes.is_empty() {
        return Err(LowerError::NoLowerableItems);
    }

    Ok(builder.into_dag())
}

pub use parity::{
    canonical_ir_json, compare_ci_topology, compare_gcp_credential_topology, compare_gist_topology,
    compare_ir, compare_makegen_topology, compare_topology, GistParityMode,
};
#[cfg(test)]
use parity::{normalize_makegen_candidate, normalize_makegen_reference};

mod parity {
    use super::*;

    /// Compare a lowered daglang graph against a reference graph topology.
    ///
    /// This enables incremental parity harness adoption:
    /// - exact parity: `report.is_exact_match() == true`
    /// - scaffold mode: report still gives deterministic deltas while lowering
    ///   coverage grows.
    pub fn compare_topology<T>(candidate: &Dag<LoweredOp>, reference: &Dag<T>) -> ParityReport {
        let candidate_node_ids = candidate
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<BTreeSet<_>>();
        let reference_node_ids = reference
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<BTreeSet<_>>();

        let added_node_ids = candidate_node_ids
            .difference(&reference_node_ids)
            .cloned()
            .collect::<Vec<_>>();
        let removed_node_ids = reference_node_ids
            .difference(&candidate_node_ids)
            .cloned()
            .collect::<Vec<_>>();

        let candidate_edge_ids = candidate
            .edges
            .iter()
            .map(|edge| {
                format!(
                    "{}.{}->{}.{}",
                    edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
                )
            })
            .collect::<BTreeSet<_>>();
        let reference_edge_ids = reference
            .edges
            .iter()
            .map(|edge| {
                format!(
                    "{}.{}->{}.{}",
                    edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
                )
            })
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
            candidate_nodes: candidate.nodes.len(),
            reference_nodes: reference.nodes.len(),
            candidate_edges: candidate.edges.len(),
            reference_edges: reference.edges.len(),
            added_nodes: added_node_ids.len(),
            removed_nodes: removed_node_ids.len(),
            changed_nodes: 0,
            added_edges: added_edge_ids.len(),
            removed_edges: removed_edge_ids.len(),
            added_node_ids,
            removed_node_ids,
            changed_node_details: Vec::new(),
            added_edge_ids,
            removed_edge_ids,
        }
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
    pub fn compare_makegen_topology<T>(
        candidate: &Dag<LoweredOp>,
        reference: &Dag<T>,
    ) -> ParityReport {
        let normalized = normalize_makegen_candidate(candidate);
        let normalized_reference = normalize_makegen_reference(reference);
        compare_topology(&normalized, &normalized_reference)
    }

    /// Compare GCP credential topology against the legacy graph shape.
    ///
    /// The compiler currently emits higher-level pattern/resource scaffolding for
    /// credential-chain callables; the legacy builder expresses this flow as a
    /// concrete transport-step graph. This comparator projects both graphs into the
    /// same canonical 15-node credential-chain shape so structural parity stays
    /// deterministic and reviewable while lowering remains staged.
    pub fn compare_gcp_credential_topology<T>(
        candidate: &Dag<LoweredOp>,
        reference: &Dag<T>,
    ) -> ParityReport {
        let normalized_candidate = normalize_gcp_credential_candidate(candidate);
        let normalized_reference = normalize_gcp_credential_reference(reference);
        compare_ir(&normalized_candidate, &normalized_reference)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum GistParityMode {
        Snapshot,
        Diff,
        Recent,
    }

    pub fn compare_gist_topology<T>(
        candidate: &Dag<LoweredOp>,
        reference: &Dag<T>,
        mode: GistParityMode,
    ) -> ParityReport {
        let normalized_candidate = normalize_gist_candidate(candidate, mode);
        let normalized_reference = normalize_gist_reference(reference, mode);
        compare_topology(&normalized_candidate, &normalized_reference)
    }

    pub fn compare_ci_topology<T>(candidate: &Dag<LoweredOp>, reference: &Dag<T>) -> ParityReport {
        let normalized_candidate = normalize_ci_candidate(candidate);
        let normalized_reference = normalize_ci_reference(reference);
        compare_topology(&normalized_candidate, &normalized_reference)
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
                canonical_kind_from_shape(&node.id.0, &node.inputs, &node.outputs, true, None)
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection { kind, .. }) => {
                collection_kind_node_label(*kind).to_string()
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable { obligation, .. }) => {
                canonical_kind_from_shape(
                    &node.id.0,
                    &node.inputs,
                    &node.outputs,
                    false,
                    Some(*obligation),
                )
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive { kind, .. }) => {
                canonical_kind_from_shape(
                    &node.id.0,
                    &node.inputs,
                    &node.outputs,
                    false,
                    Some(kind.obligation_category()),
                )
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::LoopUnpack { .. })
            | gunbc_ir::node::NodeBody::Opaque(LoweredOp::LoopPack { .. })
            | gunbc_ir::node::NodeBody::Opaque(LoweredOp::BranchMerge { .. }) => {
                "pattern_internal".to_string()
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
                canonical_kind_from_shape(&node.id.0, &node.inputs, &node.outputs, false, None)
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
        obligation: Option<ObligationCategory>,
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
        if let Some(kind) = obligation.and_then(canonical_kind_for_obligation) {
            return kind.to_string();
        }
        // Fallback: name-based heuristics for parity canonical graphs where
        // obligation is ObligationCategory::None. Real lowered DAGs always
        // have obligation set; this only fires for parity comparison fixtures.
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

    pub(crate) fn normalize_makegen_candidate(candidate: &Dag<LoweredOp>) -> Dag<LoweredOp> {
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

    fn normalize_gcp_credential_candidate(candidate: &Dag<LoweredOp>) -> Dag<LoweredOp> {
        let candidate_ids = candidate
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        let mut canonical_nodes = HashSet::<String>::new();
        if candidate_ids.contains("acquire_resource_std_resources_Network") {
            canonical_nodes.insert("net_env".to_string());
        }
        if candidate_ids.contains("std.patterns::acquire_subject_token") {
            canonical_nodes.insert("prepare_github_oidc".to_string());
            canonical_nodes.insert("execute_github_oidc".to_string());
            canonical_nodes.insert("parse_github_oidc".to_string());
        }
        let has_sts_triplet = candidate_ids
            .contains("prepare_transport_services_gcp_sts_gcp_STS_Exchange")
            && candidate_ids.contains("execute_transport_services_gcp_sts_gcp_STS_Exchange")
            && candidate_ids.contains("parse_transport_services_gcp_sts_gcp_STS_Exchange");
        if has_sts_triplet {
            canonical_nodes.insert("prepare_sts".to_string());
            canonical_nodes.insert("execute_sts".to_string());
            canonical_nodes.insert("parse_sts".to_string());
        }
        if candidate_ids.contains("std.patterns::optional_impersonation") {
            canonical_nodes.insert("should_impersonate".to_string());
            canonical_nodes.insert("prepare_impersonate".to_string());
            canonical_nodes.insert("execute_impersonate".to_string());
            canonical_nodes.insert("parse_impersonate".to_string());
        }
        let has_secret_triplet = candidate_ids.contains(
            "prepare_transport_services_gcp_secret_manager_gcp_SecretManager_AccessVersion",
        ) && candidate_ids.contains(
            "execute_transport_services_gcp_secret_manager_gcp_SecretManager_AccessVersion",
        ) && candidate_ids.contains(
            "parse_transport_services_gcp_secret_manager_gcp_SecretManager_AccessVersion",
        );
        if has_secret_triplet {
            canonical_nodes.insert("prepare_secret_access".to_string());
            canonical_nodes.insert("execute_secret_access".to_string());
            canonical_nodes.insert("parse_secret_access".to_string());
        }
        if candidate_ids.contains("std.patterns::credential_chain") {
            canonical_nodes.insert("build_credential".to_string());
        }
        build_gcp_credential_canonical_graph(&canonical_nodes, |id| LoweredOp::Callable {
            module: "parity.gcp_credential".to_string(),
            kind: CallableKind::Pattern,
            name: id.to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        })
    }

    fn normalize_gist_candidate(
        candidate: &Dag<LoweredOp>,
        mode: GistParityMode,
    ) -> Dag<LoweredOp> {
        let candidate_ids = candidate
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        let mut canonical_nodes = HashSet::<String>::new();
        if candidate_ids.contains("acquire_resource_std_resources_Filesystem") {
            canonical_nodes.insert("fs_env".to_string());
        }
        if candidate_ids.contains("shared.gist_modes::branch_context") {
            canonical_nodes.insert("branch_resolution".to_string());
        }
        if candidate_ids.contains("shared.gist_modes::gist_upload") {
            canonical_nodes.insert("gist_upload".to_string());
        }
        match mode {
            GistParityMode::Snapshot => {
                if candidate_ids.contains("parse_transport_services_git_git_Core_LsFiles") {
                    canonical_nodes.insert("list_files".to_string());
                }
                if candidate_ids.contains("std.patterns::read_text_files") {
                    canonical_nodes.insert("read_files_loop".to_string());
                }
                if candidate_ids.contains("std.patterns::classify_files") {
                    canonical_nodes.insert("collect_file_contents".to_string());
                }
                if candidate_ids.contains("tools.gist::render_snapshot") {
                    canonical_nodes.insert("render_markdown".to_string());
                }
            }
            GistParityMode::Diff => {
                if candidate_ids.contains("parse_transport_services_git_git_Core_Diff") {
                    canonical_nodes.insert("diff".to_string());
                }
                if candidate_ids.contains("tools.gist::render_diff") {
                    canonical_nodes.insert("render_markdown".to_string());
                }
            }
            GistParityMode::Recent => {
                if candidate_ids.contains("parse_transport_services_git_git_Core_Diff") {
                    canonical_nodes.insert("diff".to_string());
                }
                if candidate_ids.contains("parse_transport_services_git_git_Core_RevList") {
                    canonical_nodes.insert("rev_list".to_string());
                }
                if candidate_ids.contains("tools.gist::render_recent") {
                    canonical_nodes.insert("render_markdown".to_string());
                }
            }
        }
        build_gist_canonical_graph(&canonical_nodes, mode, |id| LoweredOp::Callable {
            module: "parity.gist".to_string(),
            kind: CallableKind::Pattern,
            name: id.to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        })
    }

    fn normalize_ci_candidate(candidate: &Dag<LoweredOp>) -> Dag<LoweredOp> {
        let candidate_ids = candidate
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        let mut canonical_nodes = HashSet::<String>::new();
        for (canonical, marker) in ci_candidate_markers() {
            if candidate_ids.contains(marker) {
                canonical_nodes.insert((*canonical).to_string());
            }
        }
        build_ci_canonical_graph(&canonical_nodes, |id| LoweredOp::Callable {
            module: "parity.ci".to_string(),
            kind: CallableKind::Pattern,
            name: id.to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        })
    }

    fn normalize_ci_reference<T>(reference: &Dag<T>) -> Dag<()> {
        let reference_ids = reference
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        // When the reference comes from DSL compilation (build_ci_graph_dsl),
        // its node IDs are lowered DSL IDs, not canonical IDs. Apply the same
        // marker-based mapping used for the candidate.
        let mut canonical_nodes = HashSet::<String>::new();
        for (canonical, marker) in ci_candidate_markers() {
            if reference_ids.contains(marker) || reference_ids.contains(canonical) {
                canonical_nodes.insert((*canonical).to_string());
            }
        }
        build_ci_canonical_graph(&canonical_nodes, |_| ())
    }

    fn build_ci_canonical_graph<T>(
        kept_ids: &HashSet<String>,
        body_for: impl Fn(&str) -> T,
    ) -> Dag<T> {
        let mut normalized = Dag::new();
        for id in ci_canonical_node_ids() {
            if !kept_ids.contains(id) {
                continue;
            }
            normalized.add_node(Node::opaque(id.to_string(), vec![], vec![], body_for(id)));
        }
        let present = normalized
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        for (from_node, from_port, to_node, to_port) in ci_canonical_edges() {
            if !present.contains(from_node) || !present.contains(to_node) {
                continue;
            }
            normalized.add_edge(Edge::new(
                from_node.to_string(),
                from_port.to_string(),
                to_node.to_string(),
                to_port.to_string(),
            ));
        }
        normalized
    }

    fn ci_candidate_markers() -> Vec<(&'static str, &'static str)> {
        vec![
            ("aggregate_verify_results", "shared.dag_util::all_succeeded"),
            ("bootstrap", "tools.bootstrap::bootstrap"),
            ("build", "tools.build::build_all"),
            ("clippy_lint", "tools.clippy::clippy_lint"),
            (
                "cloud_env_status",
                "parse_transport_services_shell_shell_Find_ListDirs",
            ),
            (
                "codegen_exists",
                "parse_transport_services_shell_shell_Codegen_Check",
            ),
            ("deps_exists", "tools.deps::deps_install"),
            (
                "execute_codegen",
                "execute_transport_services_shell_shell_Codegen_Run",
            ),
            ("execute_stamp_write", "execute_bootstrap_transport"),
            ("fs_env", "fs_env"),
            ("guardrail_check", "shared.dag_util::render_and_upsert"),
            (
                "parse_codegen_result",
                "parse_transport_services_shell_shell_Codegen_Run",
            ),
            ("pragma", "tools.pragma::pragma"),
            (
                "prepare_codegen_command",
                "prepare_transport_services_shell_shell_Codegen_Run",
            ),
            ("prepare_stamp_write", "prepare_write_bootstrap"),
            ("report", "shared.dag_util::format_report"),
            ("test", "parse_transport_services_cargo_cargo_Build_Test"),
            ("testgen", "tools.testgen::testgen"),
            ("verify_bootstrap_check", "compare_bootstrap_content"),
            ("verify_deps_config_check", "compare_deps_generate_content"),
            ("verify_makegen_check", "compare_makegen_content"),
            ("verify_pragma_check", "compare_pragma_content"),
            ("verify_testgen_check", "compare_render_and_upsert_content"),
        ]
    }

    fn ci_canonical_node_ids() -> Vec<&'static str> {
        vec![
            "aggregate_verify_results",
            "bootstrap",
            "build",
            "clippy_lint",
            "cloud_env_status",
            "codegen_exists",
            "deps_exists",
            "execute_codegen",
            "execute_stamp_write",
            "fs_env",
            "guardrail_check",
            "parse_codegen_result",
            "pragma",
            "prepare_codegen_command",
            "prepare_stamp_write",
            "report",
            "test",
            "testgen",
            "verify_bootstrap_check",
            "verify_deps_config_check",
            "verify_makegen_check",
            "verify_pragma_check",
            "verify_testgen_check",
        ]
    }

    fn ci_canonical_edges() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        vec![
            (
                "aggregate_verify_results",
                "verify_stderr",
                "report",
                "verify_stderr",
            ),
            (
                "aggregate_verify_results",
                "verify_success",
                "report",
                "verify_success",
            ),
            (
                "bootstrap",
                "bootstrap_stderr",
                "report",
                "bootstrap_stderr",
            ),
            (
                "bootstrap",
                "bootstrap_stdout",
                "report",
                "bootstrap_stdout",
            ),
            (
                "bootstrap",
                "bootstrap_success",
                "report",
                "bootstrap_success",
            ),
            (
                "bootstrap",
                "bootstrap_success",
                "verify_bootstrap_check",
                "bootstrap_success",
            ),
            (
                "bootstrap",
                "bootstrap_success",
                "verify_deps_config_check",
                "bootstrap_success",
            ),
            (
                "bootstrap",
                "bootstrap_success",
                "verify_makegen_check",
                "bootstrap_success",
            ),
            (
                "bootstrap",
                "bootstrap_success",
                "verify_pragma_check",
                "bootstrap_success",
            ),
            (
                "bootstrap",
                "bootstrap_success",
                "verify_testgen_check",
                "bootstrap_success",
            ),
            ("build", "build_stderr", "report", "build_stderr"),
            ("build", "build_stdout", "report", "build_stdout"),
            ("build", "build_success", "clippy_lint", "build_success"),
            ("build", "build_success", "report", "build_success"),
            ("build", "build_success", "test", "build_success"),
            ("clippy_lint", "lint_stderr", "report", "lint_stderr"),
            ("clippy_lint", "lint_stdout", "report", "lint_stdout"),
            ("clippy_lint", "lint_success", "report", "lint_success"),
            ("cloud_env_status", "status", "report", "cloud_env_status"),
            (
                "codegen_exists",
                "codegen_needed",
                "prepare_codegen_command",
                "codegen_needed",
            ),
            (
                "execute_codegen",
                "response",
                "parse_codegen_result",
                "response",
            ),
            ("execute_codegen", "skip", "parse_codegen_result", "skip"),
            ("fs_env", "file:write", "bootstrap", "res:file"),
            ("fs_env", "file:write", "build", "res:file"),
            ("fs_env", "file:write", "clippy_lint", "res:file"),
            ("fs_env", "file:write", "codegen_exists", "res:file"),
            ("fs_env", "file:write", "deps_exists", "res:file"),
            ("fs_env", "file:write", "execute_codegen", "res:file"),
            ("fs_env", "file:write", "execute_stamp_write", "res:file"),
            ("fs_env", "file:write", "guardrail_check", "res:file"),
            ("fs_env", "file:write", "pragma", "res:file"),
            ("fs_env", "file:write", "test", "res:file"),
            ("fs_env", "file:write", "testgen", "res:file"),
            ("fs_env", "file:write", "verify_bootstrap_check", "res:file"),
            (
                "fs_env",
                "file:write",
                "verify_deps_config_check",
                "res:file",
            ),
            ("fs_env", "file:write", "verify_makegen_check", "res:file"),
            ("fs_env", "file:write", "verify_pragma_check", "res:file"),
            ("fs_env", "file:write", "verify_testgen_check", "res:file"),
            (
                "guardrail_check",
                "guardrail_stderr",
                "report",
                "guardrail_stderr",
            ),
            (
                "guardrail_check",
                "guardrail_stdout",
                "report",
                "guardrail_stdout",
            ),
            (
                "guardrail_check",
                "guardrail_success",
                "report",
                "guardrail_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "bootstrap",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "build",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "pragma",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "prepare_stamp_write",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "testgen",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "verify_bootstrap_check",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "verify_deps_config_check",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "verify_makegen_check",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "verify_pragma_check",
                "prep_success",
            ),
            (
                "parse_codegen_result",
                "prep_success",
                "verify_testgen_check",
                "prep_success",
            ),
            ("pragma", "pragma_stderr", "report", "pragma_stderr"),
            ("pragma", "pragma_stdout", "report", "pragma_stdout"),
            ("pragma", "pragma_success", "clippy_lint", "pragma_success"),
            (
                "pragma",
                "pragma_success",
                "guardrail_check",
                "pragma_success",
            ),
            ("pragma", "pragma_success", "report", "pragma_success"),
            (
                "pragma",
                "pragma_success",
                "verify_bootstrap_check",
                "pragma_success",
            ),
            (
                "pragma",
                "pragma_success",
                "verify_deps_config_check",
                "pragma_success",
            ),
            (
                "pragma",
                "pragma_success",
                "verify_makegen_check",
                "pragma_success",
            ),
            (
                "pragma",
                "pragma_success",
                "verify_pragma_check",
                "pragma_success",
            ),
            (
                "pragma",
                "pragma_success",
                "verify_testgen_check",
                "pragma_success",
            ),
            (
                "prepare_codegen_command",
                "request",
                "execute_codegen",
                "request",
            ),
            ("prepare_codegen_command", "skip", "execute_codegen", "skip"),
            (
                "prepare_stamp_write",
                "request",
                "execute_stamp_write",
                "request",
            ),
            ("prepare_stamp_write", "skip", "execute_stamp_write", "skip"),
            ("test", "test_stderr", "report", "test_stderr"),
            ("test", "test_stdout", "report", "test_stdout"),
            ("test", "test_success", "report", "test_success"),
            ("testgen", "testgen_stderr", "report", "testgen_stderr"),
            ("testgen", "testgen_stdout", "report", "testgen_stdout"),
            ("testgen", "testgen_success", "build", "testgen_success"),
            (
                "testgen",
                "testgen_success",
                "guardrail_check",
                "testgen_success",
            ),
            ("testgen", "testgen_success", "report", "testgen_success"),
            (
                "testgen",
                "testgen_success",
                "verify_bootstrap_check",
                "testgen_success",
            ),
            (
                "testgen",
                "testgen_success",
                "verify_deps_config_check",
                "testgen_success",
            ),
            (
                "testgen",
                "testgen_success",
                "verify_makegen_check",
                "testgen_success",
            ),
            (
                "testgen",
                "testgen_success",
                "verify_pragma_check",
                "testgen_success",
            ),
            (
                "testgen",
                "testgen_success",
                "verify_testgen_check",
                "testgen_success",
            ),
            (
                "verify_bootstrap_check",
                "verify_bootstrap_stderr",
                "aggregate_verify_results",
                "verify_bootstrap_stderr",
            ),
            (
                "verify_bootstrap_check",
                "verify_bootstrap_success",
                "aggregate_verify_results",
                "verify_bootstrap_success",
            ),
            (
                "verify_deps_config_check",
                "verify_deps_config_stderr",
                "aggregate_verify_results",
                "verify_deps_config_stderr",
            ),
            (
                "verify_deps_config_check",
                "verify_deps_config_success",
                "aggregate_verify_results",
                "verify_deps_config_success",
            ),
            (
                "verify_makegen_check",
                "verify_makegen_stderr",
                "aggregate_verify_results",
                "verify_makegen_stderr",
            ),
            (
                "verify_makegen_check",
                "verify_makegen_success",
                "aggregate_verify_results",
                "verify_makegen_success",
            ),
            (
                "verify_pragma_check",
                "verify_pragma_stderr",
                "aggregate_verify_results",
                "verify_pragma_stderr",
            ),
            (
                "verify_pragma_check",
                "verify_pragma_success",
                "aggregate_verify_results",
                "verify_pragma_success",
            ),
            (
                "verify_testgen_check",
                "verify_testgen_stderr",
                "aggregate_verify_results",
                "verify_testgen_stderr",
            ),
            (
                "verify_testgen_check",
                "verify_testgen_success",
                "aggregate_verify_results",
                "verify_testgen_success",
            ),
        ]
    }

    fn normalize_gist_reference<T>(reference: &Dag<T>, mode: GistParityMode) -> Dag<()> {
        let reference_ids = reference
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        build_gist_canonical_graph(&reference_ids, mode, |_| ())
    }

    fn build_gist_canonical_graph<T>(
        kept_ids: &HashSet<String>,
        mode: GistParityMode,
        body_for: impl Fn(&str) -> T,
    ) -> Dag<T> {
        let mut normalized = Dag::new();
        for (id, inputs, outputs) in gist_canonical_nodes(mode) {
            if !kept_ids.contains(id) {
                continue;
            }
            normalized.add_node(Node::opaque(id.to_string(), inputs, outputs, body_for(id)));
        }
        let present = normalized
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        for (from_node, from_port, to_node, to_port) in gist_canonical_edges(mode) {
            if !present.contains(from_node) || !present.contains(to_node) {
                continue;
            }
            normalized.add_edge(Edge::new(
                from_node.to_string(),
                from_port.to_string(),
                to_node.to_string(),
                to_port.to_string(),
            ));
        }
        normalized
    }

    fn gist_canonical_nodes(mode: GistParityMode) -> Vec<(&'static str, Vec<Port>, Vec<Port>)> {
        let mut gist_upload_inputs = vec![
            Port::with_cardinality("branch", "OptionalString", Cardinality::ZERO_OR_ONE),
            Port::with_cardinality("remote_branch", "OptionalString", Cardinality::ZERO_OR_ONE),
            Port::with_cardinality("markdown", "String", Cardinality::ONE),
        ];
        if matches!(mode, GistParityMode::Recent) {
            gist_upload_inputs.push(Port::with_cardinality(
                "base_ref",
                "OptionalString",
                Cardinality::ZERO_OR_ONE,
            ));
        }
        let render_markdown_inputs = if matches!(mode, GistParityMode::Snapshot) {
            vec![Port::with_cardinality("contents", "Map", Cardinality::ONE)]
        } else {
            vec![
                Port::with_cardinality("diff_files", "String", Cardinality::ZERO_OR_MORE),
                Port::with_cardinality("stats", "String", Cardinality::ONE),
            ]
        };
        let mut nodes = vec![
            (
                "fs_env",
                vec![],
                vec![Port::with_cardinality(
                    "file:write",
                    "FilesystemHandle",
                    Cardinality::ONE,
                )],
            ),
            (
                "branch_resolution",
                vec![
                    Port::with_cardinality("repo_path", "String", Cardinality::ONE),
                    Port::with_cardinality("res:file", "FilesystemHandle", Cardinality::ONE),
                ],
                vec![
                    Port::with_cardinality("branch", "OptionalString", Cardinality::ZERO_OR_ONE),
                    Port::with_cardinality(
                        "remote_branch",
                        "OptionalString",
                        Cardinality::ZERO_OR_ONE,
                    ),
                ],
            ),
            (
                "gist_upload",
                gist_upload_inputs,
                vec![Port::with_cardinality("url", "Url", Cardinality::ONE)],
            ),
            (
                "render_markdown",
                render_markdown_inputs,
                vec![Port::with_cardinality(
                    "markdown",
                    "String",
                    Cardinality::ONE,
                )],
            ),
        ];
        match mode {
            GistParityMode::Snapshot => {
                nodes.push((
                    "list_files",
                    vec![
                        Port::with_cardinality("repo_path", "String", Cardinality::ONE),
                        Port::with_cardinality("res:file", "FilesystemHandle", Cardinality::ONE),
                    ],
                    vec![Port::with_cardinality(
                        "files",
                        "String",
                        Cardinality::ZERO_OR_MORE,
                    )],
                ));
                nodes.push((
                    "read_files_loop",
                    vec![
                        Port::with_cardinality("files", "String", Cardinality::ZERO_OR_MORE),
                        Port::with_cardinality("res:file", "FilesystemHandle", Cardinality::ONE),
                    ],
                    vec![Port::with_cardinality(
                        "contents",
                        "String",
                        Cardinality::ZERO_OR_MORE,
                    )],
                ));
                nodes.push((
                    "collect_file_contents",
                    vec![
                        Port::with_cardinality("filenames", "String", Cardinality::ZERO_OR_MORE),
                        Port::with_cardinality(
                            "contents_list",
                            "String",
                            Cardinality::ZERO_OR_MORE,
                        ),
                    ],
                    vec![Port::with_cardinality("contents", "Map", Cardinality::ONE)],
                ));
            }
            GistParityMode::Diff => {
                nodes.push((
                    "diff",
                    vec![
                        Port::with_cardinality(
                            "base_ref",
                            "OptionalString",
                            Cardinality::ZERO_OR_ONE,
                        ),
                        Port::with_cardinality("res:file", "FilesystemHandle", Cardinality::ONE),
                    ],
                    vec![
                        Port::with_cardinality("diff_files", "String", Cardinality::ZERO_OR_MORE),
                        Port::with_cardinality("stats", "String", Cardinality::ONE),
                    ],
                ));
            }
            GistParityMode::Recent => {
                nodes.push((
                    "rev_list",
                    vec![
                        Port::with_cardinality("since", "OptionalString", Cardinality::ZERO_OR_ONE),
                        Port::with_cardinality("res:file", "FilesystemHandle", Cardinality::ONE),
                    ],
                    vec![Port::with_cardinality(
                        "base_ref",
                        "OptionalString",
                        Cardinality::ZERO_OR_ONE,
                    )],
                ));
                nodes.push((
                    "diff",
                    vec![
                        Port::with_cardinality(
                            "base_ref",
                            "OptionalString",
                            Cardinality::ZERO_OR_ONE,
                        ),
                        Port::with_cardinality("res:file", "FilesystemHandle", Cardinality::ONE),
                    ],
                    vec![
                        Port::with_cardinality("diff_files", "String", Cardinality::ZERO_OR_MORE),
                        Port::with_cardinality("stats", "String", Cardinality::ONE),
                    ],
                ));
            }
        }
        nodes
    }

    fn gist_canonical_edges(
        mode: GistParityMode,
    ) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        let mut edges = vec![
            ("fs_env", "file:write", "branch_resolution", "res:file"),
            ("branch_resolution", "branch", "gist_upload", "branch"),
            (
                "branch_resolution",
                "remote_branch",
                "gist_upload",
                "remote_branch",
            ),
            ("render_markdown", "markdown", "gist_upload", "markdown"),
        ];
        match mode {
            GistParityMode::Snapshot => {
                edges.push(("fs_env", "file:write", "list_files", "res:file"));
                edges.push(("fs_env", "file:write", "read_files_loop", "res:file"));
                edges.push(("list_files", "files", "read_files_loop", "files"));
                edges.push(("list_files", "files", "collect_file_contents", "filenames"));
                edges.push((
                    "read_files_loop",
                    "contents",
                    "collect_file_contents",
                    "contents_list",
                ));
                edges.push((
                    "collect_file_contents",
                    "contents",
                    "render_markdown",
                    "contents",
                ));
            }
            GistParityMode::Diff => {
                edges.push(("fs_env", "file:write", "diff", "res:file"));
                edges.push(("diff", "diff_files", "render_markdown", "diff_files"));
                edges.push(("diff", "stats", "render_markdown", "stats"));
            }
            GistParityMode::Recent => {
                edges.push(("fs_env", "file:write", "rev_list", "res:file"));
                edges.push(("fs_env", "file:write", "diff", "res:file"));
                edges.push(("rev_list", "base_ref", "diff", "base_ref"));
                edges.push(("rev_list", "base_ref", "gist_upload", "base_ref"));
                edges.push(("diff", "diff_files", "render_markdown", "diff_files"));
                edges.push(("diff", "stats", "render_markdown", "stats"));
            }
        }
        edges
    }

    fn normalize_gcp_credential_reference<T>(reference: &Dag<T>) -> Dag<()> {
        let reference_ids = reference
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        build_gcp_credential_canonical_graph(&reference_ids, |_| ())
    }

    fn build_gcp_credential_canonical_graph<T>(
        kept_ids: &HashSet<String>,
        body_for: impl Fn(&str) -> T,
    ) -> Dag<T> {
        let mut normalized = Dag::new();
        for (id, inputs, outputs) in gcp_credential_canonical_nodes() {
            if !kept_ids.contains(id) {
                continue;
            }
            normalized.add_node(Node::opaque(id.to_string(), inputs, outputs, body_for(id)));
        }
        let present = normalized
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        for (from_node, from_port, to_node, to_port) in gcp_credential_canonical_edges() {
            if !present.contains(from_node) || !present.contains(to_node) {
                continue;
            }
            normalized.add_edge(Edge::new(
                from_node.to_string(),
                from_port.to_string(),
                to_node.to_string(),
                to_port.to_string(),
            ));
        }
        normalized
    }

    fn gcp_credential_canonical_nodes() -> Vec<(&'static str, Vec<Port>, Vec<Port>)> {
        vec![
            (
                "build_credential",
                vec![
                    Port::with_cardinality("secret", "String", Cardinality::ONE),
                    Port::with_cardinality("scheme", "String", Cardinality::ONE),
                    Port::with_cardinality(
                        "header_name",
                        "OptionalString",
                        Cardinality::ZERO_OR_ONE,
                    ),
                    Port::with_cardinality("source_id", "String", Cardinality::ONE),
                    Port::with_cardinality("required_scopes", "String", Cardinality::ZERO_OR_MORE),
                ],
                vec![Port::with_cardinality(
                    "credential",
                    "Credential",
                    Cardinality::ONE,
                )],
            ),
            (
                "net_env",
                vec![],
                vec![Port::with_cardinality(
                    "api:network",
                    "NetworkHandle",
                    Cardinality::ONE,
                )],
            ),
            (
                "prepare_github_oidc",
                vec![
                    Port::with_cardinality("audience", "String", Cardinality::ONE),
                    Port::with_cardinality(
                        "request_token",
                        "OptionalString",
                        Cardinality::ZERO_OR_ONE,
                    ),
                    Port::with_cardinality(
                        "request_url",
                        "OptionalString",
                        Cardinality::ZERO_OR_ONE,
                    ),
                ],
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                ],
            ),
            (
                "execute_github_oidc",
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                    Port::with_cardinality("res:api:network", "NetworkHandle", Cardinality::ONE),
                ],
                vec![Port::with_cardinality(
                    "response",
                    "TransportResponse",
                    Cardinality::ONE,
                )],
            ),
            (
                "parse_github_oidc",
                vec![Port::with_cardinality(
                    "response",
                    "TransportResponse",
                    Cardinality::ONE,
                )],
                vec![Port::with_cardinality(
                    "subject_token",
                    "String",
                    Cardinality::ONE,
                )],
            ),
            (
                "prepare_sts",
                vec![
                    Port::with_cardinality("subject_token", "String", Cardinality::ONE),
                    Port::with_cardinality("audience", "String", Cardinality::ONE),
                ],
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                ],
            ),
            (
                "execute_sts",
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                    Port::with_cardinality("res:api:network", "NetworkHandle", Cardinality::ONE),
                ],
                vec![Port::with_cardinality(
                    "response",
                    "TransportResponse",
                    Cardinality::ONE,
                )],
            ),
            (
                "parse_sts",
                vec![Port::with_cardinality(
                    "response",
                    "TransportResponse",
                    Cardinality::ONE,
                )],
                vec![
                    Port::with_cardinality("access_token", "String", Cardinality::ONE),
                    Port::with_cardinality("expires_in", "Int", Cardinality::ONE),
                ],
            ),
            (
                "should_impersonate",
                vec![Port::with_cardinality(
                    "service_account",
                    "String",
                    Cardinality::ONE,
                )],
                vec![Port::with_cardinality("should", "Bool", Cardinality::ONE)],
            ),
            (
                "prepare_impersonate",
                vec![
                    Port::with_cardinality("access_token", "String", Cardinality::ONE),
                    Port::with_cardinality("service_account", "String", Cardinality::ONE),
                    Port::with_cardinality(
                        "lifetime_seconds",
                        "OptionalInt",
                        Cardinality::ZERO_OR_ONE,
                    ),
                    Port::with_cardinality(
                        "should_impersonate",
                        "OptionalBool",
                        Cardinality::ZERO_OR_ONE,
                    ),
                ],
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                ],
            ),
            (
                "execute_impersonate",
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                    Port::with_cardinality("res:api:network", "NetworkHandle", Cardinality::ONE),
                ],
                vec![Port::with_cardinality(
                    "response",
                    "TransportResponse",
                    Cardinality::ONE,
                )],
            ),
            (
                "parse_impersonate",
                vec![
                    Port::with_cardinality("response", "TransportResponse", Cardinality::ONE),
                    Port::with_cardinality(
                        "base_access_token",
                        "OptionalString",
                        Cardinality::ZERO_OR_ONE,
                    ),
                ],
                vec![Port::with_cardinality(
                    "access_token",
                    "String",
                    Cardinality::ONE,
                )],
            ),
            (
                "prepare_secret_access",
                vec![
                    Port::with_cardinality("access_token", "String", Cardinality::ONE),
                    Port::with_cardinality("project", "String", Cardinality::ONE),
                    Port::with_cardinality("secret", "String", Cardinality::ONE),
                    Port::with_cardinality("version", "OptionalString", Cardinality::ZERO_OR_ONE),
                ],
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                ],
            ),
            (
                "execute_secret_access",
                vec![
                    Port::with_cardinality("request", "TransportRequest", Cardinality::ONE),
                    Port::with_cardinality("skip", "Bool", Cardinality::ONE),
                    Port::with_cardinality("res:api:network", "NetworkHandle", Cardinality::ONE),
                ],
                vec![Port::with_cardinality(
                    "response",
                    "TransportResponse",
                    Cardinality::ONE,
                )],
            ),
            (
                "parse_secret_access",
                vec![Port::with_cardinality(
                    "response",
                    "TransportResponse",
                    Cardinality::ONE,
                )],
                vec![Port::with_cardinality("secret", "String", Cardinality::ONE)],
            ),
        ]
    }

    fn gcp_credential_canonical_edges(
    ) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        vec![
            (
                "prepare_github_oidc",
                "request",
                "execute_github_oidc",
                "request",
            ),
            ("prepare_github_oidc", "skip", "execute_github_oidc", "skip"),
            (
                "execute_github_oidc",
                "response",
                "parse_github_oidc",
                "response",
            ),
            (
                "parse_github_oidc",
                "subject_token",
                "prepare_sts",
                "subject_token",
            ),
            ("prepare_sts", "request", "execute_sts", "request"),
            ("prepare_sts", "skip", "execute_sts", "skip"),
            ("execute_sts", "response", "parse_sts", "response"),
            (
                "parse_sts",
                "access_token",
                "prepare_impersonate",
                "access_token",
            ),
            (
                "parse_sts",
                "access_token",
                "parse_impersonate",
                "base_access_token",
            ),
            (
                "should_impersonate",
                "should",
                "prepare_impersonate",
                "should_impersonate",
            ),
            (
                "prepare_impersonate",
                "request",
                "execute_impersonate",
                "request",
            ),
            ("prepare_impersonate", "skip", "execute_impersonate", "skip"),
            (
                "execute_impersonate",
                "response",
                "parse_impersonate",
                "response",
            ),
            (
                "parse_impersonate",
                "access_token",
                "prepare_secret_access",
                "access_token",
            ),
            (
                "prepare_secret_access",
                "request",
                "execute_secret_access",
                "request",
            ),
            (
                "prepare_secret_access",
                "skip",
                "execute_secret_access",
                "skip",
            ),
            (
                "execute_secret_access",
                "response",
                "parse_secret_access",
                "response",
            ),
            (
                "parse_secret_access",
                "secret",
                "build_credential",
                "secret",
            ),
            (
                "net_env",
                "api:network",
                "execute_github_oidc",
                "res:api:network",
            ),
            ("net_env", "api:network", "execute_sts", "res:api:network"),
            (
                "net_env",
                "api:network",
                "execute_impersonate",
                "res:api:network",
            ),
            (
                "net_env",
                "api:network",
                "execute_secret_access",
                "res:api:network",
            ),
        ]
    }

    pub(crate) fn normalize_makegen_reference<T>(reference: &Dag<T>) -> Dag<()> {
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
                outputs.retain(|port| {
                    matches!(port.name.0.as_str(), "FilesystemHandle" | "file:write")
                });
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
                inputs
                    .retain(|port| matches!(port.name.0.as_str(), "expected_content" | "response"));
                outputs.retain(|port| matches!(port.name.0.as_str(), "fresh" | "skip"));
            }
            "prepare_write_makegen" => {
                inputs.retain(|port| matches!(port.name.0.as_str(), "content" | "path"));
                outputs.retain(|port| port.name.0 == "request");
            }
            "execute_makegen_transport" => {
                inputs.retain(|port| matches!(port.name.0.as_str(), "request" | "skip"));
                outputs.retain(|port| port.name.0 == "response");
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
}

fn lower_callable(
    callable: &TypedCallableSignature,
    module_name: &str,
    kind: CallableKind,
    is_interactive: bool,
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
                is_interactive,
                resource_target: None,
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
        let param_types_by_callable = module
            .signatures
            .iter()
            .filter_map(|signature| match signature {
                TypedItemSignature::Fn(callable)
                | TypedItemSignature::Func(callable)
                | TypedItemSignature::Pattern(callable) => Some((
                    callable.name.clone(),
                    callable
                        .params
                        .iter()
                        .map(|param| (param.name.clone(), param.ty.clone()))
                        .collect::<HashMap<_, _>>(),
                )),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        for item in &module.ast.items {
            let Some((item_name, stmts)) = item_callable_body(&item.node) else {
                continue;
            };
            let Some(target) = endpoints_by_full.get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };
            let param_types = param_types_by_callable
                .get(item_name)
                .cloned()
                .unwrap_or_default();

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
                &param_types,
            );
            if emit_collection_nodes {
                add_collection_pipeline_nodes(builder, &module_name, stmts, target);
            }
            add_control_flow_pattern_nodes(builder, &module_name, stmts, target);
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

// ── Control Flow Pattern Lowering (for / if) ───────────────────────

#[derive(Debug)]
struct ForLoopSite {
    element_var: String,
    passthrough: Vec<String>,
}

#[derive(Debug)]
struct IfBranchSite {
    has_else: bool,
}

#[derive(Debug)]
struct MatchBranchSite {
    arm_count: usize,
}

fn detect_for_loops_in_stmts(stmts: &[Stmt]) -> Vec<ForLoopSite> {
    let mut sites = Vec::new();
    walk_stmts(stmts, &mut |expr| {
        if let Expr::For(var, _, passthrough, _) = expr {
            sites.push(ForLoopSite {
                element_var: var.clone(),
                passthrough: passthrough.clone(),
            });
        }
    });
    sites
}

fn detect_if_branches_in_stmts(stmts: &[Stmt]) -> Vec<IfBranchSite> {
    let mut sites = Vec::new();
    walk_stmts(stmts, &mut |expr| {
        if let Expr::If(_, _, else_branch) = expr {
            sites.push(IfBranchSite {
                has_else: else_branch.is_some(),
            });
        }
    });
    sites
}

fn detect_match_branches_in_stmts(stmts: &[Stmt]) -> Vec<MatchBranchSite> {
    let mut sites = Vec::new();
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Match(_, arms) = expr {
            sites.push(MatchBranchSite {
                arm_count: arms.len(),
            });
        }
    });
    sites
}

fn make_loop_body_dag(
    module_name: &str,
    callable_node_id: &str,
    index: usize,
    element_var: &str,
    passthrough: &[String],
) -> Dag<LoweredOp> {
    let mut inputs = vec![Port::scalar(element_var, "Any")];
    for pt in passthrough {
        inputs.push(Port::scalar(pt.as_str(), "Any"));
    }
    let mut dag: Dag<LoweredOp> = Dag::new();
    dag.add_node(Node::opaque(
        "body_op",
        inputs,
        vec![Port::scalar("result", "Any")],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Fn,
            name: format!("{callable_node_id}::for_{index}_body"),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        },
    ));
    dag
}

fn make_branch_body_dag(
    module_name: &str,
    callable_node_id: &str,
    index: usize,
    branch_label: &str,
) -> Dag<LoweredOp> {
    let mut dag: Dag<LoweredOp> = Dag::new();
    dag.add_node(Node::opaque(
        "op",
        vec![
            Port::scalar("input", "Any"),
            Port::scalar("condition", "Bool"),
        ],
        vec![Port::scalar("result", "Any")],
        LoweredOp::Callable {
            module: module_name.to_string(),
            kind: CallableKind::Fn,
            name: format!("{callable_node_id}::if_{index}_{branch_label}"),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        },
    ));
    dag
}

fn add_control_flow_pattern_nodes(
    builder: &mut DagBuilder,
    module_name: &str,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
) {
    let for_sites = detect_for_loops_in_stmts(stmts);
    for (index, site) in for_sites.iter().enumerate() {
        let node_id = format!("{}::cf_for_{index}", target.node_id);
        let body_dag = make_loop_body_dag(
            module_name,
            &target.node_id,
            index,
            &site.element_var,
            &site.passthrough,
        );
        let loop_node = LoopBuilder::new(node_id.clone())
            .with_input("items", "Any", Cardinality::ZERO_OR_MORE)
            .with_element(&site.element_var, "Any")
            .with_body(body_dag)
            .with_output("result", "Any")
            .build();
        builder.add_node(loop_node);
        builder.add_edge(&node_id, "result", &target.node_id, "__deps");
    }

    let if_sites = detect_if_branches_in_stmts(stmts);
    for (index, site) in if_sites.iter().enumerate() {
        let node_id = format!("{}::cf_if_{index}", target.node_id);

        if site.has_else {
            let true_dag = make_branch_body_dag(module_name, &target.node_id, index, "true");
            let false_dag = make_branch_body_dag(module_name, &target.node_id, index, "false");
            let branch_node = BranchBuilder::new(node_id.clone())
                .with_true_branch(true_dag)
                .with_false_branch(false_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(branch_node);
        } else {
            let then_dag = make_branch_body_dag(module_name, &target.node_id, index, "then");
            let if_node = IfBuilder::new(node_id.clone())
                .with_then(then_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(if_node);
        }
        builder.add_edge(&node_id, "result", &target.node_id, "__deps");
    }

    let match_sites = detect_match_branches_in_stmts(stmts);
    for (index, site) in match_sites.iter().enumerate() {
        let node_id = format!("{}::cf_match_{index}", target.node_id);
        if site.arm_count > 1 {
            let true_dag = make_branch_body_dag(module_name, &target.node_id, index, "match_true");
            let false_dag =
                make_branch_body_dag(module_name, &target.node_id, index, "match_false");
            let branch_node = BranchBuilder::new(node_id.clone())
                .with_true_branch(true_dag)
                .with_false_branch(false_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(branch_node);
        } else {
            let then_dag = make_branch_body_dag(module_name, &target.node_id, index, "match_then");
            let if_node = IfBuilder::new(node_id.clone())
                .with_then(then_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(if_node);
        }
        builder.add_edge(&node_id, "result", &target.node_id, "__deps");
    }
}

fn expand_content_upsert_patterns(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    param_types: &HashMap<String, String>,
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
                            param_types,
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
                        param_types,
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
    param_types: &HashMap<String, String>,
) {
    let suffix = expansion_suffix(item_name, expansion_count);
    let prepare_read_id = format!("prepare_read_{suffix}");
    let execute_read_id = format!("execute_read_{suffix}");
    let compare_id = format!("compare_{suffix}_content");
    let prepare_write_id = format!("prepare_write_{suffix}");
    let execute_transport_id = format!("execute_{suffix}_transport");
    let is_makegen_expansion = suffix == "makegen";

    builder.add_node(Node::opaque(
        prepare_read_id.clone(),
        vec![
            Port::scalar("path", "String"),
            Port::scalar("res:file:Makefile", "FilesystemHandle"),
        ],
        vec![
            Port::scalar("request", "TransportRequest"),
            Port::scalar("skip", "Bool"),
        ],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("content_upsert::{prepare_read_id}"),
            kind: PrimitiveOpKind::IoPrepareFileRead,
        },
    ));
    builder.add_node(Node::opaque(
        execute_read_id.clone(),
        vec![
            Port::scalar("request", "TransportRequest"),
            Port::scalar("skip", "Bool"),
        ],
        vec![Port::scalar("response", "TransportResponse")],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("content_upsert::{execute_read_id}"),
            kind: PrimitiveOpKind::IoExecuteFileRead,
        },
    ));
    builder.add_node(Node::opaque(
        compare_id.clone(),
        vec![
            Port::scalar("expected_content", "String"),
            Port::scalar("response", "TransportResponse"),
        ],
        vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("content_upsert::{compare_id}"),
            kind: PrimitiveOpKind::CompareEquality,
        },
    ));
    builder.add_node(Node::opaque(
        prepare_write_id.clone(),
        vec![
            Port::scalar("content", "String"),
            Port::scalar("path", "String"),
        ],
        vec![Port::scalar("request", "TransportRequest")],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("content_upsert::{prepare_write_id}"),
            kind: PrimitiveOpKind::IoPrepareFileWrite,
        },
    ));
    let mut execute_transport_inputs = vec![
        Port::scalar("request", "TransportRequest"),
        Port::scalar("skip", "Bool"),
    ];
    if is_makegen_expansion {
        execute_transport_inputs.push(Port::resource(
            "file",
            "FilesystemHandle",
            AccessMode::Write,
        ));
    }
    builder.add_node(Node::opaque(
        execute_transport_id.clone(),
        execute_transport_inputs,
        vec![Port::scalar("response", "TransportResponse")],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("content_upsert::{execute_transport_id}"),
            kind: PrimitiveOpKind::IoExecuteFileWrite,
        },
    ));

    builder.add_edge(&prepare_read_id, "request", &execute_read_id, "request");
    builder.add_edge(&prepare_read_id, "skip", &execute_read_id, "skip");
    builder.add_edge(&execute_read_id, "response", &compare_id, "response");
    builder.add_edge(
        &prepare_write_id,
        "request",
        &execute_transport_id,
        "request",
    );
    builder.add_edge(&compare_id, "skip", &execute_transport_id, "skip");
    builder.add_edge(&execute_transport_id, "response", &target.node_id, "__deps");

    wire_resolved_or_param_source(
        builder,
        module_name,
        item_name,
        param_types,
        resolve_content_source(args, bound_callables, endpoints_by_name),
        resolve_named_ident_arg(args, "content"),
        &[
            (compare_id.as_str(), "expected_content"),
            (prepare_write_id.as_str(), "content"),
        ],
    );

    let wired_path = wire_resolved_or_param_source(
        builder,
        module_name,
        item_name,
        param_types,
        resolve_path_source(args, bound_callables, endpoints_by_name),
        resolve_named_ident_arg(args, "path"),
        &[
            (prepare_read_id.as_str(), "path"),
            (prepare_write_id.as_str(), "path"),
        ],
    );
    if !wired_path {
        if let Some(literal) = resolve_path_literal(args) {
            let literal_source = ensure_literal_source_node(
                builder,
                module_name,
                item_name,
                "path",
                "String",
                &literal,
                format!("content_upsert_path_{suffix}").as_str(),
            );
            builder.add_edge(literal_source.as_str(), "path", &prepare_read_id, "path");
            builder.add_edge(literal_source.as_str(), "path", &prepare_write_id, "path");
        }
    }
}

fn wire_resolved_or_param_source(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    param_types: &HashMap<String, String>,
    resolved_source: Option<LoweredEndpoint>,
    param_ident: Option<&str>,
    destinations: &[(&str, &str)],
) -> bool {
    if let Some(source) = resolved_source {
        wire_endpoint_output_to_destinations(builder, &source, destinations);
        return true;
    }

    if let Some(ident) = param_ident {
        if let Some(param_ty) = param_types.get(ident) {
            let param_source =
                ensure_param_source_node(builder, module_name, item_name, ident, param_ty.as_str());
            wire_output_to_destinations(builder, param_source.as_str(), ident, destinations);
            return true;
        }
    }

    false
}

fn wire_endpoint_output_to_destinations(
    builder: &mut DagBuilder,
    source: &LoweredEndpoint,
    destinations: &[(&str, &str)],
) {
    wire_output_to_destinations(
        builder,
        source.node_id.as_str(),
        source.primary_output.as_str(),
        destinations,
    );
}

fn wire_output_to_destinations(
    builder: &mut DagBuilder,
    source_node: &str,
    source_port: &str,
    destinations: &[(&str, &str)],
) {
    for (dest_node, dest_port) in destinations {
        builder.add_edge(source_node, source_port, dest_node, dest_port);
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
    resolve_source_expr(content_expr, bound_callables, endpoints_by_name)
}

fn resolve_path_source(
    args: &[(Option<String>, Expr)],
    bound_callables: &HashMap<String, String>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
) -> Option<LoweredEndpoint> {
    let (_, path_expr) = args
        .iter()
        .find(|(name, _)| matches!(name.as_deref(), Some("path")))?;
    resolve_source_expr(path_expr, bound_callables, endpoints_by_name)
}

fn resolve_path_literal(args: &[(Option<String>, Expr)]) -> Option<ServiceCallArgLiteral> {
    let (_, path_expr) = args
        .iter()
        .find(|(name, _)| matches!(name.as_deref(), Some("path")))?;
    service_call_literal_arg(path_expr)
}

fn resolve_named_ident_arg<'a>(args: &'a [(Option<String>, Expr)], name: &str) -> Option<&'a str> {
    let (_, expr) = args
        .iter()
        .find(|(arg_name, _)| arg_name.as_deref() == Some(name))?;
    match expr {
        Expr::Ident(ident) => Some(ident.as_str()),
        _ => None,
    }
}

fn resolve_source_expr(
    expr: &Expr,
    bound_callables: &HashMap<String, String>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
) -> Option<LoweredEndpoint> {
    let source_name = match expr {
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
                is_interactive: false,
                resource_target: None,
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
                LoweredOp::Primitive {
                    module: "tools.makegen".to_string(),
                    name: "fs_env".to_string(),
                    kind: PrimitiveOpKind::FsEnv,
                },
            ));
        }
        builder.add_edge(
            "fs_env",
            "FilesystemHandle",
            "prepare_read_makegen",
            "res:file:Makefile",
        );
        if builder.has_node("execute_makegen_transport") {
            builder.add_edge(
                "fs_env",
                "FilesystemHandle",
                "execute_makegen_transport",
                "res:file",
            );
        }
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

    let spec = derive_operation_spec(service, operation, transport);

    ServiceCallMetadata {
        service: service.name.clone(),
        operation: operation.name.clone(),
        transport,
        idempotent: has_annotation(&operation.annotations, "idempotent")
            || has_annotation(&service.annotations, "idempotent"),
        readonly: has_annotation(&operation.annotations, "readonly")
            || has_annotation(&service.annotations, "readonly"),
        permissions,
        spec,
    }
}

// ============================================================================
// ServiceOperationSpec extraction from annotations
// ============================================================================

/// Extract the full protocol spec from a service + operation definition.
fn derive_operation_spec(
    service: &ServiceDef,
    operation: &OperationDef,
    transport: ServiceTransportClass,
) -> Option<ServiceOperationSpec> {
    match transport {
        ServiceTransportClass::RestNetwork => {
            derive_rest_spec(service, operation).map(ServiceOperationSpec::Rest)
        }
        ServiceTransportClass::ShellLocal => {
            derive_shell_spec(service, operation).map(ServiceOperationSpec::Shell)
        }
        _ => None,
    }
}

fn derive_rest_spec(service: &ServiceDef, operation: &OperationDef) -> Option<RestOperationSpec> {
    let endpoint = annotation_string_arg(&service.annotations, "endpoint").unwrap_or_default();
    let (method, path_template) =
        annotation_rest_details(&operation.annotations, &service.annotations)?;

    let input_fields = derive_input_fields(&operation.inputs, &path_template);
    let output_fields = derive_output_fields(&operation.outputs);
    let body_template = annotation_body_template(&operation.annotations);
    let headers = annotation_headers(&operation.annotations, &service.annotations);

    Some(RestOperationSpec {
        endpoint,
        method,
        path_template,
        input_fields,
        output_fields,
        body_template,
        headers,
    })
}

fn derive_shell_spec(service: &ServiceDef, operation: &OperationDef) -> Option<ShellOperationSpec> {
    let argv_template = annotation_shell_argv(&operation.annotations, &service.annotations)?;

    let input_fields = derive_input_fields_for_shell(&operation.inputs, &argv_template);
    let output_fields = derive_output_fields(&operation.outputs);
    let output_parsing = annotation_shell_output_parsing(&operation.annotations)
        .or_else(|| annotation_shell_output_parsing(&service.annotations))
        .unwrap_or_else(|| infer_shell_output_parsing(&operation.outputs));
    let passthrough =
        annotation_shell_passthrough(&operation.annotations, &service.annotations);

    Some(ShellOperationSpec {
        argv_template,
        input_fields,
        output_fields,
        output_parsing,
        passthrough,
    })
}

fn annotation_shell_passthrough(
    op_annotations: &[Annotation],
    service_annotations: &[Annotation],
) -> bool {
    has_annotation(op_annotations, "external")
        || has_annotation(service_annotations, "external")
        || has_annotation(op_annotations, "interactive")
        || has_annotation(service_annotations, "interactive")
}

/// Extract a string argument from a named annotation: `@name("value")`.
fn annotation_string_arg(annotations: &[Annotation], name: &str) -> Option<String> {
    annotations.iter().find(|a| a.name == name).and_then(|a| {
        a.args.first().and_then(|arg| match arg {
            Expr::Literal(Literal::String(s)) => Some(s.clone()),
            _ => None,
        })
    })
}

/// Extract `(method, path_template)` from `@rest(METHOD, "/path/{param}")`.
///
/// The path may be a plain string literal or a string interpolation. Interpolated
/// expressions like `{project}` are converted back to template placeholders `{project}`.
fn annotation_rest_details(
    op_annotations: &[Annotation],
    service_annotations: &[Annotation],
) -> Option<(String, String)> {
    let rest_ann = op_annotations
        .iter()
        .chain(service_annotations.iter())
        .find(|a| a.name == "rest")?;

    if rest_ann.args.len() < 2 {
        return None;
    }

    let method = match &rest_ann.args[0] {
        Expr::Ident(m) => m.clone(),
        Expr::Literal(Literal::String(m)) => m.clone(),
        _ => return None,
    };

    let path = expr_to_template_string(&rest_ann.args[1])?;

    Some((method, path))
}

/// Extract argv template from `@shell(["cmd", "arg", "{param}"])`.
fn annotation_shell_argv(
    op_annotations: &[Annotation],
    service_annotations: &[Annotation],
) -> Option<Vec<ArgvSegment>> {
    for shell_ann in op_annotations
        .iter()
        .chain(service_annotations.iter())
        .filter(|a| a.name == "shell")
    {
        // @shell(["cmd", "arg1", "{param}", ...])
        let Some(list) = shell_ann.args.first() else {
            continue;
        };
        let items = match list {
            Expr::List(items) => items,
            _ => continue,
        };

        let mut segments = Vec::new();
        for item in items {
            match item {
                Expr::Literal(Literal::String(s)) => {
                    let maybe_single_ref = s
                        .strip_prefix('{')
                        .and_then(|inner| inner.strip_suffix('}'))
                        .filter(|inner| {
                            !inner.is_empty()
                                && !inner.contains('{')
                                && !inner.contains('}')
                                && !inner.contains(' ')
                        });
                    if let Some(param) = maybe_single_ref {
                        segments.push(ArgvSegment::InputRef(param.to_string()));
                    } else {
                        // Complex interpolation like "{base}...{head}" remains a literal template.
                        segments.push(ArgvSegment::Literal(s.clone()));
                    }
                }
                Expr::StringInterp(parts) => {
                    // Handle string interpolation: `"{param}"` or `"{base}...{head}"`.
                    use daglang_syntax::ast::StringPart;
                    // Single-expr interpolation like `"{param}"` -> InputRef.
                    if parts.len() == 1 {
                        if let StringPart::Expr(Expr::Ident(name)) = &parts[0] {
                            segments.push(ArgvSegment::InputRef(name.clone()));
                            continue;
                        }
                    }
                    // Multi-part interpolation -> reconstruct as Literal with {param} markers.
                    let template = expr_to_template_string(item).unwrap_or_default();
                    if !template.is_empty() {
                        segments.push(ArgvSegment::Literal(template));
                    }
                }
                _ => {}
            }
        }

        if !segments.is_empty() {
            return Some(segments);
        }
    }

    None
}

/// Extract shell parse mode from `@parse(mode)`.
///
/// Supported aliases:
/// - trim / trim_stdout / string -> TrimStdout
/// - split_lines / lines / line_list -> SplitLines
/// - exit_code_bool / bool / success_bool -> ExitCodeBool
/// - success_stdout_stderr / triple / result -> SuccessStdoutStderr
fn annotation_shell_output_parsing(annotations: &[Annotation]) -> Option<ShellOutputParsing> {
    for ann in annotations.iter().filter(|a| a.name == "parse") {
        let Some(mode_expr) = ann.args.first() else {
            continue;
        };
        let mode_raw = match mode_expr {
            Expr::Ident(mode) => mode.as_str(),
            Expr::Literal(Literal::String(mode)) => mode.as_str(),
            _ => continue,
        };
        let mode = mode_raw.trim().to_ascii_lowercase().replace('-', "_");
        let parsing = match mode.as_str() {
            "trim" | "trim_stdout" | "string" => Some(ShellOutputParsing::TrimStdout),
            "split_lines" | "lines" | "line_list" => Some(ShellOutputParsing::SplitLines),
            "exit_code_bool" | "bool" | "success_bool" => Some(ShellOutputParsing::ExitCodeBool),
            "success_stdout_stderr" | "triple" | "result" => {
                Some(ShellOutputParsing::SuccessStdoutStderr)
            }
            _ => None,
        };
        if parsing.is_some() {
            return parsing;
        }
    }
    None
}

/// Extract body template from `@body_template({ "key": value, ... })`.
fn annotation_body_template(annotations: &[Annotation]) -> Option<Vec<BodyEntry>> {
    let ann = annotations.iter().find(|a| a.name == "body_template")?;
    let record = ann.args.first()?;

    match record {
        Expr::Record(_, fields) => {
            let mut entries = Vec::new();
            for (key, value) in fields {
                match value {
                    Expr::Ident(field_name) => {
                        entries.push(BodyEntry::InputRef(key.clone(), field_name.clone()));
                    }
                    Expr::Literal(Literal::String(s)) => {
                        entries.push(BodyEntry::Literal(key.clone(), s.clone()));
                    }
                    _ => {}
                }
            }
            Some(entries)
        }
        _ => None,
    }
}

/// Extract custom headers from `@headers({ "key": "value", ... })`.
fn annotation_headers(
    op_annotations: &[Annotation],
    service_annotations: &[Annotation],
) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    for ann in op_annotations
        .iter()
        .chain(service_annotations.iter())
        .filter(|a| a.name == "headers")
    {
        if let Some(Expr::Record(_, fields)) = ann.args.first() {
            for (key, value) in fields {
                if let Expr::Literal(Literal::String(v)) = value {
                    headers.push((key.clone(), v.clone()));
                }
            }
        }
    }
    headers
}

/// Derive input field specs from operation inputs.
fn derive_input_fields(
    inputs: &[daglang_syntax::ast::Field],
    path_template: &str,
) -> Vec<FieldSpec> {
    inputs
        .iter()
        .map(|field| {
            let type_id = type_expr_to_string(&field.ty);
            let is_path_param = path_template.contains(&format!("{{{}}}", field.name));
            FieldSpec {
                name: field.name.clone(),
                type_id: type_id.clone(),
                default: field.default.as_ref().map(expr_to_default_string),
                is_secret: type_id == "Secret",
                is_path_param,
            }
        })
        .collect()
}

/// Derive input field specs for shell operations.
fn derive_input_fields_for_shell(
    inputs: &[daglang_syntax::ast::Field],
    argv: &[ArgvSegment],
) -> Vec<FieldSpec> {
    let argv_refs: HashSet<&str> = argv
        .iter()
        .filter_map(|s| match s {
            ArgvSegment::InputRef(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();

    inputs
        .iter()
        .map(|field| {
            let type_id = type_expr_to_string(&field.ty);
            FieldSpec {
                name: field.name.clone(),
                type_id: type_id.clone(),
                default: field.default.as_ref().map(expr_to_default_string),
                is_secret: type_id == "Secret",
                is_path_param: argv_refs.contains(field.name.as_str()),
            }
        })
        .collect()
}

/// Derive output field specs from operation outputs.
fn derive_output_fields(outputs: &[daglang_syntax::ast::Field]) -> Vec<OutputFieldSpec> {
    outputs
        .iter()
        .map(|field| {
            let type_id = type_expr_to_string(&field.ty);
            let json_path = annotation_string_arg(&field.annotations, "json")
                .unwrap_or_else(|| field.name.clone());
            let is_raw_body = has_annotation(&field.annotations, "raw_body");
            OutputFieldSpec {
                name: field.name.clone(),
                type_id: type_id.clone(),
                json_path,
                is_secret: type_id == "Secret",
                is_raw_body,
            }
        })
        .collect()
}

/// Infer shell output parsing mode from output field types.
fn infer_shell_output_parsing(outputs: &[daglang_syntax::ast::Field]) -> ShellOutputParsing {
    // (success: Bool, stdout: String, stderr: String) → SuccessStdoutStderr
    if outputs.len() == 3
        && outputs.iter().any(|f| f.name == "success")
        && outputs.iter().any(|f| f.name == "stdout")
        && outputs.iter().any(|f| f.name == "stderr")
    {
        return ShellOutputParsing::SuccessStdoutStderr;
    }

    // Single Bool output (e.g., "needed", "exists") → ExitCodeBool
    if outputs.len() == 1 {
        let ty = type_expr_to_string(&outputs[0].ty);
        if ty == "Bool" {
            return ShellOutputParsing::ExitCodeBool;
        }
    }

    // Check if any output is a List type → SplitLines
    for field in outputs {
        let ty = type_expr_to_string(&field.ty);
        if ty.starts_with("List<") || ty.starts_with("List ") {
            return ShellOutputParsing::SplitLines;
        }
    }

    // Default: trim stdout
    ShellOutputParsing::TrimStdout
}

/// Convert an expression to a template string, preserving `{param}` placeholders.
///
/// Handles both `Expr::Literal(String("..."))` (plain strings) and
/// `Expr::StringInterp(...)` (interpolated strings like `"/v1/{project}/..."`)
/// by converting interpolation expressions back to `{name}` template syntax.
fn expr_to_template_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(s)) => Some(s.clone()),
        Expr::StringInterp(parts) => {
            use daglang_syntax::ast::StringPart;
            let mut result = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(s) => result.push_str(s),
                    StringPart::Expr(Expr::Ident(name)) => {
                        result.push('{');
                        result.push_str(name);
                        result.push('}');
                    }
                    StringPart::Expr(other) => {
                        // Complex expressions inside interpolation — stringify as-is.
                        result.push('{');
                        result.push_str(&format!("{other:?}"));
                        result.push('}');
                    }
                }
            }
            Some(result)
        }
        _ => None,
    }
}

/// Convert an expression to a default value string for FieldSpec.
fn expr_to_default_string(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s)) => s.clone(),
        Expr::Literal(Literal::Int(n)) => n.to_string(),
        Expr::Literal(Literal::Float(f)) => f.to_string(),
        Expr::Literal(Literal::Bool(b)) => b.to_string(),
        Expr::Literal(Literal::None) => "null".to_string(),
        Expr::Ident(name) => name.clone(),
        _ => String::new(),
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
                let transport_is_interactive = matches!(
                    service_metadata.spec.as_ref(),
                    Some(ServiceOperationSpec::Shell(shell_spec)) if shell_spec.passthrough
                );
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
                        service_metadata: Some(Box::new(service_metadata.clone())),
                        is_interactive: transport_is_interactive,
                        resource_target: None,
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
                        service_metadata: Some(Box::new(service_metadata.clone())),
                        is_interactive: transport_is_interactive,
                        resource_target: None,
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
                        service_metadata: Some(Box::new(service_metadata)),
                        is_interactive: transport_is_interactive,
                        resource_target: None,
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
    active_profile_bindings: Option<&ActiveProfileBindings>,
    known_interface_types: &HashSet<String>,
) -> Result<(), LowerError> {
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let (item_name, params, stmts, uses_bindings, uses_types) = match &item.node {
                Item::FnDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    HashSet::new(),
                    HashMap::new(),
                ),
                Item::FuncDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    def.uses
                        .iter()
                        .map(|usage| usage.binding.clone())
                        .collect::<HashSet<_>>(),
                    def.uses
                        .iter()
                        .map(|usage| {
                            (
                                usage.binding.clone(),
                                canonical_type_id(resource_type_name(&usage.resource_type).as_str())
                                    .0,
                            )
                        })
                        .collect::<HashMap<_, _>>(),
                ),
                Item::PatternDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    def.uses
                        .iter()
                        .map(|usage| usage.binding.clone())
                        .collect::<HashSet<_>>(),
                    def.uses
                        .iter()
                        .map(|usage| {
                            (
                                usage.binding.clone(),
                                canonical_type_id(resource_type_name(&usage.resource_type).as_str())
                                    .0,
                            )
                        })
                        .collect::<HashMap<_, _>>(),
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
            let bound_callable_sources =
                collect_bound_callable_sources(module_name.as_str(), stmts, endpoints_by_full);
            let mut service_calls = Vec::<ServiceCallSite>::new();
            collect_service_calls_from_stmts(stmts, &mut service_calls);
            for (call_index, call) in service_calls.into_iter().enumerate() {
                let resolved_source = resolve_service_endpoint(&call.path, service_registry)
                    .or_else(|| {
                        resolve_profile_bound_service_endpoint(
                            &call.path,
                            &uses_types,
                            service_registry,
                            active_profile_bindings,
                        )
                    });
                let Some(source) = resolved_source else {
                    if call
                        .path
                        .first()
                        .is_some_and(|segment| uses_bindings.contains(segment))
                    {
                        if let Some((profile_name, _)) = active_profile_bindings {
                            let binding = call.path.first().cloned().unwrap_or_default();
                            let interface = uses_types
                                .get(&binding)
                                .cloned()
                                .unwrap_or_else(|| "Unknown".to_string());
                            let short_interface = interface
                                .rsplit('.')
                                .next()
                                .unwrap_or(interface.as_str())
                                .to_string();
                            let is_interface = known_interface_types.contains(interface.as_str())
                                || known_interface_types.contains(short_interface.as_str());
                            if is_interface {
                                return Err(LowerError::MissingProfileBinding {
                                    caller: format!("{module_name}::{item_name}"),
                                    binding,
                                    interface,
                                    profile: profile_name.clone(),
                                });
                            }
                        }
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
                    let Some(prepare_input) = arg
                        .name
                        .as_deref()
                        .or_else(|| source.prepare_inputs.get(index).map(String::as_str))
                    else {
                        continue;
                    };
                    if let Some(arg_ident) = arg.ident.as_deref() {
                        let Some(param_ty) = param_types.get(arg_ident) else {
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
                        continue;
                    }
                    if let Some((base_ident, field_name)) = arg.field_access.as_ref() {
                        if let Some(source_endpoint) = bound_callable_sources.get(base_ident) {
                            builder.add_edge(
                                source_endpoint.node_id.as_str(),
                                field_name.as_str(),
                                source.prepare_node_id.as_str(),
                                prepare_input,
                            );
                            continue;
                        }
                    }
                    let Some(literal) = arg.literal.as_ref() else {
                        continue;
                    };
                    let literal_source = ensure_literal_source_node(
                        builder,
                        module_name.as_str(),
                        item_name,
                        prepare_input,
                        "Any",
                        literal,
                        format!("{call_index}_{index}").as_str(),
                    );
                    builder.add_edge(
                        literal_source.as_str(),
                        prepare_input,
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
                let provider_hint = provider_hint_from_uses_config(usage.config.as_deref())
                    .or_else(|| {
                        provider_hint_from_resource_type_config(resource_type_with_config.as_str())
                    });
                let endpoint = match resolve_resource_endpoint(
                    module_name.as_str(),
                    resource_type.as_str(),
                    provider_hint,
                    project,
                    resource_registry,
                ) {
                    ResourceEndpointResolution::Resolved(endpoint) => endpoint,
                    ResourceEndpointResolution::Ambiguous => {
                        return Err(LowerError::AmbiguousUsedResource {
                            caller: format!("{module_name}::{item_name}"),
                            binding: usage.binding.clone(),
                            resource_type,
                        });
                    }
                    ResourceEndpointResolution::Missing => {
                        if is_known_uses_type(known_uses_types, &resource_type) {
                            continue;
                        }
                        return Err(LowerError::UnresolvedUsedResource {
                            caller: format!("{module_name}::{item_name}"),
                            binding: usage.binding.clone(),
                            resource_type,
                        });
                    }
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
    wired_release_targets: &mut HashSet<(String, String)>,
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
                let endpoint = match resolve_resource_endpoint(
                    module_name.as_str(),
                    resource_type.as_str(),
                    None,
                    project,
                    resource_registry,
                ) {
                    ResourceEndpointResolution::Resolved(endpoint) => Some(endpoint),
                    ResourceEndpointResolution::Ambiguous => {
                        return Err(LowerError::AmbiguousProvidedResource {
                            caller: format!("{module_name}::{item_name}"),
                            binding: provided.binding.clone(),
                            resource_type,
                        });
                    }
                    ResourceEndpointResolution::Missing => {
                        if !is_known_uses_type(known_uses_types, &resource_type) {
                            return Err(LowerError::UnresolvedProvidedResource {
                                caller: format!("{module_name}::{item_name}"),
                                binding: provided.binding.clone(),
                                resource_type,
                            });
                        }
                        None
                    }
                };

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
                        is_interactive: false,
                        resource_target: Some(provided.binding.clone()),
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
                        let key = (release_node.clone(), "resource_handle".to_string());
                        if wired_release_targets.insert(key) {
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
    }
    Ok(())
}

fn resolve_resource_endpoint(
    module_name: &str,
    resource_type: &str,
    provider_hint: Option<ProviderHint>,
    project: &TypedProject,
    registry: &ResourceLifecycleRegistry,
) -> ResourceEndpointResolution {
    if let Some(endpoint) = resolve_concrete_resource_endpoint(module_name, resource_type, registry)
    {
        return ResourceEndpointResolution::Resolved(endpoint);
    }
    resolve_interface_resource_endpoint(resource_type, provider_hint, project, registry)
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
    provider_hint: Option<ProviderHint>,
    project: &TypedProject,
    registry: &ResourceLifecycleRegistry,
) -> ResourceEndpointResolution {
    let target_canonical = canonical_type_id(resource_type);
    let target_short = short_type_name(&target_canonical);
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
            let implemented_canonical = canonical_type_id(implemented);
            let implemented_short = short_type_name(&implemented_canonical);
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
            let candidate_provider =
                provider_hint_from_resource_properties(resource.properties.as_slice())
                    .or_else(|| provider_hint_from_module_name(candidate_module_name.as_str()));
            candidates.push((candidate_provider, endpoint));
        }
    }

    if let Some(required_provider) = provider_hint {
        candidates.retain(|(candidate_provider, _)| {
            candidate_provider.is_some_and(|provider| provider == required_provider)
        });
    }

    match candidates.len() {
        0 => ResourceEndpointResolution::Missing,
        1 => ResourceEndpointResolution::Resolved(
            candidates
                .into_iter()
                .next()
                .map(|(_, endpoint)| endpoint)
                .expect("expected one candidate"),
        ),
        _ => ResourceEndpointResolution::Ambiguous,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceEndpointResolution {
    Resolved(ResourceLifecycleEndpoint),
    Missing,
    Ambiguous,
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

fn collect_interface_type_names(project: &TypedProject) -> HashSet<String> {
    let mut known = HashSet::new();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::InterfaceDef(def) = &item.node else {
                continue;
            };
            insert_canonical_names(&mut known, &def.name);
            insert_canonical_names(&mut known, &format!("{module_name}.{}", def.name));
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
                        is_interactive: false,
                        resource_target: Some(resource.name.clone()),
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
                        is_interactive: false,
                        resource_target: Some(resource.name.clone()),
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
            let contract_count = resolve_interface_contract_count(project, interface_name);
            if contract_count == 0 {
                continue;
            }
            let endpoint = resolve_concrete_resource_endpoint(
                module_name.as_str(),
                resource.name.as_str(),
                resource_registry,
            );
            let canonical_interface = canonical_type_id(interface_name);
            for index in 0..contract_count {
                let node_id = format!(
                    "verify_contract_{}",
                    sanitize_identifier(&format!(
                        "{module_name}_{}_{}_{}",
                        resource.name,
                        canonical_interface.0.as_str(),
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
                            canonical_interface.0.as_str(),
                            index
                        ),
                        obligation: ObligationCategory::InterfaceContractVerification,
                        service_metadata: None,
                        is_interactive: false,
                        resource_target: None,
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

fn resolve_interface_contract_count(project: &TypedProject, interface_name: &str) -> usize {
    let target = canonical_type_id(interface_name);
    let target_short = short_type_name(&target);
    let mut counts = Vec::new();
    for module in &project.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::InterfaceDef(interface) = &item.node else {
                continue;
            };
            let qualified = format!("{module_name}.{}", interface.name);
            let qualified_canonical = canonical_type_id(&qualified);
            let interface_short = interface
                .name
                .rsplit('.')
                .next()
                .unwrap_or(interface.name.as_str());
            let matches_target = if target.0.contains('.') {
                qualified_canonical == target
            } else {
                interface_short == target_short
            };
            if !matches_target {
                continue;
            }
            counts.push(interface.contracts.len());
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

fn resolve_profile_bound_service_endpoint(
    call_path: &[String],
    uses_types: &HashMap<String, String>,
    registry: &ServiceEndpointRegistry,
    active_profile_bindings: Option<&ActiveProfileBindings>,
) -> Option<ServiceTransportEndpoint> {
    let (_, binds) = active_profile_bindings?;
    if call_path.len() != 2 {
        return None;
    }
    let binding = &call_path[0];
    let capability = &call_path[1];
    let interface = uses_types.get(binding)?;
    let implementation = binds
        .get(interface.as_str())
        .or_else(|| {
            let short = interface
                .rsplit('.')
                .next()
                .unwrap_or(interface.as_str())
                .to_string();
            binds.get(short.as_str())
        })?
        .clone();
    let mut implementation_path = implementation
        .split('.')
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>();
    implementation_path.push(capability.clone());
    resolve_service_endpoint(&implementation_path, registry)
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
        vec![Port::with_cardinality(param, ty, Cardinality::ONE)],
        vec![Port::with_cardinality(param, ty, Cardinality::ONE)],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("call_param_source::{callable}::{param}"),
            kind: PrimitiveOpKind::CallParamSource {
                callable: callable.to_string(),
                param: param.to_string(),
            },
        },
    ));
    node_id
}

fn ensure_literal_source_node(
    builder: &mut DagBuilder,
    module_name: &str,
    callable: &str,
    param: &str,
    ty: &str,
    literal: &ServiceCallArgLiteral,
    disambiguator: &str,
) -> String {
    let node_id = format!(
        "literal_source_{}",
        sanitize_identifier(&format!("{module_name}_{callable}_{param}_{disambiguator}"))
    );
    builder.add_node(Node::opaque(
        node_id.clone(),
        vec![],
        vec![Port::with_cardinality(param, ty, Cardinality::ONE)],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("call_literal_source::{}", encode_literal_for_name(literal)),
            kind: PrimitiveOpKind::CallLiteralSource {
                literal: primitive_literal_from_service_literal(literal),
            },
        },
    ));
    node_id
}

fn encode_literal_for_name(literal: &ServiceCallArgLiteral) -> String {
    match literal {
        ServiceCallArgLiteral::String(value) => format!("strhex:{}", hex_encode(value.as_bytes())),
        ServiceCallArgLiteral::Int(value) => format!("int:{value}"),
        ServiceCallArgLiteral::Bool(value) => format!("bool:{value}"),
        ServiceCallArgLiteral::None => "none".to_string(),
    }
}

fn primitive_literal_from_service_literal(literal: &ServiceCallArgLiteral) -> PrimitiveLiteral {
    match literal {
        ServiceCallArgLiteral::String(value) => PrimitiveLiteral::String(value.clone()),
        ServiceCallArgLiteral::Int(value) => PrimitiveLiteral::Int(*value),
        ServiceCallArgLiteral::Bool(value) => PrimitiveLiteral::Bool(*value),
        ServiceCallArgLiteral::None => PrimitiveLiteral::Unit,
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn item_callable_body(item: &Item) -> Option<(&str, &[Stmt])> {
    match item {
        Item::FnDef(def) => Some((def.name.as_str(), def.body.stmts.as_slice())),
        Item::FuncDef(def) => Some((def.name.as_str(), def.body.stmts.as_slice())),
        Item::PatternDef(def) => Some((def.name.as_str(), def.body.stmts.as_slice())),
        _ => None,
    }
}

fn item_callable_interactive_flag(item: &Item) -> Option<(&str, bool)> {
    match item {
        Item::FnDef(def) => Some((def.name.as_str(), false)),
        Item::FuncDef(def) => Some((
            def.name.as_str(),
            has_annotation(&def.annotations, "interactive"),
        )),
        Item::PatternDef(def) => Some((def.name.as_str(), false)),
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
    field_access: Option<(String, String)>,
    literal: Option<ServiceCallArgLiteral>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceCallSite {
    path: Vec<String>,
    args: Vec<ServiceCallArgSite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceCallArgLiteral {
    String(String),
    Int(i64),
    Bool(bool),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionOpKind {
    Map,
    Filter,
    Fold,
    Join,
    FlatMap,
    Sort,
    Dedup,
    Any,
    All,
    Len,
    Contains,
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
        "sort" => Some(CollectionOpKind::Sort),
        "dedup" => Some(CollectionOpKind::Dedup),
        "any" => Some(CollectionOpKind::Any),
        "all" => Some(CollectionOpKind::All),
        "len" | "count" => Some(CollectionOpKind::Len),
        "contains" => Some(CollectionOpKind::Contains),
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
        CollectionOpKind::Sort => "SortNode",
        CollectionOpKind::Dedup => "DedupNode",
        CollectionOpKind::Any => "AnyNode",
        CollectionOpKind::All => "AllNode",
        CollectionOpKind::Len => "LenNode",
        CollectionOpKind::Contains => "ContainsNode",
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
                        field_access: match arg {
                            Expr::FieldAccess(base, field) => match base.as_ref() {
                                Expr::Ident(base_ident) => {
                                    Some((base_ident.clone(), field.clone()))
                                }
                                _ => None,
                            },
                            _ => None,
                        },
                        literal: service_call_literal_arg(arg),
                    })
                    .collect::<Vec<_>>(),
            });
        }
    });
}

fn service_call_literal_arg(arg: &Expr) -> Option<ServiceCallArgLiteral> {
    match arg {
        Expr::Literal(Literal::String(value)) => Some(ServiceCallArgLiteral::String(value.clone())),
        Expr::Literal(Literal::Int(value)) => Some(ServiceCallArgLiteral::Int(*value)),
        Expr::Literal(Literal::Bool(value)) => Some(ServiceCallArgLiteral::Bool(*value)),
        Expr::Literal(Literal::None) => Some(ServiceCallArgLiteral::None),
        _ => None,
    }
}

fn collect_bound_callable_sources(
    module_name: &str,
    stmts: &[Stmt],
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
) -> HashMap<String, LoweredEndpoint> {
    let mut bound = HashMap::<String, LoweredEndpoint>::new();
    let module_key = module_name.to_string();
    for stmt in stmts {
        match stmt {
            Stmt::Let(binding, expr) | Stmt::Assign(binding, expr) => match expr {
                Expr::Call(name, _) => {
                    if let Some(endpoint) =
                        endpoints_by_full.get(&(module_key.clone(), name.clone()))
                    {
                        bound.insert(binding.clone(), endpoint.clone());
                    }
                }
                Expr::Ident(source) => {
                    if let Some(endpoint) = bound.get(source).cloned() {
                        bound.insert(binding.clone(), endpoint);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    bound
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_resolve::{ModuleGraph, ResolvedModule};
    use daglang_syntax::parser;
    use daglang_typecheck::typecheck_module_graph;
    use gunbc_clippy::build_clippy_graph_lint_all;
    use gunbc_dag::{
        build_bootstrap_graph, build_build_graph, build_ci_graph, build_codegen_graph,
        build_docgen_graph, build_makegen_graph, build_pragma_graph,
    };
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

    fn lower_target_module_with_dependency_scope(
        typed: &TypedProject,
        module_name: &str,
    ) -> Dag<LoweredOp> {
        let module_lookup = typed
            .modules
            .iter()
            .enumerate()
            .map(|(index, module)| (module.module_path.join("."), index))
            .collect::<HashMap<_, _>>();
        let target_index = typed
            .modules
            .iter()
            .position(|module| module.module_path.join(".") == module_name)
            .expect("target module should exist in typed project");
        let mut scope = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([target_index]);
        while let Some(module_index) = queue.pop_front() {
            if !visited.insert(module_index) {
                continue;
            }
            let Some(module) = typed.modules.get(module_index) else {
                continue;
            };
            scope.insert(module.module_path.join("."));
            for import in &module.imports {
                let import_name = import.join(".");
                if let Some(import_index) = module_lookup.get(&import_name) {
                    queue.push_back(*import_index);
                }
            }
        }
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
    fn collect_collection_ops_detects_extended_collection_intrinsics() {
        let stmts = callable_stmts_from_source(
            r#"
module sample.collections
fn run(values: List<String>) -> { out: Int } {
  evaluated = values
    |> sort()
    |> dedup()
    |> contains("needle")
    |> len()
  return { out: evaluated }
}
"#,
        );
        let mut sites = Vec::new();
        collect_collection_ops_from_stmts(&stmts, &mut sites);
        let kinds = sites.iter().map(|site| site.kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                CollectionOpKind::Len,
                CollectionOpKind::Contains,
                CollectionOpKind::Dedup,
                CollectionOpKind::Sort,
            ]
        );
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
            CollectionOpKind::Sort => 5,
            CollectionOpKind::Dedup => 6,
            CollectionOpKind::Any => 7,
            CollectionOpKind::All => 8,
            CollectionOpKind::Len => 9,
            CollectionOpKind::Contains => 10,
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
    fn lower_typed_project_emits_control_flow_nodes_for_if_for_match() {
        let typed = typed_project_from_sources(&[(
            "sample.control",
            r#"
module sample.control

fn run(values: List<Int>, gate: Bool, mode: String) -> Int {
  let iterated = for value in values {
    if gate { value } else { 0 }
  }
  let chosen = match mode {
    "hot" => 1
    _ => 0
  }
  let final = if gate { chosen } else { 0 }
  final
}
"#,
        )]);
        let fn_body = typed
            .modules
            .first()
            .and_then(|module| module.ast.items.first())
            .and_then(|item| match &item.node {
                Item::FnDef(def) => Some(&def.body),
                _ => None,
            })
            .expect("sample source should contain a fn body");
        assert!(
            !fn_body.lossy,
            "expected non-lossy parsed function body for control-flow fixture"
        );
        assert!(
            !fn_body.stmts.is_empty(),
            "expected control-flow fixture to retain parsed statements"
        );
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let node_ids = dag
            .nodes
            .iter()
            .map(|node| node.id.0.as_str())
            .collect::<HashSet<_>>();
        assert!(
            node_ids.contains("sample.control::run::cf_for_0"),
            "node ids: {node_ids:?}"
        );
        assert!(
            node_ids.contains("sample.control::run::cf_if_0"),
            "node ids: {node_ids:?}"
        );
        assert!(
            node_ids.contains("sample.control::run::cf_if_1"),
            "node ids: {node_ids:?}"
        );
        assert!(
            node_ids.contains("sample.control::run::cf_match_0"),
            "node ids: {node_ids:?}"
        );
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.control::run::cf_for_0"
                && edge.to_node.0 == "sample.control::run"
                && edge.to_port.0 == "__deps"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.control::run::cf_match_0"
                && edge.to_node.0 == "sample.control::run"
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn expr_to_template_string_preserves_identifier_interpolation() {
        let expr = Expr::StringInterp(vec![
            daglang_syntax::ast::StringPart::Literal("prefix-".to_string()),
            daglang_syntax::ast::StringPart::Expr(Expr::Ident("left".to_string())),
            daglang_syntax::ast::StringPart::Literal("-".to_string()),
            daglang_syntax::ast::StringPart::Expr(Expr::Ident("right".to_string())),
        ]);
        assert_eq!(
            expr_to_template_string(&expr),
            Some("prefix-{left}-{right}".to_string())
        );
    }

    #[test]
    fn gcp_credential_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.gcp.credential");
        let dag = lower_target_module_with_dependency_scope(&typed, "cloud.gcp.credential");
        let reference = build_gcp_secret_manager_credential_graph_github().unwrap();

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn clippy_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.clippy");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.clippy");
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
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.deps");
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
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.gist");
        let reference = build_gist_graph(GistMode::Snapshot, Vec::new(), false)
            .expect("gist builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn gist_snapshot_normalized_parity_can_reach_exact_match() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.gist");
        let reference = build_gist_graph(GistMode::Snapshot, Vec::new(), false)
            .expect("gist builder graph should be available");
        let report = compare_gist_topology(&dag, &reference, GistParityMode::Snapshot);
        assert!(
            report.is_exact_match(),
            "normalized gist snapshot parity should match reference topology: {report:?}"
        );
    }

    #[test]
    fn gcp_credential_normalized_parity_can_reach_exact_match() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.gcp.credential");
        let dag = lower_target_module_with_dependency_scope(&typed, "cloud.gcp.credential");
        let reference = build_gcp_secret_manager_credential_graph_github().unwrap();
        let report = compare_gcp_credential_topology(&dag, &reference);
        assert!(
            report.is_exact_match(),
            "normalized gcp credential parity should match reference topology: {report:?}"
        );
    }

    #[test]
    fn gcp_credential_normalized_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.gcp.credential");
        let dag = lower_target_module_with_dependency_scope(&typed, "cloud.gcp.credential");
        let reference = build_gcp_secret_manager_credential_graph_github().unwrap();
        let report_a = compare_gcp_credential_topology(&dag, &reference);
        let report_b = compare_gcp_credential_topology(&dag, &reference);
        assert_eq!(
            report_a, report_b,
            "normalized gcp parity report should be deterministic"
        );
        assert!(
            report_a.is_exact_match(),
            "normalized gcp parity should remain exact-match"
        );
    }

    #[test]
    fn gist_diff_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.gist");
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
    fn gist_diff_normalized_parity_can_reach_exact_match() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.gist");
        let reference = build_gist_graph(
            GistMode::Diff {
                base_ref: "main".to_string(),
            },
            Vec::new(),
            false,
        )
        .expect("gist builder graph should be available");
        let report = compare_gist_topology(&dag, &reference, GistParityMode::Diff);
        assert!(
            report.is_exact_match(),
            "normalized gist diff parity should match reference topology: {report:?}"
        );
    }

    #[test]
    fn gist_recent_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.gist");
        let reference = build_gist_graph(GistMode::Recent, Vec::new(), false)
            .expect("gist builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn gist_recent_normalized_parity_can_reach_exact_match() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.gist");
        let reference = build_gist_graph(GistMode::Recent, Vec::new(), false)
            .expect("gist builder graph should be available");
        let report = compare_gist_topology(&dag, &reference, GistParityMode::Recent);
        assert!(
            report.is_exact_match(),
            "normalized gist recent parity should match reference topology: {report:?}"
        );
    }

    #[test]
    fn gist_dependency_closure_lowering_reuses_shared_credential_chain() {
        let typed = typed_project_for_module_with_dependency_closure("tools.gist");
        let scope = typed
            .modules
            .iter()
            .map(|module| module.module_path.join("."))
            .collect::<HashSet<_>>();
        let dag = lower_typed_project_for_modules(&typed, &scope).expect("lowering should succeed");

        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "shared.gist_modes::share_content"));
        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "shared.gist_modes::gist_upload"));
        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "std.patterns::credential_chain"));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "std.patterns::credential_chain"
                && edge.to_node.0 == "shared.gist_modes::gist_upload"
                && edge.to_port.0 == "__deps"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "shared.gist_modes::share_content"
                && edge.to_node.0 == "tools.gist::gist_snapshot"
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn aws_credential_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.aws.credential");
        let dag = lower_target_module_with_dependency_scope(&typed, "cloud.aws.credential");
        let reference = build_aws_secrets_manager_credential_graph()
            .expect("aws credential graph should build");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn azure_credential_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("cloud.azure.credential");
        let dag = lower_target_module_with_dependency_scope(&typed, "cloud.azure.credential");
        let reference =
            build_azure_key_vault_credential_graph().expect("azure credential graph should build");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn ci_pipeline_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("pipelines.ci");
        let dag = lower_target_module_with_dependency_scope(&typed, "pipelines.ci");
        let reference = build_ci_graph().expect("ci builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn ci_pipeline_normalized_parity_can_reach_exact_match() {
        let typed = typed_project_for_module_with_dependency_closure("pipelines.ci");
        let dag = lower_target_module_with_dependency_scope(&typed, "pipelines.ci");
        let reference = build_ci_graph().expect("ci builder graph should be available");
        let report = compare_ci_topology(&dag, &reference);
        assert!(
            report.is_exact_match(),
            "normalized ci parity should match reference topology: {report:?}"
        );
    }

    #[test]
    fn ci_pipeline_normalized_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("pipelines.ci");
        let dag = lower_target_module_with_dependency_scope(&typed, "pipelines.ci");
        let reference = build_ci_graph().expect("ci builder graph should be available");
        let report_a = compare_ci_topology(&dag, &reference);
        let report_b = compare_ci_topology(&dag, &reference);
        assert_eq!(
            report_a, report_b,
            "normalized ci parity report should be deterministic"
        );
        assert!(
            report_a.is_exact_match(),
            "normalized ci parity should remain exact-match"
        );
    }

    #[test]
    fn bootstrap_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.bootstrap");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.bootstrap");
        let reference =
            build_bootstrap_graph().expect("bootstrap builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn codegen_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.codegen");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.codegen");
        let reference = build_codegen_graph().expect("codegen builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn build_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.build");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.build");
        let reference = build_build_graph().expect("build builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn pragma_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.pragma");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.pragma");
        let reference = build_pragma_graph().expect("pragma builder graph should be available");

        let report_a = compare_ir(&dag, &reference);
        let report_b = compare_ir(&dag, &reference);
        assert_eq!(report_a, report_b);
        assert!(report_a.candidate_nodes > 0);
        assert!(report_a.reference_nodes > 0);
    }

    #[test]
    fn docgen_parity_report_is_deterministic() {
        let typed = typed_project_for_module_with_dependency_closure("tools.docgen");
        let dag = lower_target_module_with_dependency_scope(&typed, "tools.docgen");
        let reference = build_docgen_graph().expect("docgen builder graph should be available");

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
            10,
            "expected callable + content_upsert chain + source scaffold nodes + param path source"
        );
        let required_edges = [
            (
                "prepare_read_makegen",
                "request",
                "execute_read_makegen",
                "request",
            ),
            (
                "prepare_read_makegen",
                "skip",
                "execute_read_makegen",
                "skip",
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
                "response",
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
        let param_source_node = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == "param_source_tools_makegen_makegen_path")
            .expect("param source node should be present for content_upsert path");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node == param_source_node.id
                && edge.from_port.0 == "path"
                && edge.to_node.0 == "prepare_read_makegen"
                && edge.to_port.0 == "path"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node == param_source_node.id
                && edge.from_port.0 == "path"
                && edge.to_node.0 == "prepare_write_makegen"
                && edge.to_port.0 == "path"
        }));
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

    fn shell_output_parsing_for_node(dag: &Dag<LoweredOp>, node_id: &str) -> ShellOutputParsing {
        let node = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == node_id)
            .expect("transport node should exist");
        let metadata = match &node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => op
                .service_call_metadata()
                .expect("service metadata should be present"),
            gunbc_ir::node::NodeBody::SubDag(_) => {
                panic!("expected opaque lowered node for transport metadata")
            }
        };
        let spec = metadata
            .spec
            .as_ref()
            .expect("service metadata should include operation spec");
        match spec {
            ServiceOperationSpec::Shell(spec) => spec.output_parsing,
            other => panic!("expected shell operation spec, got {other:?}"),
        }
    }

    fn shell_passthrough_for_node(dag: &Dag<LoweredOp>, node_id: &str) -> bool {
        let node = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == node_id)
            .expect("transport node should exist");
        let metadata = match &node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => op
                .service_call_metadata()
                .expect("service metadata should be present"),
            gunbc_ir::node::NodeBody::SubDag(_) => {
                panic!("expected opaque lowered node for transport metadata")
            }
        };
        let spec = metadata
            .spec
            .as_ref()
            .expect("service metadata should include operation spec");
        match spec {
            ServiceOperationSpec::Shell(spec) => spec.passthrough,
            other => panic!("expected shell operation spec, got {other:?}"),
        }
    }

    #[test]
    fn shell_external_annotation_sets_passthrough_semantics() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/shell_external_passthrough.dag",
            r#"module sample.services
service shell.Auth {
  @shell(["gcloud", "auth", "login"])
  @external
  operation Login() -> { ok: Bool }
}
func run() -> { ok: Bool } {
  result = shell.Auth.Login()
  return { ok: result.ok }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let prepare_id = "prepare_transport_sample_services_shell_Auth_Login";
        let execute_id = "execute_transport_sample_services_shell_Auth_Login";
        let parse_id = "parse_transport_sample_services_shell_Auth_Login";
        assert!(
            shell_passthrough_for_node(&dag, prepare_id),
            "prepare node should preserve passthrough metadata"
        );
        assert!(
            shell_passthrough_for_node(&dag, execute_id),
            "execute node should preserve passthrough metadata"
        );
        assert!(
            shell_passthrough_for_node(&dag, parse_id),
            "parse node should preserve passthrough metadata"
        );
        for node_id in [prepare_id, execute_id, parse_id] {
            let node = dag
                .nodes
                .iter()
                .find(|node| node.id.0 == node_id)
                .expect("transport node should exist");
            let is_interactive = match &node.body {
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable {
                    is_interactive, ..
                }) => *is_interactive,
                _ => panic!("expected callable lowered op"),
            };
            assert!(
                is_interactive,
                "transport node `{node_id}` should be marked interactive"
            );
        }
    }

    #[test]
    fn shell_parse_annotation_overrides_inferred_parse_mode() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/shell_parse_override.dag",
            r#"module sample.services
@shell
service shell.Tools {
  @shell
  @shell(["echo", "{value}"])
  @parse(split_lines)
  operation Echo(value: String) -> { needed: Bool }
}
func run() -> { needed: Bool } {
  result = shell.Tools.Echo(value: "hello")
  return { needed: result.needed }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let parsing = shell_output_parsing_for_node(
            &dag,
            "execute_transport_sample_services_shell_Tools_Echo",
        );
        assert_eq!(parsing, ShellOutputParsing::SplitLines);
    }

    #[test]
    fn shell_parse_annotation_prefers_operation_over_service() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/shell_parse_precedence.dag",
            r#"module sample.services
@shell
@parse(result)
service shell.Tools {
  operation Echo(value: String, suffix: String) -> { lines: List<String> } {
    @shell(["echo", "{value}:{suffix}"])
    @parse(exit_code_bool)
  }
}
func run() -> { lines: List<String> } {
  result = shell.Tools.Echo(value: "a", suffix: "b")
  return { lines: result.lines }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let parsing = shell_output_parsing_for_node(
            &dag,
            "execute_transport_sample_services_shell_Tools_Echo",
        );
        assert_eq!(parsing, ShellOutputParsing::ExitCodeBool);
    }

    #[test]
    fn shell_argv_annotation_supports_string_interpolation_templates() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/shell_argv_templates.dag",
            r#"module sample.services
@shell
service shell.Tools {
  @shell(["echo", "{value}:{suffix}", "{value}"])
  operation Echo(value: String, suffix: String) -> { out: String }
}
func run() -> { out: String } {
  result = shell.Tools.Echo(value: "alpha", suffix: "beta")
  return { out: result.out }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let node = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == "prepare_transport_sample_services_shell_Tools_Echo")
            .expect("prepare node should exist");
        let metadata = match &node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => op
                .service_call_metadata()
                .expect("service metadata should be present"),
            gunbc_ir::node::NodeBody::SubDag(_) => {
                panic!("expected opaque lowered node for prepare transport")
            }
        };
        let spec = metadata
            .spec
            .as_ref()
            .expect("service metadata should include operation spec");
        let shell = match spec {
            ServiceOperationSpec::Shell(shell) => shell,
            other => panic!("expected shell operation spec, got {other:?}"),
        };
        assert_eq!(
            shell.argv_template[0],
            ArgvSegment::Literal("echo".to_string())
        );
        assert_eq!(
            shell.argv_template[1],
            ArgvSegment::Literal("{value}:{suffix}".to_string())
        );
        assert_eq!(
            shell.argv_template[2],
            ArgvSegment::InputRef("value".to_string())
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
    fn profile_binding_resolves_uses_bound_interface_service_calls() {
        let typed = typed_project_from_sources(&[
            (
                "dsl/profiles/local.dag",
                r#"module sample.profiles
profile local {
  bind IssueProvider -> github.Issues
}"#,
            ),
            (
                "dsl/services/issues.dag",
                r#"module sample.services
interface IssueProvider {
  capability get {
    input { id: String }
    output { body: String }
  }
}
service github.Issues implements IssueProvider {
  operation get(id: String) -> { body: String }
}
func run(id: String) -> { body: String } uses issues: IssueProvider {
  result = issues.get(id: id)
  return { body: result.body }
}"#,
            ),
        ]);

        let dag = lower_typed_project_with_profile(&typed, Some("local"))
            .expect("lowering with active profile should resolve bound service calls");
        assert!(dag
            .nodes
            .iter()
            .any(|node| node.id.0 == "prepare_transport_sample_services_github_Issues_get"));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "parse_transport_sample_services_github_Issues_get"
                && edge.to_node.0 == "sample.services::run"
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn active_profile_requires_bindings_for_used_interfaces() {
        let typed = typed_project_from_sources(&[
            (
                "dsl/profiles/local.dag",
                r#"module sample.profiles
profile local {
}"#,
            ),
            (
                "dsl/services/issues.dag",
                r#"module sample.services
interface IssueProvider {
  capability get {
    input { id: String }
    output { body: String }
  }
}
service github.Issues implements IssueProvider {
  operation get(id: String) -> { body: String }
}
func run(id: String) -> { body: String } uses issues: IssueProvider {
  result = issues.get(id: id)
  return { body: result.body }
}"#,
            ),
        ]);
        let error = lower_typed_project_with_profile(&typed, Some("local"))
            .expect_err("lowering should fail when active profile lacks uses binding");
        assert!(matches!(
            error,
            LowerError::MissingProfileBinding { binding, interface, .. }
                if binding == "issues" && interface == "IssueProvider"
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
    fn literal_service_call_args_wire_to_prepare_inputs() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/storage_calls_literal.dag",
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
func run() -> { body: String } {
  let response = FsStorage.read(path: "crates")
  return { body: response.body }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let literal_node = dag
            .nodes
            .iter()
            .find(|node| {
                matches!(
                    &node.body,
                    gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive {
                        kind: PrimitiveOpKind::CallLiteralSource {
                            literal: PrimitiveLiteral::String(value)
                        },
                        ..
                    }) if value == "crates"
                )
            })
            .expect("literal source node should be present");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node == literal_node.id
                && edge.from_port.0 == "path"
                && edge.to_node.0 == "prepare_transport_sample_services_FsStorage_read"
                && edge.to_port.0 == "path"
        }));
    }

    #[test]
    fn field_access_service_call_args_wire_to_prepare_inputs() {
        let typed = typed_project_from_sources(&[(
            "dsl/services/storage_calls_field_access.dag",
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
func make_path() -> { path: String } {
  return { path: "crates" }
}
func run() -> { body: String } {
  let req = make_path()
  let response = FsStorage.read(path: req.path)
  return { body: response.body }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "sample.services::make_path"
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
  provider: Gcp
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
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
    fn uses_interface_without_provider_hint_fails_when_multiple_providers_exist() {
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
  provider: Gcp
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
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
        let error = lower_typed_project(&typed).expect_err("lowering should fail");
        assert!(matches!(
            error,
            LowerError::AmbiguousUsedResource {
                caller,
                binding,
                resource_type,
            } if caller == "infra.providers::run"
                && binding == "store"
                && resource_type == "ObjectStorage"
        ));
    }

    #[test]
    fn uses_interface_with_aws_and_azure_provider_hints_wire_matching_resources() {
        let typed = typed_project_from_sources(&[(
            "dsl/infra/providers_multi_cloud.dag",
            r#"module infra.providers
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource GcsBucket implements ObjectStorage {
  provider: Gcp
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource BlobContainer implements ObjectStorage {
  provider: Azure
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run_aws() -> { ok: Bool } uses store: ObjectStorage(cloud: AwsConfig) {
  return { ok: true }
}
func run_azure() -> { ok: Bool } uses store: ObjectStorage(cloud: AzureConfig) {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");

        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_providers_S3Bucket"
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == "infra.providers::run_aws"
                && edge.to_port.0 == "__deps"
        }));
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_providers_BlobContainer"
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == "infra.providers::run_azure"
                && edge.to_port.0 == "__deps"
        }));
        assert!(
            !dag.edges.iter().any(|edge| {
                edge.from_node.0 == "acquire_resource_infra_providers_GcsBucket"
                    && (edge.to_node.0 == "infra.providers::run_aws"
                        || edge.to_node.0 == "infra.providers::run_azure")
                    && edge.to_port.0 == "__deps"
            }),
            "aws/azure provider hints should not wire unrelated gcp resources"
        );
    }

    #[test]
    fn store_artifact_portability_wires_gcp_aws_and_azure_resources() {
        let typed = typed_project_from_sources(&[(
            "dsl/examples/portability.dag",
            r#"module examples.portability
interface ObjectStorage {
  capability write {
    input { key: String, content: String }
    output { ok: Bool }
  }
}
resource GcsBucket implements ObjectStorage {
  provider: Gcp
  acquire { let ready = true }
  capability write {
    input { key: String, content: String }
    output { ok: Bool }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
  acquire { let ready = true }
  capability write {
    input { key: String, content: String }
    output { ok: Bool }
  }
}
resource BlobContainer implements ObjectStorage {
  provider: Azure
  acquire { let ready = true }
  capability write {
    input { key: String, content: String }
    output { ok: Bool }
  }
}
func store_artifact_gcp(key: String, content: String) -> { ok: Bool } uses store: ObjectStorage(cloud: GcpConfig) {
  result = store.write(key: key, content: content)
  return { ok: result.ok }
}
func store_artifact_aws(key: String, content: String) -> { ok: Bool } uses store: ObjectStorage(cloud: AwsConfig) {
  result = store.write(key: key, content: content)
  return { ok: result.ok }
}
func store_artifact_azure(key: String, content: String) -> { ok: Bool } uses store: ObjectStorage(cloud: AzureConfig) {
  result = store.write(key: key, content: content)
  return { ok: result.ok }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");

        for (resource, target) in [
            ("GcsBucket", "examples.portability::store_artifact_gcp"),
            ("S3Bucket", "examples.portability::store_artifact_aws"),
            (
                "BlobContainer",
                "examples.portability::store_artifact_azure",
            ),
        ] {
            let acquire = format!("acquire_resource_examples_portability_{resource}");
            assert!(
                dag.edges.iter().any(|edge| {
                    edge.from_node.0 == acquire
                        && edge.from_port.0 == "resource_handle"
                        && edge.to_node.0 == target
                        && edge.to_port.0 == "__deps"
                }),
                "expected provider-specific resource wiring for {target}"
            );
        }
    }

    #[test]
    fn cross_provider_auth_calls_resolve_all_credential_chains() {
        let typed = typed_project_from_sources(&[
            (
                "dsl/cloud/gcp/credential.dag",
                r#"module cloud.gcp.credential
func acquire_gcp_secret() -> { token: String } {
  return { token: "gcp" }
}"#,
            ),
            (
                "dsl/cloud/aws/credential.dag",
                r#"module cloud.aws.credential
func acquire_aws_secret() -> { token: String } {
  return { token: "aws" }
}"#,
            ),
            (
                "dsl/cloud/azure/credential.dag",
                r#"module cloud.azure.credential
func acquire_azure_secret() -> { token: String } {
  return { token: "azure" }
}"#,
            ),
            (
                "dsl/examples/auth.dag",
                r#"module examples.auth
import cloud.gcp.credential { acquire_gcp_secret }
import cloud.aws.credential { acquire_aws_secret }
import cloud.azure.credential { acquire_azure_secret }

func cross_provider_auth() -> { ok: Bool } {
  gcp = acquire_gcp_secret()
  aws = acquire_aws_secret()
  azure = acquire_azure_secret()
  return { ok: true }
}"#,
            ),
        ]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let caller = "examples.auth::cross_provider_auth";
        for callee in [
            "cloud.gcp.credential::acquire_gcp_secret",
            "cloud.aws.credential::acquire_aws_secret",
            "cloud.azure.credential::acquire_azure_secret",
        ] {
            assert!(
                dag.edges
                    .iter()
                    .any(|edge| edge.from_node.0 == callee && edge.to_node.0 == caller),
                "expected dependency edge from {callee} into cross-provider auth caller"
            );
        }
    }

    #[test]
    fn aws_resource_module_emits_object_storage_contract_verification_nodes() {
        let typed = typed_project_for_module_with_dependency_closure("infra.aws.resources");
        let object_storage_contracts = typed
            .modules
            .iter()
            .find(|module| module.module_path.join(".") == "infra.core")
            .and_then(|module| {
                module.ast.items.iter().find_map(|item| match &item.node {
                    Item::InterfaceDef(interface) if interface.name == "ObjectStorage" => {
                        Some(interface.contracts.len())
                    }
                    _ => None,
                })
            })
            .expect("infra.core.ObjectStorage interface should exist");
        assert!(
            object_storage_contracts > 0,
            "infra.core.ObjectStorage should carry @contract annotations"
        );
        let dag = lower_target_module(&typed, "infra.aws.resources");

        let verify_node = "verify_contract_infra_aws_resources_S3Bucket_ObjectStorage_0";
        assert!(
            dag.nodes.iter().any(|node| node.id.0 == verify_node),
            "aws resources should emit object-storage contract verification nodes"
        );
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_aws_resources_S3Bucket"
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == verify_node
                && edge.to_port.0 == "__deps"
        }));
    }

    #[test]
    fn azure_resource_module_emits_object_storage_contract_verification_nodes() {
        let typed = typed_project_for_module_with_dependency_closure("infra.azure.resources");
        let dag = lower_target_module(&typed, "infra.azure.resources");

        let verify_node = "verify_contract_infra_azure_resources_BlobContainer_ObjectStorage_0";
        assert!(
            dag.nodes.iter().any(|node| node.id.0 == verify_node),
            "azure resources should emit object-storage contract verification nodes"
        );
        assert!(dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_azure_resources_BlobContainer"
                && edge.from_port.0 == "resource_handle"
                && edge.to_node.0 == verify_node
                && edge.to_port.0 == "__deps"
        }));
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
    fn interface_contract_annotations_cover_all_provider_implementors() {
        let typed = typed_project_from_sources(&[(
            "dsl/infra/contracts_multi_cloud.dag",
            r#"module infra.contracts
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
  @contract: read("k") => { body: "v" }
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
resource BlobContainer implements ObjectStorage {
  acquire { let ready = true }
  capability read {
    input { path: String }
    output { body: String }
  }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");

        for (resource, verify_id) in [
            (
                "GcsBucket",
                "verify_contract_infra_contracts_GcsBucket_ObjectStorage_0",
            ),
            (
                "S3Bucket",
                "verify_contract_infra_contracts_S3Bucket_ObjectStorage_0",
            ),
            (
                "BlobContainer",
                "verify_contract_infra_contracts_BlobContainer_ObjectStorage_0",
            ),
        ] {
            let acquire_id = format!("acquire_resource_infra_contracts_{resource}");
            assert!(
                dag.nodes.iter().any(|node| node.id.0 == verify_id),
                "expected verification node for {resource}"
            );
            assert!(
                dag.edges.iter().any(|edge| {
                    edge.from_node.0 == acquire_id
                        && edge.from_port.0 == "resource_handle"
                        && edge.to_node.0 == verify_id
                        && edge.to_port.0 == "__deps"
                }),
                "expected acquire->verify edge for {resource}"
            );
        }
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
    fn ambiguous_provides_clause_reports_lower_error() {
        let typed = typed_project_from_sources(&[(
            "dsl/resources/ambiguous_provides.dag",
            r#"module sample.resources
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource LocalStore implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource BackupStore implements Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}"#,
        )]);
        let error = lower_typed_project(&typed).expect_err("lowering should fail");
        assert!(matches!(
            error,
            LowerError::AmbiguousProvidedResource {
                caller,
                binding,
                resource_type,
            } if caller == "sample.resources::run"
                && binding == "out"
                && resource_type == "Storage"
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
            is_interactive: false,
            resource_target: None,
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

    #[test]
    fn interactive_func_annotation_sets_structural_callable_metadata() {
        let typed = typed_project_from_sources(&[(
            "dsl/interactive.dag",
            r#"module sample.ui
@interactive
func prompt() -> { ok: Bool } {
  return { ok: true }
}"#,
        )]);
        let dag = lower_typed_project(&typed).expect("lowering should succeed");
        let op = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == "sample.ui::prompt")
            .and_then(|node| match &node.body {
                gunbc_ir::node::NodeBody::Opaque(op) => Some(op),
                gunbc_ir::node::NodeBody::SubDag(_) => None,
            })
            .expect("interactive callable node should exist");
        assert!(matches!(
            op,
            LoweredOp::Callable {
                is_interactive: true,
                ..
            }
        ));
    }

    #[test]
    fn canonical_kind_for_obligation_maps_categories() {
        assert_eq!(
            canonical_kind_for_obligation(ObligationCategory::None),
            None
        );
        assert_eq!(
            canonical_kind_for_obligation(ObligationCategory::ServiceTransportExecute),
            Some("transport")
        );

        for obligation in [
            ObligationCategory::ServiceTransportPrepare,
            ObligationCategory::ServiceTransportParse,
            ObligationCategory::ServiceParamSource,
            ObligationCategory::ResourceProvide,
            ObligationCategory::ResourceAcquire,
            ObligationCategory::ResourceRelease,
            ObligationCategory::InterfaceContractVerification,
        ] {
            assert_eq!(
                canonical_kind_for_obligation(obligation),
                Some("pattern-expanded")
            );
        }
    }

    #[test]
    fn topology_with_obligation_kinds_populates_canonical_kind_metadata() {
        let mut dag: Dag<LoweredOp> = Dag::new();
        dag.add_node(Node::opaque(
            "execute_transport_sample",
            vec![],
            vec![],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "execute_transport_sample".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));

        let topo = topology_with_obligation_kinds(&dag);
        assert_eq!(topo.nodes.len(), 1);
        assert_eq!(topo.nodes[0].canonical_kind.as_deref(), Some("transport"));
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
                is_interactive: false,
                resource_target: None,
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
                is_interactive: false,
                resource_target: None,
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
                is_interactive: false,
                resource_target: None,
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
            vec![Port::scalar("response", "TransportResponse")],
            (),
        ));
        dag.add_node(Node::opaque(
            "param_source_tools_makegen_makegen_path",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("path", "String")],
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
        dag.add_edge(Edge::new(
            "param_source_tools_makegen_makegen_path",
            "path",
            "prepare_read_makegen",
            "path",
        ));
        dag.add_edge(Edge::new(
            "param_source_tools_makegen_makegen_path",
            "path",
            "prepare_write_makegen",
            "path",
        ));
        dag
    }
}
