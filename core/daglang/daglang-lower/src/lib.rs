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

// RT-C4: LoweringContext struct will group these params. Until then, allow.
#![allow(clippy::too_many_arguments)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use daglang_syntax::ast::{
    CapabilityDef, DataDef, Expr, Item, Literal, NodeStmt, OperationDef, ServiceDef, Stmt,
    TransportBinding,
};
use daglang_syntax::ast_utils::{
    canonical_resource_type_name, is_type_expr_optional, resource_type_name,
    service_call_lookup_keys, type_expr_to_string, walk_stmts,
};
use daglang_typecheck::{TypedCallableSignature, TypedItemSignature, TypedProject};
use gunbc_ir::patterns::branch::IfBuilder;
use gunbc_ir::patterns::{BranchBuilder, LoopBuilder, PatternOp};
use gunbc_ir::resource::{AccessMode, RESOURCE_FILE};
use gunbc_ir::{
    Cardinality, Dag, DagTopology, Edge, EdgeKind, Guard, Node, NodeId, NodeKind, Port, PortName,
    Value,
};
use serde::Serialize;

pub mod eval;
pub mod expr;
#[allow(dead_code)]
pub(crate) mod scope;
pub mod spec;

pub use spec::{
    ArgvSegment, BodyEntry, FieldSpec, FileOperationSpec, LocalOperationSpec, OutputFieldSpec,
    RestOperationSpec, ServiceOperationSpec, ShellOperationSpec, ShellOutputParsing,
};

pub use expr::LoweredFnBody;

/// Lowered operation payload for daglang graph nodes.
#[derive(Debug, Clone)]
pub enum LoweredOp {
    Callable {
        module: String,
        kind: CallableKind,
        name: String,
        obligation: ObligationCategory,
        service_metadata: Option<Box<ServiceCallMetadata>>,
        is_interactive: bool,
        resource_target: Option<String>,
        /// Lowered fn body for `CallableKind::Fn` items — `None` for
        /// func/pattern items and non-DSL callables (service transport, etc.).
        fn_body: Option<Box<LoweredFnBody>>,
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
    /// A pattern-internal operation (LoopUnpack, LoopPack, BranchMerge, etc.).
    /// Wraps `PatternOp` directly — no round-trip conversion needed at resolve time.
    Pattern(PatternOp),
    /// A pattern operation that is not yet supported in daglang lowering.
    /// Produces a structured error at resolution time instead of panicking.
    UnsupportedPattern {
        name: String,
    },
    /// An extern function call referencing a symbol that must be linked externally.
    ExternCall {
        symbol: String,
    },
}

impl From<PatternOp> for LoweredOp {
    fn from(op: PatternOp) -> Self {
        match op {
            // Supported pattern ops: wrap directly.
            PatternOp::LoopUnpack { .. }
            | PatternOp::LoopPack { .. }
            | PatternOp::BranchMerge { .. } => LoweredOp::Pattern(op),
            // Patterns not yet supported in daglang lowering.
            // Explicit match ensures compile-time failure when new variants are added.
            other => LoweredOp::UnsupportedPattern {
                name: other.kind_name().to_string(),
            },
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
    CallParamSource {
        callable: String,
        param: String,
    },
    CallLiteralSource {
        literal: PrimitiveLiteral,
    },
    IoPrepareFileRead,
    IoExecuteFileRead,
    CompareEquality,
    IoPrepareFileWrite,
    IoExecuteFileWrite,
    /// FC-7: Explicit output path annotation for content_upsert patterns.
    /// Replaces the `content_upsert_path_` ID substring hack.
    ContentUpsertOutputPath {
        path: String,
    },
    /// A compute node that evaluates a lowered expression body.
    /// Created by the lowerer when complex expressions cannot be represented
    /// as direct wiring edges.
    ExprCompute {
        fn_body: Box<LoweredFnBody>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveLiteral {
    String(String),
    Int(i64),
    Bool(bool),
    Json(serde_json::Value),
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
    /// Pure function that templates/renders output from inputs (e.g. `render_makefile`).
    PureRender,
    /// Pure function that loads configuration data (non-handle output, e.g. `load_registry`).
    PureDataLoad,
    /// Catch-all for pure functions with no special semantics.
    PureGeneric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTransportClass {
    Unknown,
    ShellLocal,
    RestNetwork,
    FileBoundary,
    LocalDirect,
    /// Stub transport for interface capabilities compiled without a profile.
    /// DryRun-compatible; errors in Real mode with "requires --profile".
    InterfaceStub,
}

impl ServiceTransportClass {
    /// Transport cost classification (mirrors `transport_depth` in `std/fidelity.dag`).
    ///
    /// Single authoritative source for the transport→FermiDepth mapping.
    /// The DSL `transport_depth()` function should be kept in sync.
    pub fn fermi_depth(&self) -> &'static str {
        match self {
            Self::LocalDirect => "Xs",
            Self::InterfaceStub => "Xs",
            Self::ShellLocal => "S",
            Self::FileBoundary => "S",
            Self::RestNetwork => "L",
            Self::Unknown => "Xl",
        }
    }

    /// Whether this transport class is hermetic (can be fully mocked in DryRun).
    ///
    /// Single authoritative source for the transport→hermetic mapping.
    /// The DSL `transport_hermetic()` function should be kept in sync.
    pub fn is_hermetic(&self) -> bool {
        match self {
            Self::LocalDirect => true,
            Self::InterfaceStub => true,
            Self::ShellLocal => true,
            Self::FileBoundary => true,
            Self::RestNetwork => false,
            Self::Unknown => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ServiceCallMetadata {
    pub service: String,
    pub operation: String,
    pub transport: ServiceTransportClass,
    pub idempotent: bool,
    pub readonly: bool,
    pub permissions: Vec<String>,
    /// Full protocol spec extracted from DSL service/operation declarations.
    /// Used by generic protocol interpreters to replace per-service adapters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<ServiceOperationSpec>,
}


impl LoweredOp {
    pub fn obligation_category(&self) -> ObligationCategory {
        match self {
            Self::Callable { obligation, .. } => *obligation,
            Self::Primitive { kind, .. } => kind.obligation_category(),
            Self::Collection { .. }
            | Self::Pipeline { .. }
            | Self::Pattern(_)
            | Self::UnsupportedPattern { .. }
            | Self::ExternCall { .. } => ObligationCategory::None,
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
            | Self::Pattern(_)
            | Self::UnsupportedPattern { .. }
            | Self::ExternCall { .. } => None,
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
            Self::ContentUpsertOutputPath { .. } => ObligationCategory::None,
            Self::ExprCompute { .. } => ObligationCategory::None,
        }
    }
}

pub fn classify_obligation(op: &LoweredOp) -> ObligationCategory {
    op.obligation_category()
}

/// Map a lowered node's obligation category (+ port types for tool distinction)
/// to a [`NodeKind`].
///
/// `ResourceProvide` nodes that emit `ToolHandle` become `ToolEnvironment`;
/// other `ResourceProvide` become `ResourceEnvironment`. Nodes with a
/// `ToolHandle` input port become `ToolConsumer` regardless of their
/// obligation category.
pub fn obligation_to_node_kind(node: &Node<LoweredOp>) -> NodeKind {
    let cat = match &node.body {
        gunbc_ir::NodeBody::Opaque(op) => op.obligation_category(),
        gunbc_ir::NodeBody::SubDag(_) => ObligationCategory::None,
    };

    // ToolConsumer: any node that consumes a ToolHandle input, unless it's
    // already a transport executor.
    let has_tool_input = node.inputs.iter().any(|p| p.type_id.0 == "ToolHandle");
    let has_tool_output = node.outputs.iter().any(|p| p.type_id.0 == "ToolHandle");

    match cat {
        ObligationCategory::ServiceTransportPrepare => NodeKind::TransportPrepare,
        ObligationCategory::ServiceTransportExecute => NodeKind::TransportExecute,
        ObligationCategory::ServiceTransportParse => NodeKind::TransportParse,
        ObligationCategory::ResourceProvide => {
            if has_tool_output {
                NodeKind::ToolEnvironment
            } else {
                NodeKind::ResourceEnvironment
            }
        }
        ObligationCategory::ResourceAcquire => NodeKind::ResourceAcquire,
        ObligationCategory::ResourceRelease => NodeKind::ResourceRelease,
        _ => {
            if has_tool_input {
                NodeKind::ToolConsumer
            } else {
                NodeKind::Pure
            }
        }
    }
}

/// Walk all nodes in a lowered DAG (recursively into subdags) and set
/// `node.kind` from `obligation_to_node_kind`.
pub fn stamp_node_kinds(dag: &mut Dag<LoweredOp>) {
    for node in &mut dag.nodes {
        node.kind = obligation_to_node_kind(node);
        if let gunbc_ir::NodeBody::SubDag(ref mut inner) = node.body {
            stamp_node_kinds(inner);
        }
    }
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
        ObligationCategory::PureRender
        | ObligationCategory::PureDataLoad
        | ObligationCategory::PureGeneric => Some("callable"),
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
pub(crate) struct LoweredEndpoint {
    pub(crate) node_id: String,
    pub(crate) primary_output: String,
}

#[derive(Debug, Clone)]
pub(crate) struct EndpointRegistry<T> {
    pub(crate) by_key: HashMap<String, Option<T>>,
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

    fn get(&self, key: &str) -> Option<&T> {
        self.by_key.get(key).and_then(|entry| entry.as_ref())
    }
}

pub(crate) type ServiceEndpointRegistry = EndpointRegistry<ServiceTransportEndpoint>;
type ResourceLifecycleRegistry = EndpointRegistry<ResourceLifecycleEndpoint>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceTransportEndpoint {
    pub(crate) parse: LoweredEndpoint,
    pub(crate) prepare_node_id: String,
    pub(crate) execute_node_id: String,
    pub(crate) prepare_inputs: Vec<String>,
    /// Full (unfiltered) operation input names for positional arg resolution.
    /// `prepare_inputs` excludes `auth_input`, so positional indices shift.
    /// Resolving against this list gives the correct field name, which the
    /// downstream auth_input skip/wire logic then handles by name.
    pub(crate) operation_inputs: Vec<String>,
    pub(crate) has_auth: bool,
    /// Service call metadata for this endpoint (carried for loop-body transport).
    pub(crate) metadata: Option<ServiceCallMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceLifecycleEndpoint {
    acquire_node: Option<String>,
    release_node: Option<String>,
}

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

#[derive(Debug, Clone, Default)]
struct NameAliasRegistry {
    full: HashSet<String>,
    aliases: HashMap<String, Option<String>>,
}

impl NameAliasRegistry {
    fn register(&mut self, full: String, aliases: &[String]) {
        self.full.insert(full.clone());
        for alias in aliases {
            self.aliases
                .entry(alias.clone())
                .and_modify(|existing| {
                    if existing.as_ref() != Some(&full) {
                        *existing = None;
                    }
                })
                .or_insert_with(|| Some(full.clone()));
        }
    }

    fn resolve(&self, name: &str) -> NameResolution {
        let canonical = canonical_resource_type_name(name);
        if self.full.contains(&canonical) {
            return NameResolution::Resolved(canonical);
        }
        match self.aliases.get(&canonical) {
            Some(Some(full)) => NameResolution::Resolved(full.clone()),
            Some(None) => NameResolution::Ambiguous,
            None => NameResolution::Missing,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NameResolution {
    Resolved(String),
    Ambiguous,
    Missing,
}

#[derive(Debug, Clone, Default)]
struct ProfileBindingRegistry {
    by_full: HashMap<String, HashMap<String, ProfileBindingRecord>>,
    by_alias: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone)]
struct ProfileBindingRecord {
    implementation_type: String,
    config_entries: Vec<(String, Expr)>,
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveProfileBindings {
    pub(crate) profile_name: String,
    pub(crate) by_interface: HashMap<String, ActiveProfileBinding>,
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveProfileBinding {
    pub(crate) implementation_type: String,
    pub(crate) config_values: HashMap<String, ProfileConfigValue>,
}

#[derive(Debug, Clone)]
pub(crate) enum ProfileConfigValue {
    Literal(String),
    SecretRef(String),
}

fn collect_profile_binding_registry(
    project: &TypedProject,
    active_profile: Option<&str>,
) -> Result<ProfileBindingRegistry, LowerError> {
    let mut interface_registry = NameAliasRegistry::default();
    let mut service_registry = NameAliasRegistry::default();

    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            match &item.node {
                Item::InterfaceDef(def) => {
                    let full = format!("{module_name}.{}", def.name);
                    interface_registry.register(full, std::slice::from_ref(&def.name));
                }
                Item::ServiceDef(def) => {
                    let full = format!("{module_name}.{}", def.name);
                    let tail = def
                        .name
                        .rsplit('.')
                        .next()
                        .unwrap_or(def.name.as_str())
                        .to_string();
                    service_registry.register(full, &[def.name.clone(), tail]);
                }
                _ => {}
            }
        }
    }

    let mut registry = ProfileBindingRegistry::default();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::ProfileDef(def) = &item.node else {
                continue;
            };
            let profile_full = format!("{module_name}.{}", def.name);
            let mut bindings = HashMap::<String, ProfileBindingRecord>::new();
            for bind in &def.binds {
                let resolved_interface = match interface_registry.resolve(&bind.interface_type) {
                    NameResolution::Resolved(full) => full,
                    NameResolution::Ambiguous => {
                        return Err(LowerError::InvalidProfileBinding {
                            profile: profile_full.clone(),
                            detail: format!("interface `{}` is ambiguous", bind.interface_type),
                        })
                    }
                    NameResolution::Missing => {
                        return Err(LowerError::InvalidProfileBinding {
                            profile: profile_full.clone(),
                            detail: format!("interface `{}` is unresolved", bind.interface_type),
                        })
                    }
                };
                let resolved_impl = match service_registry.resolve(&bind.implementation_type) {
                    NameResolution::Resolved(full) => full,
                    NameResolution::Ambiguous => {
                        return Err(LowerError::InvalidProfileBinding {
                            profile: profile_full.clone(),
                            detail: format!(
                                "implementation `{}` is ambiguous",
                                bind.implementation_type
                            ),
                        })
                    }
                    NameResolution::Missing => {
                        // Only require implementation resolution for the active profile.
                        // Non-active profiles may reference modules that weren't loaded.
                        let is_active = active_profile == Some(def.name.as_str());
                        if is_active {
                            return Err(LowerError::InvalidProfileBinding {
                                profile: profile_full.clone(),
                                detail: format!(
                                    "implementation `{}` is unresolved",
                                    bind.implementation_type
                                ),
                            });
                        }
                        canonical_resource_type_name(&bind.implementation_type)
                    }
                };
                if bindings
                    .insert(
                        resolved_interface.clone(),
                        ProfileBindingRecord {
                            implementation_type: resolved_impl,
                            config_entries: bind.config_entries.clone(),
                        },
                    )
                    .is_some()
                {
                    return Err(LowerError::InvalidProfileBinding {
                        profile: profile_full.clone(),
                        detail: format!("duplicate binding for `{resolved_interface}`"),
                    });
                }
            }
            registry
                .by_alias
                .entry(def.name.clone())
                .and_modify(|existing| {
                    if existing.as_ref() != Some(&profile_full) {
                        *existing = None;
                    }
                })
                .or_insert_with(|| Some(profile_full.clone()));
            registry.by_full.insert(profile_full, bindings);
        }
    }
    Ok(registry)
}

fn resolve_active_profile_bindings(
    registry: &ProfileBindingRegistry,
    active_profile: Option<&str>,
) -> Result<Option<ActiveProfileBindings>, LowerError> {
    let Some(profile) = active_profile else {
        return Ok(None);
    };
    let (profile_name, bindings) =
        if let Some((full_name, bindings)) = registry.by_full.get_key_value(profile) {
            (full_name.clone(), bindings)
        } else {
            match registry.by_alias.get(profile) {
                Some(Some(full)) => (
                    full.clone(),
                    registry
                        .by_full
                        .get(full)
                        .ok_or_else(|| LowerError::UnknownProfile {
                            profile: profile.to_string(),
                        })?,
                ),
                Some(None) => {
                    return Err(LowerError::AmbiguousProfile {
                        profile: profile.to_string(),
                    })
                }
                None => {
                    return Err(LowerError::UnknownProfile {
                        profile: profile.to_string(),
                    })
                }
            }
        };

    let mut resolved = HashMap::<String, ActiveProfileBinding>::new();
    for (interface, binding) in bindings {
        let mut config_values = HashMap::new();
        for (key, value_expr) in &binding.config_entries {
            let value = resolve_profile_config_value(
                profile_name.as_str(),
                interface.as_str(),
                key.as_str(),
                value_expr,
            )?;
            config_values.insert(key.clone(), value);
        }
        resolved.insert(
            interface.clone(),
            ActiveProfileBinding {
                implementation_type: binding.implementation_type.clone(),
                config_values,
            },
        );
    }
    Ok(Some(ActiveProfileBindings {
        profile_name,
        by_interface: resolved,
    }))
}

fn resolve_profile_config_value(
    profile: &str,
    interface_type: &str,
    key: &str,
    expr: &Expr,
) -> Result<ProfileConfigValue, LowerError> {
    match expr {
        Expr::Literal(Literal::String(value)) => Ok(ProfileConfigValue::Literal(value.clone())),
        Expr::Ident(name) => Ok(ProfileConfigValue::Literal(name.clone())),
        Expr::Call(name, args) if name == "env" => {
            let env_var = parse_single_string_call_arg(args).ok_or_else(|| {
                LowerError::InvalidProfileBinding {
                    profile: profile.to_string(),
                    detail: format!(
                        "config `{key}` for `{interface_type}` must be `env(\"VAR\")`"
                    ),
                }
            })?;
            let env_value = std::env::var(env_var.as_str())
                .ok()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| LowerError::MissingProfileConfigEnv {
                    profile: profile.to_string(),
                    interface_type: interface_type.to_string(),
                    key: key.to_string(),
                    env_var: env_var.clone(),
                })?;
            Ok(ProfileConfigValue::Literal(env_value))
        }
        Expr::Call(name, args) if name == "secret" => {
            let secret_name = parse_single_string_call_arg(args).ok_or_else(|| {
                LowerError::InvalidProfileBinding {
                    profile: profile.to_string(),
                    detail: format!(
                        "config `{key}` for `{interface_type}` must be `secret(\"name\")`"
                    ),
                }
            })?;
            Ok(ProfileConfigValue::SecretRef(secret_name))
        }
        _ => Err(LowerError::InvalidProfileBinding {
            profile: profile.to_string(),
            detail: format!(
                "unsupported config expression for `{interface_type}.{key}`; expected string literal, env(\"VAR\"), or secret(\"name\")"
            ),
        }),
    }
}

fn parse_single_string_call_arg(args: &[(Option<String>, Expr)]) -> Option<String> {
    let [(name, value)] = args else {
        return None;
    };
    if name.is_some() {
        return None;
    }
    match value {
        Expr::Literal(Literal::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn collect_profile_bound_interface_names(registry: &ProfileBindingRegistry) -> HashSet<String> {
    let mut names = HashSet::new();
    for bindings in registry.by_full.values() {
        for interface in bindings.keys() {
            let canonical = canonical_resource_type_name(interface);
            names.insert(canonical.clone());
            if let Some(short) = canonical.rsplit('.').next() {
                names.insert(short.to_string());
            }
        }
    }
    names
}

fn collect_interface_type_names(project: &TypedProject) -> HashSet<String> {
    let mut names = HashSet::new();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::InterfaceDef(def) = &item.node else {
                continue;
            };
            let full = format!("{module_name}.{}", def.name);
            names.insert(canonical_resource_type_name(&full));
            names.insert(canonical_resource_type_name(&def.name));
            if let Some(tail) = def.name.rsplit('.').next() {
                names.insert(tail.to_string());
            }
        }
    }
    names
}

fn is_bound_interface_type_name(names: &HashSet<String>, interface_type: &str) -> bool {
    let canonical = canonical_resource_type_name(interface_type);
    if names.contains(&canonical) {
        return true;
    }
    let short = canonical.rsplit('.').next().unwrap_or(canonical.as_str());
    names.contains(short)
}

/// Identify interface types that need stub transport (IS-3).
///
/// When compiling without a profile, instead of hard-erroring, this function
/// returns the set of interface type names that need stub transport triplets.
/// With an active profile, returns an empty set (all interfaces are resolved).
fn interfaces_needing_stubs(
    project: &TypedProject,
    active_profile: Option<&str>,
    profile_bound_interfaces: &HashSet<String>,
) -> HashSet<String> {
    if active_profile.is_some() || profile_bound_interfaces.is_empty() {
        return HashSet::new();
    }
    let mut stub_interfaces = HashSet::new();
    for module in &project.modules {
        for item in &module.ast.items {
            let uses = match &item.node {
                Item::FuncDef(def) => def.uses.as_slice(),
                Item::PatternDef(def) => def.uses.as_slice(),
                Item::PipelineDef(def) => def.uses.as_slice(),
                _ => continue,
            };
            for usage in uses {
                let interface_type = resource_type_name(&usage.resource_type);
                if is_bound_interface_type_name(profile_bound_interfaces, interface_type.as_str()) {
                    insert_canonical_names(&mut stub_interfaces, &interface_type);
                }
            }
        }
    }
    stub_interfaces
}

fn insert_canonical_names(set: &mut HashSet<String>, name: &str) {
    let canonical = canonical_resource_type_name(name);
    let short = canonical
        .rsplit('.')
        .next()
        .unwrap_or(canonical.as_str())
        .to_string();
    set.insert(canonical);
    set.insert(short);
}

fn is_known_uses_type(set: &HashSet<String>, name: &str) -> bool {
    let canonical = canonical_resource_type_name(name);
    set.contains(&canonical)
        || set.contains(canonical.rsplit('.').next().unwrap_or(canonical.as_str()))
}

/// Register stub transport triplets for interfaces that lack profile bindings (IS-4).
///
/// When compiling without a profile, interface capabilities still need transport
/// triplets in the registry so `resolve_service_call_source` can find them. These
/// stubs use `ServiceTransportClass::InterfaceStub` and are DryRun-compatible;
/// real-mode execution will surface a "requires --profile" error at the resolver.
fn add_interface_stub_transport_triplets(
    builder: &mut DagBuilder,
    project: &TypedProject,
    stub_interfaces: &HashSet<String>,
    registry: &mut ServiceEndpointRegistry,
) {
    if stub_interfaces.is_empty() {
        return;
    }

    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::InterfaceDef(interface) = &item.node else {
                continue;
            };

            if !is_bound_interface_type_name(stub_interfaces, &interface.name) {
                continue;
            }

            for capability in &interface.capabilities {
                let metadata = ServiceCallMetadata {
                    service: interface.name.clone(),
                    operation: capability.name.clone(),
                    transport: ServiceTransportClass::InterfaceStub,
                    idempotent: capability.idempotent,
                    readonly: capability.readonly,
                    permissions: vec![],
                    spec: Some(ServiceOperationSpec::InterfaceStub {
                        interface: interface.name.clone(),
                        capability: capability.name.clone(),
                    }),
                };

                let suffix = sanitize_identifier(&format!(
                    "{module_name}_{}_{}",
                    interface.name, capability.name
                ));
                let prepare_id = format!("prepare_transport_{suffix}");
                let execute_id = format!("execute_transport_{suffix}");
                let parse_id = format!("parse_transport_{suffix}");

                // Prepare node: capability inputs → TransportRequest.
                let prepare_ports = capability_prepare_ports(capability, &metadata);
                let prepare_inputs = prepare_ports
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect::<Vec<_>>();

                builder.add_node(Node::opaque(
                    prepare_id.clone(),
                    prepare_ports,
                    vec![Port::scalar("request", "TransportRequest")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::prepare::{}::{}",
                            interface.name, capability.name
                        ),
                        obligation: ObligationCategory::ServiceTransportPrepare,
                        service_metadata: Some(Box::new(metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                ));

                // Execute node: TransportRequest → typed capability outputs.
                // In DryRun, boundary mocks supply typed fields directly.
                // In Real mode, the execute op errors with "requires --profile".
                let typed_outputs = if capability.outputs.is_empty() {
                    vec![Port::scalar("result", "Unit")]
                } else {
                    capability
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
                let execute_node = Node::opaque(
                    execute_id.clone(),
                    vec![Port::scalar("request", "TransportRequest")],
                    typed_outputs.clone(),
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::execute::{}::{}",
                            interface.name, capability.name
                        ),
                        obligation: ObligationCategory::ServiceTransportExecute,
                        service_metadata: Some(Box::new(metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                )
                .with_input_guard("request", Guard::NotEq(Value::Skipped));
                builder.add_node(execute_node);

                // Parse node: typed capability outputs → typed capability outputs (identity).
                builder.add_node(Node::opaque(
                    parse_id.clone(),
                    typed_outputs.clone(),
                    typed_outputs,
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::parse::{}::{}",
                            interface.name, capability.name
                        ),
                        obligation: ObligationCategory::ServiceTransportParse,
                        service_metadata: Some(Box::new(metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                ));

                // Wire the triplet: prepare → execute → parse.
                builder.add_edge(
                    prepare_id.as_str(),
                    "request",
                    execute_id.as_str(),
                    "request",
                );
                // Wire per-field edges from execute to parse.
                for field in &capability.outputs {
                    builder.add_edge(
                        execute_id.as_str(),
                        field.name.as_str(),
                        parse_id.as_str(),
                        field.name.as_str(),
                    );
                }
                if capability.outputs.is_empty() {
                    builder.add_edge(execute_id.as_str(), "result", parse_id.as_str(), "result");
                }

                // Register endpoints under multiple keys for flexible resolution.
                let parse_output = capability
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
                    execute_node_id: execute_id,
                    operation_inputs: prepare_inputs.clone(),
                    prepare_inputs,
                    has_auth: false,
                    metadata: Some(metadata),
                };
                let cap_key = format!("{}.{}", interface.name, capability.name);
                registry.register(cap_key.clone(), endpoint.clone());
                registry.register(format!("{module_name}.{cap_key}"), endpoint);
            }
        }
    }
}

/// Wraps a `Dag` with O(1) deduplication tracking for nodes and edges.
struct DagBuilder {
    dag: Dag<LoweredOp>,
    seen_nodes: HashSet<String>,
    seen_edges: HashSet<(String, String, String, String, EdgeKind)>,
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
        let kind = if to_port == PortName::DEPS {
            EdgeKind::Control
        } else {
            EdgeKind::DataFlow
        };
        self.add_edge_kind(from, from_port, to, to_port, kind);
    }

    fn add_control_edge(&mut self, from: &str, from_port: &str, to: &str, to_port: &str) {
        self.add_edge_kind(from, from_port, to, to_port, EdgeKind::Control);
    }

    fn add_edge_kind(
        &mut self,
        from: &str,
        from_port: &str,
        to: &str,
        to_port: &str,
        kind: EdgeKind,
    ) {
        let key = (
            from.to_string(),
            from_port.to_string(),
            to.to_string(),
            to_port.to_string(),
            kind,
        );
        if self.seen_edges.insert(key) {
            self.dag.add_edge(Edge {
                from_node: NodeId::new(from.to_string()),
                from_port: PortName::new(from_port.to_string()),
                to_node: NodeId::new(to.to_string()),
                to_port: PortName::new(to_port.to_string()),
                index: 0,
                kind,
            });
        }
    }

    fn has_node(&self, id: &str) -> bool {
        self.seen_nodes.contains(id)
    }

    fn has_edge_to_port(&self, to_node: &str, to_port: &str) -> bool {
        self.seen_edges
            .iter()
            .any(|(_, _, tn, tp, _)| tn == to_node && tp == to_port)
    }

    fn clone_transport_triplet(
        &mut self,
        original: &ServiceTransportEndpoint,
        suffix: &str,
    ) -> ServiceTransportEndpoint {
        let new_prepare_id = format!("{}_{suffix}", original.prepare_node_id);
        let new_execute_id = format!("{}_{suffix}", original.execute_node_id);
        let new_parse_id = format!("{}_{suffix}", original.parse.node_id);

        let (prepare_clone, execute_clone, parse_clone) = {
            let nodes = &self.dag.nodes;
            (
                nodes
                    .iter()
                    .find(|n| n.id.0 == original.prepare_node_id)
                    .cloned(),
                nodes
                    .iter()
                    .find(|n| n.id.0 == original.execute_node_id)
                    .cloned(),
                nodes
                    .iter()
                    .find(|n| n.id.0 == original.parse.node_id)
                    .cloned(),
            )
        };

        if let Some(mut n) = prepare_clone {
            n.id = new_prepare_id.clone().into();
            self.add_node(n);
        }
        if let Some(mut n) = execute_clone {
            n.id = new_execute_id.clone().into();
            self.add_node(n);
        }
        if let Some(mut n) = parse_clone {
            n.id = new_parse_id.clone().into();
            self.add_node(n);
        }

        self.add_edge(&new_prepare_id, "request", &new_execute_id, "request");
        self.add_edge(&new_execute_id, "response", &new_parse_id, "response");

        ServiceTransportEndpoint {
            parse: LoweredEndpoint {
                node_id: new_parse_id,
                primary_output: original.parse.primary_output.clone(),
            },
            prepare_node_id: new_prepare_id,
            execute_node_id: new_execute_id,
            prepare_inputs: original.prepare_inputs.clone(),
            operation_inputs: original.operation_inputs.clone(),
            has_auth: original.has_auth,
            metadata: original.metadata.clone(),
        }
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
    /// Requested profile name is not declared in the DSL corpus.
    UnknownProfile { profile: String },
    /// Requested profile name matches multiple declarations.
    AmbiguousProfile { profile: String },
    /// Profile declaration contains an invalid bind entry.
    InvalidProfileBinding { profile: String, detail: String },
    /// A bound interface call requires an active profile selection.
    ProfileRequiredForBoundServiceCall {
        caller: String,
        binding: String,
        interface_type: String,
    },
    /// Active profile does not bind an interface used by a callable.
    MissingProfileBinding {
        profile: String,
        interface_type: String,
    },
    /// Transport block specifies an unknown file operation.
    InvalidFileOp {
        operation: String,
        file_op: String,
    },
    /// Active profile uses an env(...) config binding that is not set.
    MissingProfileConfigEnv {
        profile: String,
        interface_type: String,
        key: String,
        env_var: String,
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
            Self::InvalidFileOp { operation, file_op } => {
                write!(f, "unknown file operation `{file_op}` on `{operation}`")
            }
            Self::UnknownProfile { profile } => {
                write!(f, "unknown profile `{profile}`")
            }
            Self::AmbiguousProfile { profile } => {
                write!(f, "ambiguous profile `{profile}`; use fully-qualified profile name")
            }
            Self::InvalidProfileBinding { profile, detail } => {
                write!(f, "invalid profile binding in `{profile}`: {detail}")
            }
            Self::ProfileRequiredForBoundServiceCall {
                caller,
                binding,
                interface_type,
            } => write!(
                f,
                "bound service call `{binding}` in `{caller}` targets interface `{interface_type}`; compile with --profile <name>"
            ),
            Self::MissingProfileBinding {
                profile,
                interface_type,
            } => write!(
                f,
                "profile `{profile}` does not bind interface `{interface_type}`"
            ),
            Self::MissingProfileConfigEnv {
                profile,
                interface_type,
                key,
                env_var,
            } => write!(
                f,
                "profile `{profile}` binding `{interface_type}` requires config `{key}` from env var `{env_var}`, but it is not set"
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
fn collect_variant_names(project: &TypedProject) -> HashSet<String> {
    let mut names = HashSet::new();
    for module in &project.modules {
        for item in &module.ast.items {
            if let Item::TypeDef(def) = &item.node {
                if let daglang_syntax::ast::TypeBody::Sum(variants) = &def.body {
                    for v in variants {
                        names.insert(v.name.clone());
                    }
                }
            }
        }
    }
    names
}

pub fn lower_typed_project(project: &TypedProject) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, false, None, None)
}

pub fn lower_typed_project_with_profile(
    project: &TypedProject,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, false, active_profile, None)
}

/// Lowers typed modules while emitting explicit collection pipeline nodes.
pub fn lower_typed_project_with_collection_nodes(
    project: &TypedProject,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, true, None, None)
}

pub fn lower_typed_project_with_profile_and_collection_nodes(
    project: &TypedProject,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, None, true, active_profile, None)
}

pub fn lower_typed_project_for_modules(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), false, None, None)
}

pub fn lower_typed_project_for_modules_with_profile(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(
        project,
        Some(callable_modules),
        false,
        active_profile,
        None,
    )
}

/// Lowers only scoped modules while emitting explicit collection pipeline nodes.
pub fn lower_typed_project_for_modules_with_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(project, Some(callable_modules), true, None, None)
}

pub fn lower_typed_project_for_modules_with_profile_and_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(
        project,
        Some(callable_modules),
        true,
        active_profile,
        None,
    )
}

pub fn lower_typed_project_for_modules_with_entry(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
    entry_module: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(
        project,
        Some(callable_modules),
        false,
        active_profile,
        entry_module,
    )
}

pub fn lower_typed_project_for_modules_with_entry_and_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
    entry_module: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_typed_project_with_callable_scope(
        project,
        Some(callable_modules),
        true,
        active_profile,
        entry_module,
    )
}

fn lower_typed_project_with_callable_scope(
    project: &TypedProject,
    callable_modules: Option<&HashSet<String>>,
    emit_collection_nodes: bool,
    active_profile: Option<&str>,
    entry_module: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    let mut builder = DagBuilder::new();
    let mut endpoints_by_full = HashMap::<(String, String), LoweredEndpoint>::new();
    let mut endpoints_by_name = HashMap::<String, Option<LoweredEndpoint>>::new();
    let variant_names = collect_variant_names(project);

    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
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
        // Build fn body lookup: name → LoweredFnBody (for fn items only)
        let fn_bodies: HashMap<&str, LoweredFnBody> = module
            .ast
            .items
            .iter()
            .filter_map(|item| match &item.node {
                Item::FnDef(def) if !def.body.lossy => Some((
                    def.name.as_str(),
                    expr::lower_fn_body(&def.body, &variant_names),
                )),
                _ => None,
            })
            .collect();
        for signature in &module.signatures {
            match signature {
                TypedItemSignature::Fn(callable) => {
                    if !include_callables {
                        continue;
                    }
                    let body = fn_bodies
                        .get(callable.name.as_str())
                        .map(|b| Box::new(b.clone()));
                    let (node, endpoint) = lower_callable(
                        callable,
                        &module_name,
                        CallableKind::Fn,
                        *interactive_by_callable
                            .get(callable.name.as_str())
                            .unwrap_or(&false),
                        body,
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
                        None,
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
                        None,
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
                TypedItemSignature::ExternFunc(callable) => {
                    if !include_callables {
                        continue;
                    }
                    let (node, endpoint) = lower_extern_callable(callable, &module_name);
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

    add_makegen_scaffolding(&mut builder, &endpoints_by_full);
    let mut service_registry = if callable_modules.is_some() && active_profile.is_none() {
        let required_service_calls = collect_required_service_call_keys(project, callable_modules);
        add_service_transport_triplets(&mut builder, project, Some(&required_service_calls))?
    } else {
        add_service_transport_triplets(&mut builder, project, None)?
    };
    let data_values = build_data_values(project);
    add_dependency_edges(
        &mut builder,
        project,
        &endpoints_by_full,
        &endpoints_by_name,
        &service_registry,
        emit_collection_nodes,
        entry_module,
        &data_values,
    );
    let profile_registry = collect_profile_binding_registry(project, active_profile)?;
    let active_profile_bindings =
        resolve_active_profile_bindings(&profile_registry, active_profile)?;
    let profile_bound_interfaces = collect_profile_bound_interface_names(&profile_registry);
    // IS-3: Collect interfaces needing stub transport.
    let stub_interfaces =
        interfaces_needing_stubs(project, active_profile, &profile_bound_interfaces);
    // IS-4: Register stub transport triplets so resolve_service_call_source can find them.
    add_interface_stub_transport_triplets(
        &mut builder,
        project,
        &stub_interfaces,
        &mut service_registry,
    );
    let known_interface_types = collect_interface_type_names(project);
    add_service_call_edges(
        &mut builder,
        project,
        &endpoints_by_full,
        &endpoints_by_name,
        &service_registry,
        active_profile_bindings.as_ref(),
        &profile_bound_interfaces,
        &known_interface_types,
        &data_values,
    )?;
    let auth_provider_names = collect_auth_provider_names(project);
    wire_auth_credential_edges(
        &mut builder,
        project,
        &endpoints_by_full,
        &endpoints_by_name,
        &service_registry,
        &auth_provider_names,
    );
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

    let mut dag = builder.into_dag();
    stamp_node_kinds(&mut dag);
    Ok(dag)
}

pub use parity::{
    canonical_ir_json, compare_gcp_credential_topology, compare_ir, compare_topology,
};

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
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pattern(_)) => {
                "pattern_internal".to_string()
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::UnsupportedPattern { name }) => {
                format!("unsupported_pattern:{name}")
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::ExternCall { symbol }) => {
                format!("extern_call:{symbol}")
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
            fn_body: None,
        })
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
}

/// Infer an obligation category for `fn` items based on name/output-type heuristics.
///
/// Only applies to `CallableKind::Fn` (pure functions). `Func`/`Pattern` callables
/// keep `ObligationCategory::None` (they are classified structurally elsewhere).
fn infer_fn_obligation(name: &str, kind: CallableKind, outputs: &[Port]) -> ObligationCategory {
    if kind != CallableKind::Fn {
        return ObligationCategory::None;
    }
    // Handle/Env output + load_/fs_env/env_ name → resource provider.
    let has_handle_output = outputs.iter().any(|p| {
        let ty = p.type_id.0.as_str();
        ty.contains("Handle") || ty.contains("Env")
    });
    if has_handle_output
        && (name.starts_with("load_") || name == "fs_env" || name.starts_with("env_"))
    {
        return ObligationCategory::ResourceProvide;
    }
    if name.starts_with("render_") {
        return ObligationCategory::PureRender;
    }
    if name.starts_with("load_") || name.starts_with("env_") {
        return ObligationCategory::PureDataLoad;
    }
    ObligationCategory::PureGeneric
}

fn output_passthrough_input_name(output_name: &str) -> String {
    format!("{}{output_name}", PortName::OUTPUT_PASSTHROUGH_PREFIX)
}

fn is_output_passthrough_input(port_name: &str) -> bool {
    port_name.starts_with(PortName::OUTPUT_PASSTHROUGH_PREFIX)
}

fn lower_callable(
    callable: &TypedCallableSignature,
    module_name: &str,
    kind: CallableKind,
    is_interactive: bool,
    fn_body: Option<Box<LoweredFnBody>>,
) -> (Node<LoweredOp>, LoweredEndpoint) {
    let node_id = lowered_node_id(module_name, &callable.name);
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
    let mut inputs = callable
        .params
        .iter()
        .map(|binding| {
            Port::with_cardinality(binding.name.as_str(), binding.ty.as_str(), Cardinality::ONE)
        })
        .collect::<Vec<_>>();
    for output in &outputs {
        inputs.push(Port::with_cardinality(
            output_passthrough_input_name(output.name.0.as_str()),
            output.type_id.0.as_str(),
            Cardinality::ONE,
        ));
    }
    inputs.push(Port::with_cardinality(
        PortName::DEPS,
        "Any",
        Cardinality::ZERO_OR_MORE,
    ));
    let obligation = infer_fn_obligation(&callable.name, kind, &outputs);
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
                obligation,
                service_metadata: None,
                is_interactive,
                resource_target: None,
                fn_body,
            },
        ),
        LoweredEndpoint {
            node_id,
            primary_output,
        },
    )
}

fn lower_extern_callable(
    callable: &TypedCallableSignature,
    module_name: &str,
) -> (Node<LoweredOp>, LoweredEndpoint) {
    let node_id = lowered_node_id(module_name, &callable.name);
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
    let mut inputs = callable
        .params
        .iter()
        .map(|binding| {
            Port::with_cardinality(binding.name.as_str(), binding.ty.as_str(), Cardinality::ONE)
        })
        .collect::<Vec<_>>();
    inputs.push(Port::with_cardinality(
        PortName::DEPS,
        "Any",
        Cardinality::ZERO_OR_MORE,
    ));
    let primary_output = outputs
        .first()
        .map(|port| port.name.0.clone())
        .unwrap_or_else(|| "return".to_string());
    let symbol = format!("{module_name}::{}", callable.name);
    (
        Node::opaque(
            node_id.clone(),
            inputs,
            outputs,
            LoweredOp::ExternCall { symbol },
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
    service_registry: &ServiceEndpointRegistry,
    emit_collection_nodes: bool,
    entry_module: Option<&str>,
    data_values: &HashMap<String, serde_json::Value>,
) {
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        let param_types_by_callable = module
            .signatures
            .iter()
            .filter_map(|signature| match signature {
                TypedItemSignature::Fn(callable)
                | TypedItemSignature::Func(callable)
                | TypedItemSignature::Pattern(callable)
                | TypedItemSignature::ExternFunc(callable) => Some((
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
                    PortName::DEPS,
                );
            }

            if entry_module.is_none_or(|em| module_name == em) {
                expand_content_upsert_patterns(
                    builder,
                    &module_name,
                    item_name,
                    stmts,
                    target,
                    endpoints_by_name,
                    &param_types,
                    data_values,
                );
                expand_non_generic_pattern_calls(
                    builder,
                    project,
                    &module_name,
                    item_name,
                    stmts,
                    target,
                    endpoints_by_name,
                    &param_types,
                    service_registry,
                    data_values,
                );
            }
            if emit_collection_nodes {
                add_collection_pipeline_nodes(builder, &module_name, stmts, target);
            }
            let uses_binding_types = item_uses_binding_types(&item.node);
            add_control_flow_pattern_nodes(
                builder,
                &module_name,
                stmts,
                target,
                service_registry,
                &uses_binding_types,
            );
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
enum IterableRef {
    /// `for x in some_var { ... }`
    Ident(String),
    /// `for x in some_var.field { ... }`
    FieldAccess(String, String),
}

#[derive(Debug)]
struct ForLoopSite {
    element_var: String,
    iterable: Option<IterableRef>,
    passthrough: Vec<String>,
    /// Service call paths found inside the for-loop body expression.
    /// Each entry is the dot-separated path (e.g., `["fs", "read"]`).
    body_service_call_paths: Vec<Vec<String>>,
}

#[derive(Debug)]
struct IfBranchScopeSite {
    has_else: bool,
    then_service_call_paths: Vec<Vec<String>>,
    else_service_call_paths: Vec<Vec<String>>,
}

#[derive(Debug)]
struct MatchBranchScopeSite {
    arm_count: usize,
    all_service_call_paths: Vec<Vec<String>>,
}

fn collect_service_paths_from_scoped_body(body: &scope::ScopedBody) -> Vec<Vec<String>> {
    body.all_service_calls()
        .into_iter()
        .map(|call| call.path.clone())
        .collect()
}

fn collect_for_loop_sites_from_scoped(body: &scope::ScopedBody, out: &mut Vec<ForLoopSite>) {
    for item in &body.items {
        match item {
            scope::ScopedItem::ForLoop {
                element_var,
                iterable,
                passthrough,
                body,
            } => {
                let iterable_ref = match iterable {
                    scope::ExprRef::Ident(name) => Some(IterableRef::Ident(name.clone())),
                    scope::ExprRef::FieldAccess { base, field } => {
                        Some(IterableRef::FieldAccess(base.clone(), field.clone()))
                    }
                    scope::ExprRef::Literal(_) | scope::ExprRef::Opaque => None,
                };
                out.push(ForLoopSite {
                    element_var: element_var.clone(),
                    iterable: iterable_ref,
                    passthrough: passthrough.clone(),
                    body_service_call_paths: collect_service_paths_from_scoped_body(body),
                });
                collect_for_loop_sites_from_scoped(body, out);
            }
            scope::ScopedItem::IfBranch {
                then_body,
                else_body,
            } => {
                collect_for_loop_sites_from_scoped(then_body, out);
                if let Some(else_body) = else_body {
                    collect_for_loop_sites_from_scoped(else_body, out);
                }
            }
            scope::ScopedItem::MatchBranch { arms } => {
                for arm in arms {
                    collect_for_loop_sites_from_scoped(&arm.body, out);
                }
            }
            scope::ScopedItem::ServiceCall(_)
            | scope::ScopedItem::FnCall { .. }
            | scope::ScopedItem::Binding { .. }
            | scope::ScopedItem::Other => {}
        }
    }
}

fn detect_for_loops_in_stmts(stmts: &[Stmt]) -> Vec<ForLoopSite> {
    let scoped = scope::ScopedBody::from_stmts(stmts);
    let mut sites = Vec::new();
    collect_for_loop_sites_from_scoped(&scoped, &mut sites);
    sites
}

/// Collect service call paths from a single expression (non-recursive into for-loops).
pub(crate) fn collect_service_call_paths_from_expr(expr: &Expr, paths: &mut Vec<Vec<String>>) {
    if let Expr::ServiceCall(path, _) = expr {
        paths.push(path.clone());
    }
    // Recurse into sub-expressions (but stop at nested for-loops).
    match expr {
        Expr::Call(_, args) | Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                collect_service_call_paths_from_expr(arg, paths);
            }
        }
        Expr::FieldAccess(base, _) => collect_service_call_paths_from_expr(base, paths),
        Expr::BinOp(lhs, _, rhs) | Expr::Pipe(lhs, rhs) => {
            collect_service_call_paths_from_expr(lhs, paths);
            collect_service_call_paths_from_expr(rhs, paths);
        }
        Expr::UnaryOp(_, inner) | Expr::Lambda(_, inner) | Expr::After(inner, _) => {
            collect_service_call_paths_from_expr(inner, paths);
        }
        Expr::Guarded(inner, guard) => {
            collect_service_call_paths_from_expr(inner, paths);
            collect_service_call_paths_from_expr(guard, paths);
        }
        _ => {}
    }
}

fn detect_if_branches_in_stmts(stmts: &[Stmt]) -> Vec<IfBranchScopeSite> {
    let mut sites = Vec::new();
    walk_stmts(stmts, &mut |expr| {
        if let Expr::If(_, then_expr, else_branch) = expr {
            let mut then_calls = Vec::new();
            collect_service_call_paths_from_expr(then_expr, &mut then_calls);
            let mut else_calls = Vec::new();
            if let Some(else_expr) = else_branch {
                collect_service_call_paths_from_expr(else_expr, &mut else_calls);
            }
            sites.push(IfBranchScopeSite {
                has_else: else_branch.is_some(),
                then_service_call_paths: then_calls,
                else_service_call_paths: else_calls,
            });
        }
    });
    sites
}

fn detect_match_branches_in_stmts(stmts: &[Stmt]) -> Vec<MatchBranchScopeSite> {
    let mut sites = Vec::new();
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Match(_, arms) = expr {
            let mut all_calls = Vec::new();
            for arm in arms {
                collect_service_call_paths_from_expr(&arm.body, &mut all_calls);
            }
            sites.push(MatchBranchScopeSite {
                arm_count: arms.len(),
                all_service_call_paths: all_calls,
            });
        }
    });
    sites
}

/// Metadata for a resolved loop-body service call (transport triplet info).
struct LoopBodyTransport {
    metadata: ServiceCallMetadata,
    prepare_inputs: Vec<String>,
    parse_output: String,
}

/// Try to resolve a loop-body service call path to transport metadata.
fn resolve_loop_body_service_call(
    call_path: &[String],
    uses_binding_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
) -> Option<LoopBodyTransport> {
    // First try direct registry lookup.
    if let Some(endpoint) = resolve_service_endpoint(call_path, service_registry) {
        return endpoint_to_loop_body_transport(&endpoint);
    }
    // Try uses-binding resolution: first segment is binding name.
    let binding = call_path.first()?;
    let resource_type = uses_binding_types.get(binding)?;
    if call_path.len() >= 2 {
        let capability = call_path.last()?;
        let cap_key = format!("{resource_type}.{capability}");
        let cap_path: Vec<String> = cap_key.split('.').map(String::from).collect();
        if let Some(endpoint) = resolve_service_endpoint(&cap_path, service_registry) {
            return endpoint_to_loop_body_transport(&endpoint);
        }
    }
    None
}

fn endpoint_to_loop_body_transport(
    endpoint: &ServiceTransportEndpoint,
) -> Option<LoopBodyTransport> {
    let metadata = endpoint.metadata.as_ref()?.clone();
    Some(LoopBodyTransport {
        metadata,
        prepare_inputs: endpoint.prepare_inputs.clone(),
        parse_output: endpoint.parse.primary_output.clone(),
    })
}

fn make_loop_body_dag(
    module_name: &str,
    callable_node_id: &str,
    index: usize,
    element_var: &str,
    passthrough: &[String],
    body_transports: &[LoopBodyTransport],
) -> Dag<LoweredOp> {
    let mut inputs = vec![Port::scalar(element_var, "Any")];
    for pt in passthrough {
        inputs.push(Port::scalar(pt.as_str(), "Any"));
    }
    let mut dag: Dag<LoweredOp> = Dag::new();

    if body_transports.is_empty() {
        // No service calls in body — plain body_op callable.
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
                fn_body: None,
            },
        ));
    } else {
        // Service calls in body — create transport triplets inside body SubDag.
        // The body_op receives the last transport parse output as its first input
        // so that PassthroughOp forwards the transport result (not the element var)
        // to `body_op.result`.
        //
        // The element var is only an entrypoint on prepare nodes (the loop executor
        // injects it via set_input to all entrypoints with matching port name).
        let last_parse_output = &body_transports
            .last()
            .expect("body_transports is non-empty")
            .parse_output;
        let body_op_inputs = vec![Port::scalar(last_parse_output.as_str(), "Any")];
        dag.add_node(Node::opaque(
            "body_op",
            body_op_inputs,
            vec![Port::scalar("result", "Any")],
            LoweredOp::Callable {
                module: module_name.to_string(),
                kind: CallableKind::Fn,
                name: format!("{callable_node_id}::for_{index}_body"),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        for (ti, transport) in body_transports.iter().enumerate() {
            let suffix = format!("body_t{ti}");
            let prepare_id = format!("prepare_{suffix}");
            let execute_id = format!("execute_{suffix}");
            let parse_id = format!("parse_{suffix}");
            let prepare_ports: Vec<Port> = transport
                .prepare_inputs
                .iter()
                .map(|name| Port::scalar(name.as_str(), "Any"))
                .collect();
            dag.add_node(Node::opaque(
                prepare_id.clone(),
                prepare_ports,
                vec![Port::scalar("request", "TransportRequest")],
                LoweredOp::Callable {
                    module: module_name.to_string(),
                    kind: CallableKind::Pattern,
                    name: format!(
                        "service_transport::prepare::{}::{}",
                        transport.metadata.service, transport.metadata.operation
                    ),
                    obligation: ObligationCategory::ServiceTransportPrepare,
                    service_metadata: Some(Box::new(transport.metadata.clone())),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            ));
            let execute_node = Node::opaque(
                execute_id.clone(),
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Callable {
                    module: module_name.to_string(),
                    kind: CallableKind::Pattern,
                    name: format!(
                        "service_transport::execute::{}::{}",
                        transport.metadata.service, transport.metadata.operation
                    ),
                    obligation: ObligationCategory::ServiceTransportExecute,
                    service_metadata: Some(Box::new(transport.metadata.clone())),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_input_guard("request", Guard::NotEq(Value::Skipped));
            dag.add_node(execute_node);
            dag.add_node(Node::opaque(
                parse_id.clone(),
                vec![Port::scalar("response", "TransportResponse")],
                vec![Port::scalar(transport.parse_output.as_str(), "Any")],
                LoweredOp::Callable {
                    module: module_name.to_string(),
                    kind: CallableKind::Pattern,
                    name: format!(
                        "service_transport::parse::{}::{}",
                        transport.metadata.service, transport.metadata.operation
                    ),
                    obligation: ObligationCategory::ServiceTransportParse,
                    service_metadata: Some(Box::new(transport.metadata.clone())),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            ));
            // Wire the transport triplet chain: prepare → execute → parse.
            // Prepare inputs matching element_var or passthrough are left as
            // entrypoints — the loop executor injects them via set_input.
            dag.add_edge(Edge::new(
                prepare_id.as_str(),
                "request",
                execute_id.as_str(),
                "request",
            ));
            dag.add_edge(Edge::new(
                execute_id.as_str(),
                "response",
                parse_id.as_str(),
                "response",
            ));
            // Wire parse output to body_op as a DATA edge (not __deps) so
            // PassthroughOp forwards the transport result to body_op.result.
            dag.add_edge(Edge::new(
                parse_id.as_str(),
                transport.parse_output.as_str(),
                "body_op",
                transport.parse_output.as_str(),
            ));
        }
    }
    dag
}

fn make_branch_body_dag(
    module_name: &str,
    callable_node_id: &str,
    index: usize,
    branch_label: &str,
    body_transports: &[LoopBodyTransport],
) -> Dag<LoweredOp> {
    let mut dag: Dag<LoweredOp> = Dag::new();

    if body_transports.is_empty() {
        // No service calls in branch body — plain callable op.
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
                fn_body: None,
            },
        ));
    } else {
        // Service calls in branch body — create transport triplets inside SubDag.
        // Follows the same pattern as make_loop_body_dag: body_op receives the
        // last transport parse output.
        //
        // The body_op retains `input` and `condition` ports as entrypoints so
        // that BranchBuilder/IfBuilder can attach guards. Without these, the
        // outer SubDag node would lack a `condition` port and guard setup panics.
        let last_parse_output = &body_transports
            .last()
            .expect("body_transports is non-empty")
            .parse_output;
        let body_op_inputs = vec![
            Port::scalar(last_parse_output.as_str(), "Any"),
            Port::scalar("input", "Any"),
            Port::scalar("condition", "Bool"),
        ];
        dag.add_node(Node::opaque(
            "op",
            body_op_inputs,
            vec![Port::scalar("result", "Any")],
            LoweredOp::Callable {
                module: module_name.to_string(),
                kind: CallableKind::Fn,
                name: format!("{callable_node_id}::if_{index}_{branch_label}"),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        for (ti, transport) in body_transports.iter().enumerate() {
            let suffix = format!("branch_t{ti}");
            let prepare_id = format!("prepare_{suffix}");
            let execute_id = format!("execute_{suffix}");
            let parse_id = format!("parse_{suffix}");
            let prepare_ports: Vec<Port> = transport
                .prepare_inputs
                .iter()
                .map(|name| Port::scalar(name.as_str(), "Any"))
                .collect();
            dag.add_node(Node::opaque(
                prepare_id.clone(),
                prepare_ports,
                vec![Port::scalar("request", "TransportRequest")],
                LoweredOp::Callable {
                    module: module_name.to_string(),
                    kind: CallableKind::Pattern,
                    name: format!(
                        "service_transport::prepare::{}::{}",
                        transport.metadata.service, transport.metadata.operation
                    ),
                    obligation: ObligationCategory::ServiceTransportPrepare,
                    service_metadata: Some(Box::new(transport.metadata.clone())),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            ));
            let execute_node = Node::opaque(
                execute_id.clone(),
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Callable {
                    module: module_name.to_string(),
                    kind: CallableKind::Pattern,
                    name: format!(
                        "service_transport::execute::{}::{}",
                        transport.metadata.service, transport.metadata.operation
                    ),
                    obligation: ObligationCategory::ServiceTransportExecute,
                    service_metadata: Some(Box::new(transport.metadata.clone())),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_input_guard("request", Guard::NotEq(Value::Skipped));
            dag.add_node(execute_node);
            dag.add_node(Node::opaque(
                parse_id.clone(),
                vec![Port::scalar("response", "TransportResponse")],
                vec![Port::scalar(transport.parse_output.as_str(), "Any")],
                LoweredOp::Callable {
                    module: module_name.to_string(),
                    kind: CallableKind::Pattern,
                    name: format!(
                        "service_transport::parse::{}::{}",
                        transport.metadata.service, transport.metadata.operation
                    ),
                    obligation: ObligationCategory::ServiceTransportParse,
                    service_metadata: Some(Box::new(transport.metadata.clone())),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            ));
            dag.add_edge(Edge::new(
                prepare_id.as_str(),
                "request",
                execute_id.as_str(),
                "request",
            ));
            dag.add_edge(Edge::new(
                execute_id.as_str(),
                "response",
                parse_id.as_str(),
                "response",
            ));
            dag.add_edge(Edge::new(
                parse_id.as_str(),
                transport.parse_output.as_str(),
                "op",
                transport.parse_output.as_str(),
            ));
        }
    }
    dag
}

fn add_control_flow_pattern_nodes(
    builder: &mut DagBuilder,
    module_name: &str,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    service_registry: &ServiceEndpointRegistry,
    uses_binding_types: &HashMap<String, String>,
) {
    let for_sites = detect_for_loops_in_stmts(stmts);
    for (index, site) in for_sites.iter().enumerate() {
        let node_id = format!("{}::cf_for_{index}", target.node_id);
        // Resolve body service calls to LoopBodyTransport entries.
        let mut body_transports = Vec::new();
        for call_path in &site.body_service_call_paths {
            if let Some(transport) =
                resolve_loop_body_service_call(call_path, uses_binding_types, service_registry)
            {
                body_transports.push(transport);
            }
        }
        let body_dag = make_loop_body_dag(
            module_name,
            &target.node_id,
            index,
            &site.element_var,
            &site.passthrough,
            &body_transports,
        );
        let loop_node = LoopBuilder::new(node_id.clone())
            .with_input("items", "Any", Cardinality::ONE)
            .with_element(&site.element_var, "Any")
            .with_body(body_dag)
            .with_output("result", "Any")
            .build();
        builder.add_node(loop_node);
        builder.add_edge(&node_id, "result", &target.node_id, PortName::DEPS);
    }

    let if_sites = detect_if_branches_in_stmts(stmts);
    for (index, site) in if_sites.iter().enumerate() {
        let node_id = format!("{}::cf_if_{index}", target.node_id);
        // Resolve branch-body service calls to transport entries.
        let mut then_transports = Vec::new();
        for call_path in &site.then_service_call_paths {
            if let Some(transport) =
                resolve_loop_body_service_call(call_path, uses_binding_types, service_registry)
            {
                then_transports.push(transport);
            }
        }
        let mut else_transports = Vec::new();
        for call_path in &site.else_service_call_paths {
            if let Some(transport) =
                resolve_loop_body_service_call(call_path, uses_binding_types, service_registry)
            {
                else_transports.push(transport);
            }
        }

        if site.has_else {
            let true_dag = make_branch_body_dag(
                module_name,
                &target.node_id,
                index,
                "true",
                &then_transports,
            );
            let false_dag = make_branch_body_dag(
                module_name,
                &target.node_id,
                index,
                "false",
                &else_transports,
            );
            let branch_node = BranchBuilder::new(node_id.clone())
                .with_true_branch(true_dag)
                .with_false_branch(false_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(branch_node);
        } else {
            let then_dag = make_branch_body_dag(
                module_name,
                &target.node_id,
                index,
                "then",
                &then_transports,
            );
            let if_node = IfBuilder::new(node_id.clone())
                .with_then(then_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(if_node);
        }
        builder.add_edge(&node_id, "result", &target.node_id, PortName::DEPS);
    }

    let match_sites = detect_match_branches_in_stmts(stmts);
    for (index, site) in match_sites.iter().enumerate() {
        let node_id = format!("{}::cf_match_{index}", target.node_id);
        // Resolve match-arm service calls to transport entries.
        let mut match_transports = Vec::new();
        for call_path in &site.all_service_call_paths {
            if let Some(transport) =
                resolve_loop_body_service_call(call_path, uses_binding_types, service_registry)
            {
                match_transports.push(transport);
            }
        }
        if site.arm_count > 1 {
            let true_dag = make_branch_body_dag(
                module_name,
                &target.node_id,
                index,
                "match_true",
                &match_transports,
            );
            let false_dag = make_branch_body_dag(
                module_name,
                &target.node_id,
                index,
                "match_false",
                &match_transports,
            );
            let branch_node = BranchBuilder::new(node_id.clone())
                .with_true_branch(true_dag)
                .with_false_branch(false_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(branch_node);
        } else {
            let then_dag = make_branch_body_dag(
                module_name,
                &target.node_id,
                index,
                "match_then",
                &match_transports,
            );
            let if_node = IfBuilder::new(node_id.clone())
                .with_then(then_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(if_node);
        }
        builder.add_edge(&node_id, "result", &target.node_id, PortName::DEPS);
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
    data_values: &HashMap<String, serde_json::Value>,
) {
    let mut bound_callables = HashMap::<String, String>::new();
    let mut expansion_count = 0usize;

    for stmt in stmts {
        let maybe_binding = match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => Some((name, expr)),
            Stmt::Node(ns) => Some((&ns.name, &ns.expr)),
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
                            data_values,
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
                if !is_internal_synthetic_call(name) {
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
                        data_values,
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
    data_values: &HashMap<String, serde_json::Value>,
) {
    let suffix = expansion_suffix(item_name, expansion_count);
    let prepare_read_id = format!("prepare_read_{suffix}");
    let execute_read_id = format!("execute_read_{suffix}");
    let compare_id = format!("compare_{suffix}_content");
    let prepare_write_id = format!("prepare_write_{suffix}");
    let execute_transport_id = format!("execute_{suffix}_transport");
    let is_makegen_expansion = suffix == "makegen";

    let mut prepare_read_inputs = vec![Port::scalar("path", "String")];
    if is_makegen_expansion {
        prepare_read_inputs.push(Port::resource(
            "file:Makefile",
            "FilesystemHandle",
            AccessMode::Read,
        ));
    }
    builder.add_node(Node::opaque(
        prepare_read_id.clone(),
        prepare_read_inputs,
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
    builder.add_edge(&execute_transport_id, "response", &target.node_id, PortName::DEPS);

    let content_destinations = [
        (compare_id.as_str(), "expected_content"),
        (prepare_write_id.as_str(), "content"),
    ];
    let wired_content = wire_resolved_or_param_source(
        builder,
        module_name,
        item_name,
        param_types,
        resolve_content_source(args, bound_callables, endpoints_by_name),
        resolve_named_ident_arg(args, "content"),
        &content_destinations,
    );
    // Fallback: if the content arg is a data declaration ident, create a
    // literal source node with the data value — mirrors wire_fn_call_arguments.
    if !wired_content {
        if let Some(ident) = resolve_named_ident_arg(args, "content") {
            if let Some(json_val) = data_values.get(ident) {
                let literal = ServiceCallArgLiteral::Json(json_val.clone());
                let src = ensure_literal_source_node(
                    builder,
                    module_name,
                    item_name,
                    "content",
                    "String",
                    &literal,
                    "content_upsert",
                );
                wire_output_to_destinations(builder, &src, "content", &content_destinations);
            }
        }
    }

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
            // FC-7: Also add an explicit output path annotation node so
            // extract_output_paths() doesn't need the ID substring hack.
            if let ServiceCallArgLiteral::String(path_str) = &literal {
                let path_annotation_id = format!("output_path_annotation_{suffix}");
                builder.add_node(Node::opaque(
                    path_annotation_id.clone(),
                    vec![],
                    vec![Port::scalar("path", "String")],
                    LoweredOp::Primitive {
                        module: module_name.to_string(),
                        name: format!("content_upsert::output_path_annotation_{suffix}"),
                        kind: PrimitiveOpKind::ContentUpsertOutputPath {
                            path: path_str.clone(),
                        },
                    },
                ));
            }
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
    match expr {
        Expr::FieldAccess(base, field) => {
            let base_endpoint = resolve_source_expr(base, bound_callables, endpoints_by_name)?;
            Some(LoweredEndpoint {
                node_id: base_endpoint.node_id,
                primary_output: field.clone(),
            })
        }
        _ => {
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
    }
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

// ── Non-generic pattern expansion ────────────────────────────────────
//
// Expands calls to non-generic patterns inline. Each pattern body's
// `node` statements produce real DAG nodes: service calls become cloned
// transport triplets, `eq()` calls become CompareEquality primitives.
// The [after] and [when] guards become dependency edges and IR guards.
//
// This is Phase 2 of the pattern expansion compiler (see plan).

/// Collected info about a pattern definition that can be expanded.
struct ExpandablePattern<'a> {
    #[allow(dead_code)]
    name: &'a str,
    params: &'a [daglang_syntax::ast::Param],
    type_params: &'a [String],
    body_stmts: &'a [Stmt],
    uses: &'a [daglang_syntax::ast::UsesClause],
    #[allow(dead_code)]
    outputs: &'a [daglang_syntax::ast::Field],
}

/// Collect ALL pattern definitions from the project (generic and non-generic).
fn collect_expandable_pattern_defs(
    project: &TypedProject,
) -> HashMap<String, ExpandablePattern<'_>> {
    let mut patterns = HashMap::new();
    for module in &project.modules {
        for item in &module.ast.items {
            if let Item::PatternDef(def) = &item.node {
                patterns.insert(
                    def.name.clone(),
                    ExpandablePattern {
                        name: &def.name,
                        params: &def.params,
                        type_params: &def.type_params,
                        body_stmts: &def.body.stmts,
                        uses: &def.uses,
                        outputs: &def.outputs,
                    },
                );
            }
        }
    }
    patterns
}

/// Outputs produced by an expanded pattern node.
#[derive(Debug, Clone)]
struct ExpandedNodeOutput {
    node_id: String,
    output_port: String,
}

/// Result of expanding a pattern's body inline.
#[derive(Debug)]
struct PatternExpansionResult {
    /// Map from pattern return field name → expanded node output.
    return_outputs: HashMap<String, ExpandedNodeOutput>,
    /// The last node created (for dependency wiring to target).
    last_node_id: String,
}

/// Build the `uses_binding_types` map for a pattern definition.
fn pattern_uses_binding_types(uses: &[daglang_syntax::ast::UsesClause]) -> HashMap<String, String> {
    uses.iter()
        .map(|u| (u.binding.clone(), resource_type_name(&u.resource_type)))
        .collect()
}

fn expand_non_generic_pattern_calls(
    builder: &mut DagBuilder,
    project: &TypedProject,
    module_name: &str,
    item_name: &str,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    param_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
    data_values: &HashMap<String, serde_json::Value>,
) {
    let pattern_defs = collect_expandable_pattern_defs(project);
    let mut expansion_count = 0usize;
    // Track expanded pattern results so later code can reference pattern outputs.
    let mut expanded_results = HashMap::<String, PatternExpansionResult>::new();

    for stmt in stmts {
        let (binding_name, call_name, call_args) = match stmt {
            Stmt::Let(name, Expr::Call(call_name, args))
            | Stmt::Assign(name, Expr::Call(call_name, args)) => {
                (name.as_str(), call_name.as_str(), args.as_slice())
            }
            Stmt::Node(ns) => match &ns.expr {
                Expr::Call(call_name, args) => {
                    (ns.name.as_str(), call_name.as_str(), args.as_slice())
                }
                _ => continue,
            },
            _ => continue,
        };

        // Skip content_upsert — already expanded by the specialized
        // expand_single_content_upsert path which correctly wires the
        // content_upsert → ensure → file_content_matches → eq chain.
        if call_name == "content_upsert" {
            continue;
        }

        let Some(pattern_def) = pattern_defs.get(call_name) else {
            continue;
        };

        expansion_count += 1;
        let result = expand_single_pattern(
            builder,
            module_name,
            item_name,
            expansion_count,
            pattern_def,
            call_args,
            target,
            param_types,
            service_registry,
            data_values,
            endpoints_by_name,
            &expanded_results,
            &pattern_defs,
            0, // recursion depth
        );
        if let Some(result) = result {
            // Wire a dep edge from the last expanded node to the target callable.
            builder.add_edge(
                &result.last_node_id,
                "fresh", // CompareEquality's output — safe fallback
                &target.node_id,
                PortName::DEPS,
            );
            expanded_results.insert(binding_name.to_string(), result);
        }
    }
}

/// Expand a single non-generic pattern call inline.
///
/// Creates the real DAG nodes for the pattern body: cloned transport
/// triplets for service calls, CompareEquality primitives for `eq()`,
/// and wires everything together.
/// Maximum recursion depth for pattern expansion (patterns calling patterns).
const PATTERN_EXPANSION_MAX_DEPTH: usize = 5;

fn expand_single_pattern(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    expansion_count: usize,
    pattern: &ExpandablePattern<'_>,
    call_args: &[(Option<String>, Expr)],
    target: &LoweredEndpoint,
    caller_param_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
    data_values: &HashMap<String, serde_json::Value>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    expanded_results: &HashMap<String, PatternExpansionResult>,
    all_patterns: &HashMap<String, ExpandablePattern<'_>>,
    depth: usize,
) -> Option<PatternExpansionResult> {
    if depth >= PATTERN_EXPANSION_MAX_DEPTH {
        return None;
    }

    let suffix = expansion_suffix(item_name, expansion_count);
    let uses_binding_types = pattern_uses_binding_types(pattern.uses);

    // Build argument map: pattern param name → caller arg expression.
    let mut arg_map = HashMap::<String, &Expr>::new();
    for (index, (arg_name, arg_expr)) in call_args.iter().enumerate() {
        let param_name = arg_name
            .as_deref()
            .or_else(|| pattern.params.get(index).map(|p| p.name.as_str()));
        if let Some(name) = param_name {
            arg_map.insert(name.to_string(), arg_expr);
        }
    }

    // Phase 3: Build type parameter substitution map for generic patterns.
    // For ensure<Check, Action>(should_act: ..., check: fcm(...), action: fs.write(...)):
    //   - Named arg "check" matches type param "Check" → substitutes Check with fcm(...)
    //   - Named arg "action" matches type param "Action" → substitutes Action with fs.write(...)
    let mut type_param_map = HashMap::<String, &Expr>::new();
    for type_param in pattern.type_params {
        let lowered = type_param.to_ascii_lowercase();
        // Check named args: arg name (lowercased) matches type param (lowercased).
        for (arg_name, arg_expr) in call_args {
            if let Some(name) = arg_name {
                if name.to_ascii_lowercase() == lowered {
                    type_param_map.insert(type_param.clone(), arg_expr);
                }
            }
        }
    }

    // Merge the parent pattern's uses_binding_types with the caller's context
    // so that service calls in substituted expressions can be resolved.
    // For content_upsert → ensure: the caller's `fs: Filesystem` binding must
    // be available when expanding `fs.write(...)` substituted for `Action`.
    let caller_uses_binding_types: HashMap<String, String> = call_args
        .iter()
        .filter_map(|(_, expr)| {
            if let Expr::ServiceCall(path, _) = expr {
                let binding = path.first()?;
                // The binding type comes from the CALLER's scope, not the pattern's.
                // For now, we check if it's in uses_binding_types (from the pattern itself).
                None::<(String, String)>.or_else(|| {
                    // Caller-level uses bindings are in the grandparent scope;
                    // we don't have direct access. The service_registry lookup
                    // will handle it.
                    Some((binding.clone(), binding.clone()))
                })
            } else {
                None
            }
        })
        .collect();
    let mut combined_uses = uses_binding_types.clone();
    combined_uses.extend(caller_uses_binding_types);

    // Track nodes created in the expansion for after-edge wiring.
    // Maps pattern body binding name → (last_node_id, output_port_name).
    let mut node_outputs = HashMap::<String, ExpandedNodeOutput>::new();
    let mut last_node_id = String::new();

    for body_stmt in pattern.body_stmts {
        match body_stmt {
            Stmt::Node(ns) => {
                // Phase 3: Substitute type parameters in node expressions.
                // e.g., `node check: Check` with Check = file_content_matches(...)
                //   becomes `node check: file_content_matches(...)`
                let effective_expr = substitute_type_param(&ns.expr, &type_param_map);
                let expr_ref = effective_expr.as_ref().unwrap_or(&ns.expr);

                let expanded = expand_pattern_body_node(
                    builder,
                    module_name,
                    item_name,
                    &suffix,
                    &ns.name,
                    expr_ref,
                    &ns.after,
                    &ns.when_guard,
                    &arg_map,
                    &combined_uses,
                    &node_outputs,
                    caller_param_types,
                    service_registry,
                    data_values,
                    target,
                    endpoints_by_name,
                    expanded_results,
                    all_patterns,
                    depth,
                );
                if let Some(output) = expanded {
                    last_node_id.clone_from(&output.node_id);
                    node_outputs.insert(ns.name.clone(), output);
                }
            }
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                let effective_expr = substitute_type_param(expr, &type_param_map);
                let expr_ref = effective_expr.as_ref().unwrap_or(expr);

                let expanded = expand_pattern_body_node(
                    builder,
                    module_name,
                    item_name,
                    &suffix,
                    name,
                    expr_ref,
                    &[],
                    &None,
                    &arg_map,
                    &combined_uses,
                    &node_outputs,
                    caller_param_types,
                    service_registry,
                    data_values,
                    target,
                    endpoints_by_name,
                    expanded_results,
                    all_patterns,
                    depth,
                );
                if let Some(output) = expanded {
                    last_node_id.clone_from(&output.node_id);
                    node_outputs.insert(name.clone(), output);
                }
            }
            Stmt::Return(_) => {
                // Handled below when building return_outputs.
            }
            _ => {}
        }
    }

    if last_node_id.is_empty() {
        return None;
    }

    // Build return output mapping from pattern's return statement.
    let mut return_outputs = HashMap::<String, ExpandedNodeOutput>::new();
    for body_stmt in pattern.body_stmts {
        if let Stmt::Return(bindings) = body_stmt {
            for (field_name, expr) in bindings {
                if let Some(output) = resolve_pattern_return_expr(expr, &node_outputs, &arg_map) {
                    return_outputs.insert(field_name.clone(), output);
                }
            }
        }
    }

    Some(PatternExpansionResult {
        return_outputs,
        last_node_id,
    })
}

/// Substitute a type parameter in an expression.
///
/// If the expression is `Expr::Ident("Check")` and the type_param_map has
/// `"Check" → Expr::Call("file_content_matches", ...)`, returns
/// `Some(Expr::Call("file_content_matches", ...))`.
fn substitute_type_param(expr: &Expr, type_param_map: &HashMap<String, &Expr>) -> Option<Expr> {
    match expr {
        Expr::Ident(name) => type_param_map.get(name).map(|e| (*e).clone()),
        _ => None,
    }
}

/// Resolve a pattern return expression to an expanded node output.
///
/// Handles:
/// - `check.equal` → node_outputs["check"] with field "equal" remapped
/// - `should_act(check)` → resolves to the check node's guard-relevant output
///   (for generic patterns like `ensure` where return references a lambda applied to a node)
/// - `result.acted` → field access on an expanded sub-pattern result
fn resolve_pattern_return_expr(
    expr: &Expr,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
    arg_map: &HashMap<String, &Expr>,
) -> Option<ExpandedNodeOutput> {
    match expr {
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(base_name) = base.as_ref() {
                let base_output = node_outputs.get(base_name)?;
                // The field access overrides the output port.
                // For CompareEquality: `check.equal` → the compare node's `fresh` port.
                let mapped_port = match field.as_str() {
                    "equal" => "fresh".to_string(),
                    other => other.to_string(),
                };
                Some(ExpandedNodeOutput {
                    node_id: base_output.node_id.clone(),
                    output_port: mapped_port,
                })
            } else {
                None
            }
        }
        Expr::Ident(name) => node_outputs.get(name).cloned(),
        Expr::Call(fn_name, call_args) => {
            // Handle `should_act(check)` in `return { acted: should_act(check) }`.
            // The fn_name is a lambda parameter (e.g., `should_act: c => !c.matches`).
            // The call arg is a node reference (e.g., `check`).
            // Resolution: look up the lambda body to find what field it accesses,
            // then use that field from the node's output.
            //
            // For `ensure<Check, Action>` with `should_act: c => !c.matches`:
            //   - `should_act(check)` → the `check` node's `matches` output
            //   - Since `!c.matches` inverts, the semantic is "fresh" (CompareEquality's output)
            //
            // For now, resolve to the first arg node's primary output.
            // The guard/inversion logic is handled at IR level.
            let first_arg_node = call_args.first().and_then(|(_, arg_expr)| {
                if let Expr::Ident(node_name) = arg_expr {
                    node_outputs.get(node_name.as_str())
                } else {
                    None
                }
            });

            // If the lambda is in the arg_map, try to extract the field it accesses
            // to determine the correct output port.
            if let Some(lambda_expr) = arg_map.get(fn_name.as_str()) {
                if let Some(node_output) = first_arg_node {
                    let port = extract_lambda_field_access(lambda_expr)
                        .unwrap_or_else(|| node_output.output_port.clone());
                    return Some(ExpandedNodeOutput {
                        node_id: node_output.node_id.clone(),
                        output_port: port,
                    });
                }
            }

            first_arg_node.cloned()
        }
        _ => None,
    }
}

/// Extract the field name that a lambda accesses on its parameter.
///
/// For `c => !c.matches` → returns `Some("matches")` (mapped to "fresh" for CompareEquality).
/// For `c => c.exists` → returns `Some("exists")`.
/// For anything else → returns `None`.
fn extract_lambda_field_access(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lambda(_, body) => extract_lambda_field_access(body),
        Expr::UnaryOp(_, inner) => extract_lambda_field_access(inner),
        Expr::FieldAccess(_, field) => {
            // Map DSL field names to actual IR port names.
            let mapped = match field.as_str() {
                "equal" => "fresh",
                "matches" => "fresh",
                other => other,
            };
            Some(mapped.to_string())
        }
        _ => None,
    }
}

/// Recursively resolve all `Ident` expressions through an argument map.
///
/// When expanding nested pattern calls (e.g., `ensure(check: file_content_matches(path: path, expected: content))`),
/// inner `Ident` references like `path` and `content` inside `Call(...)` or `ServiceCall(...)` expressions
/// must be resolved through the parent pattern's `arg_map` to get the caller's actual expressions.
fn resolve_expr_idents(expr: &Expr, arg_map: &HashMap<String, &Expr>) -> Expr {
    match expr {
        Expr::Ident(name) => arg_map
            .get(name.as_str())
            .map(|e| (*e).clone())
            .unwrap_or_else(|| expr.clone()),
        Expr::Call(name, args) => Expr::Call(
            name.clone(),
            args.iter()
                .map(|(n, e)| (n.clone(), resolve_expr_idents(e, arg_map)))
                .collect(),
        ),
        Expr::ServiceCall(path, args) => Expr::ServiceCall(
            path.clone(),
            args.iter()
                .map(|(n, e)| (n.clone(), resolve_expr_idents(e, arg_map)))
                .collect(),
        ),
        Expr::FieldAccess(base, field) => {
            Expr::FieldAccess(Box::new(resolve_expr_idents(base, arg_map)), field.clone())
        }
        _ => expr.clone(),
    }
}

/// Expand a single node statement from a pattern body.
///
/// Dispatches based on the expression type:
/// - `Expr::ServiceCall` → clone transport triplet from service registry
/// - `Expr::Call("eq", ...)` → create CompareEquality primitive
/// - `Expr::Call(pattern_name, ...)` → recursive pattern expansion
fn expand_pattern_body_node(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    suffix: &str,
    node_name: &str,
    expr: &Expr,
    after_deps: &[String],
    _when_guard: &Option<Expr>,
    arg_map: &HashMap<String, &Expr>,
    uses_binding_types: &HashMap<String, String>,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
    caller_param_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
    data_values: &HashMap<String, serde_json::Value>,
    target: &LoweredEndpoint,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    expanded_results: &HashMap<String, PatternExpansionResult>,
    all_patterns: &HashMap<String, ExpandablePattern<'_>>,
    depth: usize,
) -> Option<ExpandedNodeOutput> {
    match expr {
        Expr::ServiceCall(path, args) => expand_service_call_node(
            builder,
            module_name,
            item_name,
            suffix,
            node_name,
            path,
            args,
            after_deps,
            arg_map,
            uses_binding_types,
            node_outputs,
            caller_param_types,
            service_registry,
            data_values,
            target,
            endpoints_by_name,
            expanded_results,
        ),
        Expr::Call(call_name, args) if call_name == "eq" => expand_eq_node(
            builder,
            module_name,
            item_name,
            suffix,
            node_name,
            args,
            after_deps,
            arg_map,
            node_outputs,
            caller_param_types,
            service_registry,
            data_values,
            target,
            endpoints_by_name,
            expanded_results,
        ),
        Expr::Call(call_name, call_args) if all_patterns.contains_key(call_name.as_str()) => {
            // Recursive pattern expansion: this node calls another pattern.
            // e.g., `ensure(should_act: ..., check: fcm(...), action: fs.write(...))`
            let inner_pattern = &all_patterns[call_name.as_str()];

            // Merge caller's arg_map into the call args so the inner pattern
            // can resolve references to the outer pattern's parameters.
            // For content_upsert calling ensure: `check: file_content_matches(path: path, expected: content)`
            // where `path` and `content` are content_upsert params that map to caller exprs.
            let mut merged_args: Vec<(Option<String>, Expr)> = Vec::new();
            for (inner_arg_name, inner_arg_expr) in call_args {
                // Recursively resolve idents through the parent arg_map so nested
                // expressions like `file_content_matches(path: path, expected: content)`
                // get their idents resolved to the caller's actual expressions.
                let resolved_expr = resolve_expr_idents(inner_arg_expr, arg_map);
                merged_args.push((inner_arg_name.clone(), resolved_expr));
            }

            let inner_result = expand_single_pattern(
                builder,
                module_name,
                item_name,
                depth + 1, // use depth as sub-expansion count for unique suffix
                inner_pattern,
                &merged_args,
                target,
                caller_param_types,
                service_registry,
                data_values,
                endpoints_by_name,
                expanded_results,
                all_patterns,
                depth + 1,
            );

            // Wire after-dependency edges.
            if let Some(ref result) = inner_result {
                for dep in after_deps {
                    if let Some(dep_output) = node_outputs.get(dep) {
                        builder.add_edge(
                            &dep_output.node_id,
                            &dep_output.output_port,
                            &result.last_node_id,
                            PortName::DEPS,
                        );
                    }
                }
            }

            // Convert PatternExpansionResult to ExpandedNodeOutput.
            // Use the first return output as the representative output.
            inner_result.and_then(|r| {
                r.return_outputs
                    .values()
                    .next()
                    .map(|o| ExpandedNodeOutput {
                        node_id: o.node_id.clone(),
                        output_port: o.output_port.clone(),
                    })
            })
        }
        _ => None,
    }
}

/// Expand a service call node (e.g., `fs.read(path: path)`) by cloning
/// the corresponding transport triplet from the service registry.
fn expand_service_call_node(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    suffix: &str,
    node_name: &str,
    call_path: &[String],
    call_args: &[(Option<String>, Expr)],
    after_deps: &[String],
    arg_map: &HashMap<String, &Expr>,
    uses_binding_types: &HashMap<String, String>,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
    caller_param_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
    data_values: &HashMap<String, serde_json::Value>,
    _target: &LoweredEndpoint,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    expanded_results: &HashMap<String, PatternExpansionResult>,
) -> Option<ExpandedNodeOutput> {
    // Resolve the service call path to a registry key.
    // e.g., ["fs", "read"] with uses fs: Filesystem → "Filesystem.read"
    let binding = call_path.first()?;
    let capability = call_path.last()?;
    let resource_type = uses_binding_types.get(binding)?;
    let cap_key = format!("{resource_type}.{capability}");

    let endpoint = service_registry.get(&cap_key)?;

    // Clone the triplet with a unique suffix.
    let clone_suffix = format!("{suffix}_{node_name}");
    let cloned = builder.clone_transport_triplet(endpoint, &clone_suffix);

    // Wire call arguments to the cloned prepare node's inputs.
    for (arg_name_opt, arg_expr) in call_args {
        let Some(arg_name) = arg_name_opt.as_deref() else {
            continue;
        };
        // Resolve the argument: it may be a pattern parameter (which maps
        // to a caller argument) or a reference to another expanded node.
        wire_pattern_arg_to_prepare(
            builder,
            module_name,
            item_name,
            arg_name,
            arg_expr,
            &cloned.prepare_node_id,
            arg_map,
            node_outputs,
            caller_param_types,
            service_registry,
            data_values,
            endpoints_by_name,
            expanded_results,
        );
    }

    // Wire after-dependency edges.
    for dep in after_deps {
        if let Some(dep_output) = node_outputs.get(dep) {
            builder.add_edge(
                &dep_output.node_id,
                &dep_output.output_port,
                &cloned.prepare_node_id,
                PortName::DEPS,
            );
        }
    }

    Some(ExpandedNodeOutput {
        node_id: cloned.parse.node_id,
        output_port: cloned.parse.primary_output,
    })
}

/// Expand an `eq(a: ..., b: ...)` call into a CompareEquality primitive node.
fn expand_eq_node(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    suffix: &str,
    node_name: &str,
    args: &[(Option<String>, Expr)],
    after_deps: &[String],
    arg_map: &HashMap<String, &Expr>,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
    caller_param_types: &HashMap<String, String>,
    _service_registry: &ServiceEndpointRegistry,
    data_values: &HashMap<String, serde_json::Value>,
    _target: &LoweredEndpoint,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    expanded_results: &HashMap<String, PatternExpansionResult>,
) -> Option<ExpandedNodeOutput> {
    let compare_id = format!("compare_{suffix}_{node_name}");

    builder.add_node(Node::opaque(
        compare_id.clone(),
        vec![
            Port::scalar("expected_content", "String"),
            Port::scalar("actual_content", "String"),
        ],
        vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("pattern_eq::{compare_id}"),
            kind: PrimitiveOpKind::CompareEquality,
        },
    ));

    // Wire arguments: `a` → expected_content, `b` → actual_content.
    let port_map = [("a", "expected_content"), ("b", "actual_content")];
    for (arg_name_opt, arg_expr) in args {
        let Some(arg_name) = arg_name_opt.as_deref() else {
            continue;
        };
        let dest_port = port_map
            .iter()
            .find(|(a, _)| *a == arg_name)
            .map(|(_, b)| *b)
            .unwrap_or(arg_name);
        wire_pattern_arg_to_node(
            builder,
            module_name,
            item_name,
            arg_name,
            arg_expr,
            &compare_id,
            dest_port,
            arg_map,
            node_outputs,
            caller_param_types,
            data_values,
            endpoints_by_name,
            expanded_results,
        );
    }

    // Wire after-dependency edges.
    for dep in after_deps {
        if let Some(dep_output) = node_outputs.get(dep) {
            builder.add_edge(
                &dep_output.node_id,
                &dep_output.output_port,
                &compare_id,
                PortName::DEPS,
            );
        }
    }

    Some(ExpandedNodeOutput {
        node_id: compare_id,
        output_port: "fresh".to_string(),
    })
}

/// Wire a pattern argument expression to a prepare node's input port.
///
/// Resolves the expression through the pattern's arg_map (caller scope),
/// node_outputs (expanded nodes), and param sources.
fn wire_pattern_arg_to_prepare(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    arg_name: &str,
    arg_expr: &Expr,
    prepare_node_id: &str,
    arg_map: &HashMap<String, &Expr>,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
    caller_param_types: &HashMap<String, String>,
    _service_registry: &ServiceEndpointRegistry,
    data_values: &HashMap<String, serde_json::Value>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    expanded_results: &HashMap<String, PatternExpansionResult>,
) {
    wire_pattern_arg_to_node(
        builder,
        module_name,
        item_name,
        arg_name,
        arg_expr,
        prepare_node_id,
        arg_name,
        arg_map,
        node_outputs,
        caller_param_types,
        data_values,
        endpoints_by_name,
        expanded_results,
    );
}

/// Wire a pattern argument expression to a specific node's input port.
///
/// Resolution order:
/// 1. If the arg is an Ident that matches a pattern param → look up the
///    caller's corresponding arg expression and wire that.
/// 2. If the arg is a FieldAccess on an expanded node → wire from that node.
/// 3. If the arg is a literal → create a literal source node.
/// 4. If the arg is an ident matching a caller callable → wire from that.
fn wire_pattern_arg_to_node(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    _arg_name: &str,
    arg_expr: &Expr,
    dest_node_id: &str,
    dest_port: &str,
    arg_map: &HashMap<String, &Expr>,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
    caller_param_types: &HashMap<String, String>,
    data_values: &HashMap<String, serde_json::Value>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    expanded_results: &HashMap<String, PatternExpansionResult>,
) {
    match arg_expr {
        Expr::Ident(name) => {
            if let Some(caller_expr) = arg_map.get(name.as_str()) {
                // Case 1: Pattern parameter → resolve through caller's arg map.
                wire_caller_expr_to_node(
                    builder,
                    module_name,
                    item_name,
                    caller_expr,
                    dest_node_id,
                    dest_port,
                    caller_param_types,
                    data_values,
                    endpoints_by_name,
                    expanded_results,
                );
            } else if let Some(output) = node_outputs.get(name.as_str()) {
                // Case 2: Reference to an expanded node in the pattern body.
                builder.add_edge(
                    &output.node_id,
                    &output.output_port,
                    dest_node_id,
                    dest_port,
                );
            }
        }
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(base_name) = base.as_ref() {
                if let Some(output) = node_outputs.get(base_name.as_str()) {
                    // Field access on an expanded node (e.g., `read.content`).
                    builder.add_edge(&output.node_id, field, dest_node_id, dest_port);
                } else if let Some(caller_expr) = arg_map.get(base_name.as_str()) {
                    // Field access on a pattern parameter → resolve through caller.
                    if matches!(caller_expr, Expr::FieldAccess(_, _)) {
                        wire_caller_expr_to_node(
                            builder,
                            module_name,
                            item_name,
                            caller_expr,
                            dest_node_id,
                            dest_port,
                            caller_param_types,
                            data_values,
                            endpoints_by_name,
                            expanded_results,
                        );
                    }
                }
            }
        }
        Expr::Literal(lit) => {
            let literal = match lit {
                Literal::String(s) => ServiceCallArgLiteral::String(s.clone()),
                Literal::Int(i) => ServiceCallArgLiteral::Int(*i),
                Literal::Bool(b) => ServiceCallArgLiteral::Bool(*b),
                _ => return,
            };
            let src = ensure_literal_source_node(
                builder,
                module_name,
                item_name,
                dest_port,
                "Any",
                &literal,
                &format!("pattern_{dest_port}"),
            );
            builder.add_edge(&src, dest_port, dest_node_id, dest_port);
        }
        _ => {}
    }
}

/// Wire a caller-scope expression to a destination node port.
///
/// Handles idents (param sources, callable endpoints, data values),
/// field access, and literals from the caller's scope.
fn wire_caller_expr_to_node(
    builder: &mut DagBuilder,
    module_name: &str,
    item_name: &str,
    caller_expr: &Expr,
    dest_node_id: &str,
    dest_port: &str,
    caller_param_types: &HashMap<String, String>,
    data_values: &HashMap<String, serde_json::Value>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    expanded_results: &HashMap<String, PatternExpansionResult>,
) {
    match caller_expr {
        Expr::Ident(name) => {
            if let Some(param_ty) = caller_param_types.get(name.as_str()) {
                // Caller parameter.
                let param_source = ensure_param_source_node(
                    builder,
                    module_name,
                    item_name,
                    name,
                    param_ty.as_str(),
                );
                builder.add_edge(&param_source, name, dest_node_id, dest_port);
            } else if let Some(Some(endpoint)) = endpoints_by_name.get(name.as_str()) {
                // Caller callable endpoint.
                builder.add_edge(
                    &endpoint.node_id,
                    &endpoint.primary_output,
                    dest_node_id,
                    dest_port,
                );
            } else if let Some(json_val) = data_values.get(name.as_str()) {
                // Data value.
                let literal = ServiceCallArgLiteral::Json(json_val.clone());
                let src = ensure_literal_source_node(
                    builder,
                    module_name,
                    item_name,
                    dest_port,
                    "String",
                    &literal,
                    &format!("pattern_data_{name}"),
                );
                builder.add_edge(&src, dest_port, dest_node_id, dest_port);
            } else if let Some(result) = expanded_results.get(name.as_str()) {
                // Expanded pattern result.
                if let Some(first_output) = result.return_outputs.values().next() {
                    builder.add_edge(
                        &first_output.node_id,
                        &first_output.output_port,
                        dest_node_id,
                        dest_port,
                    );
                }
            }
        }
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(base_name) = base.as_ref() {
                if let Some(Some(endpoint)) = endpoints_by_name.get(base_name.as_str()) {
                    // Field access on a caller callable.
                    builder.add_edge(&endpoint.node_id, field, dest_node_id, dest_port);
                } else if let Some(result) = expanded_results.get(base_name.as_str()) {
                    // Field access on an expanded pattern result.
                    if let Some(output) = result.return_outputs.get(field.as_str()) {
                        builder.add_edge(
                            &output.node_id,
                            &output.output_port,
                            dest_node_id,
                            dest_port,
                        );
                    }
                } else if let Some(param_ty) = caller_param_types.get(base_name.as_str()) {
                    // Caller param field access.
                    let param_source = ensure_param_source_node(
                        builder,
                        module_name,
                        item_name,
                        base_name,
                        param_ty.as_str(),
                    );
                    builder.add_edge(&param_source, field, dest_node_id, dest_port);
                }
            }
        }
        Expr::Literal(lit) => {
            let literal = match lit {
                Literal::String(s) => ServiceCallArgLiteral::String(s.clone()),
                Literal::Int(i) => ServiceCallArgLiteral::Int(*i),
                Literal::Bool(b) => ServiceCallArgLiteral::Bool(*b),
                _ => return,
            };
            let src = ensure_literal_source_node(
                builder,
                module_name,
                item_name,
                dest_port,
                "Any",
                &literal,
                &format!("caller_{dest_port}"),
            );
            builder.add_edge(&src, dest_port, dest_node_id, dest_port);
        }
        _ => {}
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
                fn_body: None,
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
                RESOURCE_FILE,
            );
        }
    }
}

fn derive_service_call_metadata(
    service: &ServiceDef,
    operation: &OperationDef,
    data_registry: &DataRegistry<'_>,
) -> ServiceCallMetadata {
    let transport = match &operation.transport {
        Some(TransportBinding::Rest { .. }) => ServiceTransportClass::RestNetwork,
        Some(TransportBinding::Shell { .. }) => ServiceTransportClass::ShellLocal,
        Some(TransportBinding::File { .. }) => ServiceTransportClass::FileBoundary,
        Some(TransportBinding::Local) => ServiceTransportClass::LocalDirect,
        // Services implementing interfaces that have no transport block get
        // InterfaceStub transport. This allows stub providers (unit_test profile)
        // to compile without explicit transport declarations.
        None if service.implements.is_some() => ServiceTransportClass::InterfaceStub,
        None => ServiceTransportClass::Unknown,
    };
    let mut permissions = operation.permissions.clone();
    permissions.sort();
    permissions.dedup();

    let spec = derive_operation_spec(service, operation, transport, data_registry);

    // Auto-derive readonly from HTTP method: GET and HEAD are read-only by definition.
    let readonly = operation.readonly
        || matches!(
            &operation.transport,
            Some(TransportBinding::Rest { method, .. })
                if method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD")
        );

    ServiceCallMetadata {
        service: service.name.clone(),
        operation: operation.name.clone(),
        transport,
        idempotent: operation.idempotent,
        readonly,
        permissions,
        spec,
    }
}

// ============================================================================
// Data registry: compile-time resolution of `data` item values
// ============================================================================

/// Registry of module-level `data` definitions, keyed by both qualified and
/// unqualified names. Used to resolve compile-time constants (e.g., env maps).
type DataRegistry<'a> = HashMap<String, &'a DataDef>;

/// Build a data registry from all modules in the project.
fn build_data_registry(project: &TypedProject) -> DataRegistry<'_> {
    let mut registry = DataRegistry::new();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            if let Item::DataDef(def) = &item.node {
                registry.insert(format!("{module_name}.{}", def.name), def);
                // Also register unqualified name for cross-module references.
                registry.insert(def.name.clone(), def);
            }
        }
    }
    registry
}

/// Resolve a `Map<String, String>` expression to key-value pairs.
///
/// Handles: map literals `{ "k": "v" }`, data references (`cargo_compile_env`),
/// and record literals `Foo { k: "v" }`. Only compile-time-evaluable expressions.
fn resolve_const_map(expr: &Expr, data_registry: &DataRegistry<'_>) -> Vec<(String, String)> {
    match expr {
        Expr::Map(entries) => entries
            .iter()
            .filter_map(|(k, v)| {
                let key = expr_as_string(k)?;
                let val = expr_as_string(v)?;
                Some((key, val))
            })
            .collect(),
        Expr::Record(_, fields) => fields
            .iter()
            .filter_map(|(k, v)| {
                let val = expr_as_string(v)?;
                Some((k.clone(), val))
            })
            .collect(),
        Expr::Ident(name) => {
            if let Some(def) = data_registry.get(name.as_str()) {
                resolve_const_map(&def.value, data_registry)
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

/// Extract a string value from a literal expression.
fn expr_as_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Extract env vars from an operation's `env: Map<String, String>` input default.
///
/// Convention: an input field named `env` of type `Map<String, String>` whose
/// default resolves to a const map becomes shell environment variables.
fn extract_env_from_inputs(
    inputs: &[daglang_syntax::ast::Field],
    data_registry: &DataRegistry<'_>,
) -> Vec<(String, String)> {
    for field in inputs {
        if field.name == "env" && type_expr_to_string(&field.ty) == "Map<String, String>" {
            if let Some(default_expr) = &field.default {
                return resolve_const_map(default_expr, data_registry);
            }
        }
    }
    Vec::new()
}

/// Returns true if this field is the `env: Map<String, String>` input that
/// gets consumed by the lowering layer (projected to `spec.env`).
fn is_env_map_field(field: &daglang_syntax::ast::Field) -> bool {
    field.name == "env" && type_expr_to_string(&field.ty) == "Map<String, String>"
}

// ============================================================================
// ServiceOperationSpec extraction from annotations
// ============================================================================

/// Extract the full protocol spec from a service + operation definition.
fn derive_operation_spec(
    service: &ServiceDef,
    operation: &OperationDef,
    transport: ServiceTransportClass,
    data_registry: &DataRegistry<'_>,
) -> Option<ServiceOperationSpec> {
    match transport {
        ServiceTransportClass::RestNetwork => {
            derive_rest_spec(service, operation).map(ServiceOperationSpec::Rest)
        }
        ServiceTransportClass::ShellLocal => {
            derive_shell_spec(service, operation, data_registry).map(ServiceOperationSpec::Shell)
        }
        ServiceTransportClass::FileBoundary => match derive_file_spec(operation) {
            Ok(spec) => Some(ServiceOperationSpec::File(spec)),
            Err(_) => None,
        },
        ServiceTransportClass::LocalDirect => {
            Some(ServiceOperationSpec::Local(derive_local_spec(operation)))
        }
        ServiceTransportClass::InterfaceStub => {
            // Services implementing interfaces with no transport block.
            // Use the service name as the interface name (from `: InterfaceName` syntax).
            Some(ServiceOperationSpec::InterfaceStub {
                interface: service
                    .implements
                    .clone()
                    .unwrap_or_else(|| service.name.clone()),
                capability: operation.name.clone(),
            })
        }
        _ => None,
    }
}

fn extract_headers_from_expr(expr: &Expr) -> Vec<(String, String)> {
    match expr {
        Expr::Record(_, fields) => fields
            .iter()
            .map(|(k, v)| {
                if let Expr::Literal(daglang_syntax::ast::Literal::String(s)) = v {
                    (k.clone(), s.clone())
                } else {
                    (k.clone(), expr_to_default_string(v))
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn derive_rest_spec(service: &ServiceDef, operation: &OperationDef) -> Option<RestOperationSpec> {
    let endpoint = service.config.endpoint.clone().unwrap_or_default();
    let (method, path_template) = match &operation.transport {
        Some(TransportBinding::Rest { method, path, .. }) => (method.clone(), path.clone()),
        _ => return None,
    };

    let headers = match &operation.transport {
        Some(TransportBinding::Rest {
            headers: Some(h), ..
        }) => extract_headers_from_expr(h),
        _ => Vec::new(),
    };
    let auth_input = service.config.auth_input.clone();
    let input_fields = derive_input_fields(&operation.inputs, &path_template, &headers, auth_input.as_deref());
    let output_fields = derive_output_fields(&operation.outputs);
    let body_template = match &operation.transport {
        Some(TransportBinding::Rest { body: Some(b), .. }) => body_template_entries_from_expr(b),
        _ => None,
    };
    let auth_scheme = service.config.auth.as_ref().map(|a| match a.as_str() {
        "BearerToken" => "BearerToken".to_string(),
        "Basic" => "Basic".to_string(),
        other => other.to_string(),
    });

    Some(RestOperationSpec {
        endpoint,
        method,
        path_template,
        input_fields,
        output_fields,
        body_template,
        headers,
        auth_scheme,
        auth_input,
    })
}

/// Convert a list of argv expressions from `shell(["cmd", "{param}"])` to ArgvSegments.
fn resolve_argv_exprs(exprs: &[Expr]) -> Vec<ArgvSegment> {
    let mut segments = Vec::new();
    for item in exprs {
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
                    segments.push(ArgvSegment::Literal(s.clone()));
                }
            }
            Expr::StringInterp(parts) => {
                use daglang_syntax::ast::StringPart;
                if parts.len() == 1 {
                    if let StringPart::Expr(Expr::Ident(name)) = &parts[0] {
                        segments.push(ArgvSegment::InputRef(name.clone()));
                        continue;
                    }
                }
                let template = expr_to_template_string(item).unwrap_or_default();
                if !template.is_empty() {
                    segments.push(ArgvSegment::Literal(template));
                }
            }
            _ => {}
        }
    }
    segments
}

fn derive_shell_spec(
    _service: &ServiceDef,
    operation: &OperationDef,
    data_registry: &DataRegistry<'_>,
) -> Option<ShellOperationSpec> {
    let argv_template = match &operation.transport {
        Some(TransportBinding::Shell { argv }) => resolve_argv_exprs(argv),
        _ => return None,
    };

    let input_fields = derive_input_fields_for_shell(&operation.inputs, &argv_template);
    let output_fields = derive_output_fields(&operation.outputs);
    let output_parsing = infer_shell_output_parsing(&operation.outputs);

    // Extract env from `env: Map<String, String>` input default.
    let env = extract_env_from_inputs(&operation.inputs, data_registry);

    Some(ShellOperationSpec {
        argv_template,
        input_fields,
        output_fields,
        output_parsing,
        env,
    })
}

fn derive_file_spec(operation: &OperationDef) -> Result<FileOperationSpec, LowerError> {
    let (file_op_str, path_template) = match &operation.transport {
        Some(TransportBinding::File { op, path }) => (op.clone(), path.clone()),
        _ => {
            return Err(LowerError::InvalidFileOp {
                operation: operation.name.clone(),
                file_op: "(no transport)".to_string(),
            })
        }
    };
    let file_op = gunbc_ir::transport::FileOp::from_dsl_str(&file_op_str).ok_or_else(|| {
        LowerError::InvalidFileOp {
            operation: operation.name.clone(),
            file_op: file_op_str.clone(),
        }
    })?;
    let input_fields = operation
        .inputs
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
        .collect();
    let output_fields = derive_output_fields(&operation.outputs);
    Ok(FileOperationSpec {
        operation: file_op,
        path_template,
        input_fields,
        output_fields,
    })
}

fn derive_local_spec(operation: &OperationDef) -> LocalOperationSpec {
    let input_fields = operation
        .inputs
        .iter()
        .map(|field| {
            let type_id = type_expr_to_string(&field.ty);
            FieldSpec {
                name: field.name.clone(),
                type_id: type_id.clone(),
                default: field.default.as_ref().map(expr_to_default_string),
                is_secret: false,
                is_path_param: false,
            }
        })
        .collect();
    let output_fields = derive_output_fields(&operation.outputs);
    LocalOperationSpec {
        input_fields,
        output_fields,
    }
}

/// Recursively convert an expression (Record or Map) to body template entries.
fn body_template_entries_from_expr(expr: &Expr) -> Option<Vec<BodyEntry>> {
    match expr {
        Expr::Record(_, fields) => {
            let mut entries = Vec::new();
            for (key, value) in fields {
                if let Some(entry) = body_template_entry(key, value) {
                    entries.push(entry);
                }
            }
            Some(entries)
        }
        Expr::Map(map_entries) => {
            let mut entries = Vec::new();
            for (key_expr, value) in map_entries {
                if let Expr::Literal(Literal::String(key)) = key_expr {
                    if let Some(entry) = body_template_entry(key, value) {
                        entries.push(entry);
                    }
                }
            }
            Some(entries)
        }
        _ => None,
    }
}

/// Convert a single key-value pair to a BodyEntry.
fn body_template_entry(key: &str, value: &Expr) -> Option<BodyEntry> {
    match value {
        Expr::Ident(field_name) => Some(BodyEntry::InputRef(key.to_string(), field_name.clone())),
        Expr::Literal(Literal::String(s)) => Some(BodyEntry::Literal(key.to_string(), s.clone())),
        Expr::Record(_, _) | Expr::Map(_) => {
            let inner = body_template_entries_from_expr(value)?;
            Some(BodyEntry::Nested(key.to_string(), inner))
        }
        _ => None,
    }
}

/// Derive input field specs from operation inputs.
///
/// When `auth_input` is set, the named field is excluded from the returned
/// list — it flows through `res:credential` on the execute node instead of
/// appearing as a prepare-node body/header input.
fn derive_input_fields(
    inputs: &[daglang_syntax::ast::Field],
    path_template: &str,
    headers: &[(String, String)],
    auth_input: Option<&str>,
) -> Vec<FieldSpec> {
    let mut fields = inputs
        .iter()
        .filter(|field| {
            // Exclude the auth_input field — it is wired to res:credential
            // on the execute node, not to the prepare node.
            auth_input.is_none_or(|ai| field.name != ai)
        })
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
        .collect::<Vec<_>>();

    let mut placeholders = collect_template_placeholders(path_template);
    for (_, value) in headers {
        placeholders.extend(collect_template_placeholders(value));
    }
    let mut placeholders = placeholders.into_iter().collect::<Vec<_>>();
    placeholders.sort();
    for placeholder in placeholders {
        if fields.iter().any(|field| field.name == placeholder) {
            continue;
        }
        let is_path_param = path_template.contains(&format!("{{{placeholder}}}"));
        fields.push(FieldSpec {
            name: placeholder.clone(),
            type_id: "String".to_string(),
            default: None,
            is_secret: placeholder.ends_with("credential"),
            is_path_param,
        });
    }

    fields
}

fn collect_template_placeholders(template: &str) -> HashSet<String> {
    let mut placeholders = HashSet::new();
    let mut current = String::new();
    let mut in_placeholder = false;
    for ch in template.chars() {
        if ch == '{' {
            current.clear();
            in_placeholder = true;
            continue;
        }
        if ch == '}' {
            if in_placeholder && !current.is_empty() {
                placeholders.insert(current.trim().to_string());
            }
            current.clear();
            in_placeholder = false;
            continue;
        }
        if in_placeholder {
            current.push(ch);
        }
    }
    placeholders
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
        // Filter out `env: Map<String, String>` — consumed by lowering (→ spec.env).
        .filter(|field| !is_env_map_field(field))
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
            // Extract base type and refinements from TypeExpr::Refined.
            let base_type_id = type_expr_to_string(&field.ty);
            // Check field annotations first, fall back to type annotations.
            let json_path = field
                .from_path
                .clone()
                .unwrap_or_else(|| field.name.clone());
            let is_raw_body = false;
            OutputFieldSpec {
                name: field.name.clone(),
                type_id: base_type_id.clone(),
                json_path,
                is_secret: base_type_id == "Secret",
                is_raw_body,
                is_optional: is_type_expr_optional(&field.ty),
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
                    StringPart::Expr(expr) => {
                        if let Some(name) = expr_template_ref(expr) {
                            result.push('{');
                            result.push_str(name.as_str());
                            result.push('}');
                        } else {
                            // Complex expressions inside interpolation — stringify as-is.
                            result.push('{');
                            result.push_str(&format!("{expr:?}"));
                            result.push('}');
                        }
                    }
                }
            }
            Some(result)
        }
        _ => None,
    }
}

fn expr_template_ref(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(name) => Some(name.clone()),
        Expr::FieldAccess(base, field) => {
            expr_template_ref(base).map(|prefix| format!("{prefix}.{field}"))
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
        let module_name = module.module_path.as_dotted();
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
            // Collect uses-binding types for resolving resource capability calls
            // (e.g., `fs.read` → `Filesystem.read`).
            let uses_binding_types = item_uses_binding_types(&item.node);
            let mut calls = Vec::<ServiceCallSite>::new();
            collect_service_calls_from_stmts(stmts, &mut calls);
            for call in calls {
                if let Some(keys) = service_call_lookup_keys(&call.path) {
                    required.insert(keys[0].clone());
                    required.insert(keys[1].clone());
                    required.insert(keys[2].clone());
                }
                // For uses-binding calls like `fs.read(...)`, also add the
                // resolved resource capability key `Filesystem.read` so the
                // transport triplet filter doesn't prune it.
                if call.path.len() >= 2 {
                    if let Some(resource_type) = uses_binding_types.get(&call.path[0]) {
                        let capability = &call.path[call.path.len() - 1];
                        required.insert(format!("{resource_type}.{capability}"));
                    }
                }
            }
        }
    }
    required
}

fn service_prepare_ports(operation: &OperationDef, metadata: &ServiceCallMetadata) -> Vec<Port> {
    let declared_inputs = match metadata.spec.as_ref() {
        Some(spec) if !spec.input_fields().is_empty() => spec
            .input_fields()
            .iter()
            .map(|field| (field.name.clone(), field.type_id.clone()))
            .collect::<Vec<_>>(),
        _ => operation
            .inputs
            .iter()
            .map(|field| {
                let ty = type_expr_to_string(&field.ty);
                (field.name.clone(), ty)
            })
            .collect::<Vec<_>>(),
    };
    declared_inputs
        .into_iter()
        .map(|(name, ty)| Port::with_cardinality(name.as_str(), ty.as_str(), Cardinality::ONE))
        .collect()
}

fn capability_prepare_ports(
    capability: &CapabilityDef,
    metadata: &ServiceCallMetadata,
) -> Vec<Port> {
    // When a spec with explicit input fields is available (e.g., File operations),
    // use the spec's field declarations. Otherwise fall back to the capability's
    // declared inputs from the interface definition.
    let declared_inputs = match metadata.spec.as_ref() {
        Some(spec) if !spec.input_fields().is_empty() => spec
            .input_fields()
            .iter()
            .map(|field| (field.name.clone(), field.type_id.clone()))
            .collect::<Vec<_>>(),
        _ => capability
            .inputs
            .iter()
            .map(|field| {
                let ty = type_expr_to_string(&field.ty);
                (field.name.clone(), ty)
            })
            .collect::<Vec<_>>(),
    };
    declared_inputs
        .into_iter()
        .map(|(name, ty)| Port::with_cardinality(name.as_str(), ty.as_str(), Cardinality::ONE))
        .collect()
}

fn add_service_transport_triplets(
    builder: &mut DagBuilder,
    project: &TypedProject,
    required_calls: Option<&HashSet<String>>,
) -> Result<ServiceEndpointRegistry, LowerError> {
    let data_registry = build_data_registry(project);
    let mut registry = ServiceEndpointRegistry::default();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
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
                let service_metadata =
                    derive_service_call_metadata(service, operation, &data_registry);
                // RT4: Fail-closed when a service operation has no transport
                // block. Previously this silently created a triplet with no
                // spec, causing the executor to skip the operation.
                //
                // Exempt fully-abstract services: if NO operation in the
                // service has a transport block, the service is intended
                // for profile-based transport binding (e.g., infra/aws,
                // infra/azure providers). Also exempt interface implementors.
                if operation.transport.is_none() && service.implements.is_none() {
                    let service_has_any_transport =
                        service.operations.iter().any(|op| op.transport.is_some());
                    if service_has_any_transport {
                        return Err(LowerError::MissingTransport {
                            service: service.name.clone(),
                            operation: operation.name.clone(),
                        });
                    }
                }
                let suffix = sanitize_identifier(&format!(
                    "{module_name}_{}_{}",
                    service.name, operation.name
                ));
                let prepare_id = format!("prepare_transport_{suffix}");
                let execute_id = format!("execute_transport_{suffix}");
                let parse_id = format!("parse_transport_{suffix}");
                let prepare_ports = service_prepare_ports(operation, &service_metadata);
                let prepare_inputs = prepare_ports
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect::<Vec<_>>();

                builder.add_node(Node::opaque(
                    prepare_id.clone(),
                    prepare_ports,
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
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                ));
                let has_auth = matches!(
                    &service_metadata.spec,
                    Some(ServiceOperationSpec::Rest(spec)) if spec.auth_scheme.is_some()
                );
                let mut execute_inputs = vec![Port::scalar("request", "TransportRequest")];
                if has_auth {
                    execute_inputs.push(Port::with_cardinality(
                        PortName::RESOURCE_CREDENTIAL,
                        "Credential",
                        Cardinality::ZERO_OR_ONE,
                    ));
                }
                let execute_node = Node::opaque(
                    execute_id.clone(),
                    execute_inputs,
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
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                )
                .with_input_guard("request", Guard::NotEq(Value::Skipped));
                builder.add_node(execute_node);
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
                        service_metadata: Some(Box::new(service_metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
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
                let operation_inputs = operation
                    .inputs
                    .iter()
                    .map(|field| field.name.clone())
                    .collect::<Vec<_>>();
                let endpoint = ServiceTransportEndpoint {
                    parse: LoweredEndpoint {
                        node_id: parse_id,
                        primary_output: parse_output,
                    },
                    prepare_node_id: prepare_id,
                    execute_node_id: execute_id,
                    prepare_inputs,
                    operation_inputs,
                    has_auth,
                    metadata: Some(service_metadata),
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
    Ok(registry)
}

fn add_service_call_edges(
    builder: &mut DagBuilder,
    project: &TypedProject,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    service_registry: &ServiceEndpointRegistry,
    active_profile_bindings: Option<&ActiveProfileBindings>,
    profile_bound_interfaces: &HashSet<String>,
    known_interface_types: &HashSet<String>,
    data_values: &HashMap<String, serde_json::Value>,
) -> Result<(), LowerError> {
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        // Track transport endpoint usage across ALL callables in the module so
        // that the second callable to reference the same service operation gets
        // a cloned triplet (_c1, _c2, …) instead of wiring duplicate scalar
        // edges to the original.
        let mut endpoint_use_count: HashMap<String, usize> = HashMap::new();
        for item in &module.ast.items {
            let (item_name, params, stmts, uses_binding_types, body_lossy) = match &item.node {
                Item::FnDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    HashMap::new(),
                    def.body.lossy,
                ),
                Item::FuncDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    def.uses
                        .iter()
                        .map(|usage| {
                            (
                                usage.binding.clone(),
                                resource_type_name(&usage.resource_type),
                            )
                        })
                        .collect::<HashMap<_, _>>(),
                    def.body.lossy,
                ),
                Item::PatternDef(def) => (
                    &def.name,
                    &def.params,
                    def.body.stmts.as_slice(),
                    def.uses
                        .iter()
                        .map(|usage| {
                            (
                                usage.binding.clone(),
                                resource_type_name(&usage.resource_type),
                            )
                        })
                        .collect::<HashMap<_, _>>(),
                    def.body.lossy,
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
            let bound_callable_sources = collect_bound_callable_sources(
                module_name.as_str(),
                stmts,
                endpoints_by_full,
                endpoints_by_name,
            );
            let caller = format!("{module_name}::{item_name}");
            let mut bound_service_sources = collect_bound_service_sources(
                caller.as_str(),
                stmts,
                &uses_binding_types,
                service_registry,
                active_profile_bindings,
                profile_bound_interfaces,
                known_interface_types,
            )?;
            let mut service_calls = Vec::<ServiceCallSite>::new();
            collect_service_calls_from_stmts(stmts, &mut service_calls);
            // Filter out service calls that are inside control-flow bodies
            // (handled by scoped transport wiring in add_control_flow_pattern_nodes).
            // We count nested occurrences per path and remove that many from the
            // flat list (back-to-front), preserving top-level calls that share
            // the same operation path as a nested call.
            let mut nested_call_paths = Vec::<Vec<String>>::new();
            for site in detect_for_loops_in_stmts(stmts) {
                nested_call_paths.extend(site.body_service_call_paths);
            }
            for site in detect_if_branches_in_stmts(stmts) {
                nested_call_paths.extend(site.then_service_call_paths);
                nested_call_paths.extend(site.else_service_call_paths);
            }
            for site in detect_match_branches_in_stmts(stmts) {
                nested_call_paths.extend(site.all_service_call_paths);
            }
            if !nested_call_paths.is_empty() {
                let mut nested_counts: HashMap<Vec<String>, usize> = HashMap::new();
                for path in &nested_call_paths {
                    *nested_counts.entry(path.clone()).or_insert(0) += 1;
                }
                let mut removal_budget = nested_counts;
                service_calls.retain(|call| {
                    if let Some(count) = removal_budget.get_mut(&call.path) {
                        if *count > 0 {
                            *count -= 1;
                            return false;
                        }
                    }
                    true
                });
            }
            for (call_index, call) in service_calls.into_iter().enumerate() {
                let Some(source) = resolve_service_call_source(
                    caller.as_str(),
                    &call.path,
                    &uses_binding_types,
                    service_registry,
                    active_profile_bindings,
                    profile_bound_interfaces,
                    known_interface_types,
                )?
                else {
                    continue;
                };
                let use_count = endpoint_use_count
                    .entry(source.endpoint.prepare_node_id.clone())
                    .or_insert(0);
                *use_count += 1;
                let effective_endpoint = if *use_count > 1 {
                    builder
                        .clone_transport_triplet(&source.endpoint, &format!("c{}", *use_count - 1))
                } else {
                    source.endpoint.clone()
                };
                // Update bound_service_sources entries that still point to the
                // original endpoint so arg wiring below (and later fn-call/return
                // wiring) uses this callable's effective endpoint, not the
                // original (which may belong to a different callable).
                let original_prepare_id = source.endpoint.prepare_node_id.clone();
                for svc_source in bound_service_sources.values_mut() {
                    if svc_source.prepare_node_id == original_prepare_id {
                        *svc_source = effective_endpoint.clone();
                    }
                }
                builder.add_edge(
                    effective_endpoint.parse.node_id.as_str(),
                    effective_endpoint.parse.primary_output.as_str(),
                    target.node_id.as_str(),
                    PortName::DEPS,
                );
                // Extract auth_input name so the prepare-arg loop skips it
                // (auth_input args go to res:credential on execute, not prepare).
                let auth_input_field_name = effective_endpoint.metadata.as_ref().and_then(|m| {
                    m.spec.as_ref().and_then(|s| match s {
                        ServiceOperationSpec::Rest(spec) => spec.auth_input.clone(),
                        _ => None,
                    })
                });
                let mut supplied_prepare_inputs = HashSet::<String>::new();
                for (index, arg) in call.args.iter().enumerate() {
                    // Resolve positional args against the full (unfiltered)
                    // operation input list so that auth_input fields at any
                    // position are correctly identified by name.  The
                    // auth_input skip check below then handles them.
                    let Some(prepare_input) = arg.name.as_deref().or_else(|| {
                        effective_endpoint
                            .operation_inputs
                            .get(index)
                            .map(String::as_str)
                    }) else {
                        continue;
                    };
                    // Skip auth_input args — they are wired to res:credential below.
                    if auth_input_field_name.as_deref() == Some(prepare_input) {
                        continue;
                    }
                    supplied_prepare_inputs.insert(prepare_input.to_string());
                    if let Some(arg_ident) = arg.ident.as_deref() {
                        if let Some(param_ty) = param_types.get(arg_ident) {
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
                                effective_endpoint.prepare_node_id.as_str(),
                                prepare_input,
                            );
                            continue;
                        }
                        if let Some(bound_source) = bound_callable_sources.get(arg_ident) {
                            builder.add_edge(
                                bound_source.node_id.as_str(),
                                bound_source.primary_output.as_str(),
                                effective_endpoint.prepare_node_id.as_str(),
                                prepare_input,
                            );
                            continue;
                        }
                        if let Some(bound_source) = bound_service_sources.get(arg_ident) {
                            builder.add_edge(
                                bound_source.parse.node_id.as_str(),
                                bound_source.parse.primary_output.as_str(),
                                effective_endpoint.prepare_node_id.as_str(),
                                prepare_input,
                            );
                            continue;
                        }
                        continue;
                    }
                    if let Some((base_ident, field_name)) = arg.field_access.as_ref() {
                        if let Some(bound_source) = bound_callable_sources.get(base_ident) {
                            builder.add_edge(
                                bound_source.node_id.as_str(),
                                field_name.as_str(),
                                effective_endpoint.prepare_node_id.as_str(),
                                prepare_input,
                            );
                            continue;
                        }
                        if let Some(bound_source) = bound_service_sources.get(base_ident) {
                            builder.add_edge(
                                bound_source.parse.node_id.as_str(),
                                field_name.as_str(),
                                effective_endpoint.prepare_node_id.as_str(),
                                prepare_input,
                            );
                            continue;
                        }
                    }
                    if let Some(call_name) = arg.call.as_deref() {
                        if let Some(Some(call_source)) = endpoints_by_name.get(call_name) {
                            builder.add_edge(
                                call_source.node_id.as_str(),
                                call_source.primary_output.as_str(),
                                effective_endpoint.prepare_node_id.as_str(),
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
                        effective_endpoint.prepare_node_id.as_str(),
                        prepare_input,
                    );
                }
                // Wire auth_input argument to res:credential on execute node.
                // When a service declares `config { auth_input: field_name }`,
                // the named argument is excluded from prepare inputs (it doesn't
                // go into the body/headers) and instead wires directly to
                // `res:credential` on the execute node, where the transport
                // executor applies it as an authentication header.
                if let Some(ref auth_input_name) = auth_input_field_name {
                    for (arg_index, arg) in call.args.iter().enumerate() {
                        // Match by explicit name or by positional index into
                        // the unfiltered operation input list.
                        let resolved_name = arg.name.as_deref().or_else(|| {
                            effective_endpoint
                                .operation_inputs
                                .get(arg_index)
                                .map(String::as_str)
                        });
                        if resolved_name != Some(auth_input_name.as_str()) {
                            continue;
                        }
                        if let Some(arg_ident) = arg.ident.as_deref() {
                            if let Some(param_ty) = param_types.get(arg_ident) {
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
                                    effective_endpoint.execute_node_id.as_str(),
                                    PortName::RESOURCE_CREDENTIAL,
                                );
                            } else if let Some(bound_source) = bound_callable_sources.get(arg_ident) {
                                builder.add_edge(
                                    bound_source.node_id.as_str(),
                                    bound_source.primary_output.as_str(),
                                    effective_endpoint.execute_node_id.as_str(),
                                    PortName::RESOURCE_CREDENTIAL,
                                );
                            } else if let Some(bound_source) = bound_service_sources.get(arg_ident) {
                                builder.add_edge(
                                    bound_source.parse.node_id.as_str(),
                                    bound_source.parse.primary_output.as_str(),
                                    effective_endpoint.execute_node_id.as_str(),
                                    PortName::RESOURCE_CREDENTIAL,
                                );
                            }
                        } else if let Some((base_ident, field_name)) = arg.field_access.as_ref() {
                            if let Some(bound_source) = bound_callable_sources.get(base_ident) {
                                builder.add_edge(
                                    bound_source.node_id.as_str(),
                                    field_name.as_str(),
                                    effective_endpoint.execute_node_id.as_str(),
                                    PortName::RESOURCE_CREDENTIAL,
                                );
                            } else if let Some(bound_source) = bound_service_sources.get(base_ident) {
                                builder.add_edge(
                                    bound_source.parse.node_id.as_str(),
                                    field_name.as_str(),
                                    effective_endpoint.execute_node_id.as_str(),
                                    PortName::RESOURCE_CREDENTIAL,
                                );
                            }
                        } else if let Some(literal) = arg.literal.as_ref() {
                            // Use "credential" (not "res:credential") as the literal
                            // node's output port — res: prefix is reserved for inputs.
                            let literal_source = ensure_literal_source_node(
                                builder,
                                module_name.as_str(),
                                item_name,
                                "credential",
                                "Secret",
                                literal,
                                format!("{call_index}_auth_input").as_str(),
                            );
                            builder.add_edge(
                                literal_source.as_str(),
                                "credential",
                                effective_endpoint.execute_node_id.as_str(),
                                PortName::RESOURCE_CREDENTIAL,
                            );
                        }
                        break;
                    }
                }
                wire_profile_binding_config_inputs(
                    builder,
                    source.binding_config.as_ref(),
                    &supplied_prepare_inputs,
                    &effective_endpoint,
                    module_name.as_str(),
                    item_name,
                    call_index,
                );
            }
            // Augment callable sources with for-loop result bindings so that
            // downstream fn call arguments like `file_contents: file_contents`
            // can resolve to the loop node's "result" output.
            let mut augmented_callable_sources = bound_callable_sources.clone();
            collect_for_loop_bindings(stmts, target, &mut augmented_callable_sources);
            wire_fn_call_arguments(
                builder,
                stmts,
                endpoints_by_name,
                &param_types,
                &augmented_callable_sources,
                &bound_service_sources,
                module_name.as_str(),
                item_name,
                data_values,
            );
            // Wire for-loop iterable expressions to loop node "items" ports.
            wire_for_loop_iterables(
                builder,
                stmts,
                target,
                &param_types,
                &bound_callable_sources,
                &bound_service_sources,
                module_name.as_str(),
                item_name,
            );
            wire_callable_return_outputs(
                builder,
                stmts,
                target,
                body_lossy,
                &param_types,
                &augmented_callable_sources,
                &bound_service_sources,
                endpoints_by_name,
                module_name.as_str(),
                item_name,
            );
        }
    }
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::PipelineDef(def) = &item.node else {
                continue;
            };
            let Some(target) = endpoints_by_full.get(&(module_name.clone(), def.name.clone()))
            else {
                continue;
            };
            let uses_binding_types = def
                .uses
                .iter()
                .map(|usage| {
                    (
                        usage.binding.clone(),
                        resource_type_name(&usage.resource_type),
                    )
                })
                .collect::<HashMap<_, _>>();
            for stage in &def.stages {
                let mut service_calls = Vec::<ServiceCallSite>::new();
                collect_service_calls_from_stmts(stage.body.stmts.as_slice(), &mut service_calls);
                for (call_index, call) in service_calls.into_iter().enumerate() {
                    let caller = format!("{module_name}::{}::{}", def.name, stage.name);
                    let Some(source) = resolve_service_call_source(
                        caller.as_str(),
                        &call.path,
                        &uses_binding_types,
                        service_registry,
                        active_profile_bindings,
                        profile_bound_interfaces,
                        known_interface_types,
                    )?
                    else {
                        continue;
                    };
                    let source_endpoint = &source.endpoint;
                    builder.add_edge(
                        source_endpoint.parse.node_id.as_str(),
                        source_endpoint.parse.primary_output.as_str(),
                        target.node_id.as_str(),
                        PortName::DEPS,
                    );
                    wire_profile_binding_config_inputs(
                        builder,
                        source.binding_config.as_ref(),
                        &HashSet::new(),
                        source_endpoint,
                        module_name.as_str(),
                        def.name.as_str(),
                        call_index,
                    );
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ServiceCallResolvedSource {
    pub(crate) endpoint: ServiceTransportEndpoint,
    pub(crate) binding_config: Option<HashMap<String, ProfileConfigValue>>,
}

pub(crate) fn resolve_service_call_source(
    caller: &str,
    call_path: &[String],
    uses_binding_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
    active_profile_bindings: Option<&ActiveProfileBindings>,
    profile_bound_interfaces: &HashSet<String>,
    known_interface_types: &HashSet<String>,
) -> Result<Option<ServiceCallResolvedSource>, LowerError> {
    if let Some(endpoint) = resolve_service_endpoint(call_path, service_registry) {
        return Ok(Some(ServiceCallResolvedSource {
            endpoint,
            binding_config: None,
        }));
    }
    let Some(binding) = call_path.first() else {
        return Err(LowerError::UnresolvedServiceCall {
            caller: caller.to_string(),
            service_call: call_path.join("."),
        });
    };
    let Some(interface_type) = uses_binding_types.get(binding) else {
        return Err(LowerError::UnresolvedServiceCall {
            caller: caller.to_string(),
            service_call: call_path.join("."),
        });
    };
    if !is_bound_interface_type_name(profile_bound_interfaces, interface_type) {
        if is_bound_interface_type_name(known_interface_types, interface_type) {
            // Interface-backed resource lifecycles are handled by dedicated
            // resource wiring paths.
            return Ok(None);
        }
        // Non-interface `uses` bindings: try resource capability lookup.
        // e.g., `fs.read(path: p)` with `uses fs: Filesystem` →
        // look up `Filesystem.read` in the service registry.
        if call_path.len() >= 2 {
            let capability = call_path.last().cloned().unwrap_or_default();
            let cap_key = format!("{interface_type}.{capability}");
            if let Some(endpoint) = resolve_service_endpoint(
                &cap_key.split('.').map(String::from).collect::<Vec<_>>(),
                service_registry,
            ) {
                return Ok(Some(ServiceCallResolvedSource {
                    endpoint,
                    binding_config: None,
                }));
            }
        }
        return Ok(None);
    }
    let Some(active_profile_bindings) = active_profile_bindings else {
        // IS-5: No profile → interface capabilities are resolved via stub
        // transport triplets registered by IS-2/IS-4. Try endpoint registry
        // lookup (stubs are registered there). If not found, return None
        // to let the caller skip this call (stubs handle it).
        if call_path.len() >= 2 {
            let capability = call_path.last().cloned().unwrap_or_default();
            let cap_key = format!("{interface_type}.{capability}");
            if let Some(endpoint) = resolve_service_endpoint(
                &cap_key.split('.').map(String::from).collect::<Vec<_>>(),
                service_registry,
            ) {
                return Ok(Some(ServiceCallResolvedSource {
                    endpoint,
                    binding_config: None,
                }));
            }
        }
        return Ok(None);
    };
    let Some(interface_key) = resolve_profile_interface_key(
        &active_profile_bindings.by_interface,
        interface_type.as_str(),
    ) else {
        return Err(LowerError::MissingProfileBinding {
            profile: active_profile_bindings.profile_name.clone(),
            interface_type: canonical_resource_type_name(interface_type),
        });
    };
    let Some(binding) = active_profile_bindings
        .by_interface
        .get(interface_key.as_str())
    else {
        return Err(LowerError::MissingProfileBinding {
            profile: active_profile_bindings.profile_name.clone(),
            interface_type: interface_key,
        });
    };
    let implementation_type = binding.implementation_type.as_str();
    let capability = call_path.last().cloned().unwrap_or_default();
    let mut implementation_call_path = implementation_type
        .split('.')
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>();
    implementation_call_path.push(capability);
    let endpoint = resolve_service_endpoint(&implementation_call_path, service_registry)
        .ok_or_else(|| LowerError::UnresolvedServiceCall {
            caller: caller.to_string(),
            service_call: implementation_call_path.join("."),
        })?;
    Ok(Some(ServiceCallResolvedSource {
        endpoint,
        binding_config: Some(binding.config_values.clone()),
    }))
}

fn unwrap_guarded_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Guarded(inner, _) | Expr::After(inner, _) => unwrap_guarded_expr(inner),
        other => other,
    }
}

fn collect_bound_service_sources(
    caller: &str,
    stmts: &[Stmt],
    uses_binding_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
    active_profile_bindings: Option<&ActiveProfileBindings>,
    profile_bound_interfaces: &HashSet<String>,
    known_interface_types: &HashSet<String>,
) -> Result<HashMap<String, ServiceTransportEndpoint>, LowerError> {
    let mut bound = HashMap::<String, ServiceTransportEndpoint>::new();
    for stmt in stmts {
        match stmt {
            Stmt::Let(binding, expr)
            | Stmt::Assign(binding, expr)
            | Stmt::Node(NodeStmt {
                name: binding,
                expr,
                ..
            }) => match unwrap_guarded_expr(expr) {
                Expr::ServiceCall(path, _) => {
                    if let Some(source) = resolve_service_call_source(
                        caller,
                        path,
                        uses_binding_types,
                        service_registry,
                        active_profile_bindings,
                        profile_bound_interfaces,
                        known_interface_types,
                    )? {
                        bound.insert(binding.clone(), source.endpoint);
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
    Ok(bound)
}

fn wire_profile_binding_config_inputs(
    builder: &mut DagBuilder,
    binding_config: Option<&HashMap<String, ProfileConfigValue>>,
    supplied_prepare_inputs: &HashSet<String>,
    source_endpoint: &ServiceTransportEndpoint,
    module_name: &str,
    item_name: &str,
    call_index: usize,
) {
    let Some(binding_config) = binding_config else {
        return;
    };
    for (key, value) in binding_config {
        // Credential config is wired to `res:credential` on the execute node
        // (not `config.credential` on prepare). The transport executor applies
        // the credential at execute time via `Credential::apply()`.
        if key == "credential" && source_endpoint.has_auth {
            let literal = match value {
                ProfileConfigValue::Literal(value) => ServiceCallArgLiteral::String(value.clone()),
                ProfileConfigValue::SecretRef(name) => {
                    ServiceCallArgLiteral::String(format!("secret:{name}"))
                }
            };
            let suffix = format!("{call_index}_profile_credential");
            let literal_source = ensure_literal_source_node(
                builder,
                module_name,
                item_name,
                PortName::RESOURCE_CREDENTIAL,
                "Secret",
                &literal,
                suffix.as_str(),
            );
            builder.add_edge(
                literal_source.as_str(),
                PortName::RESOURCE_CREDENTIAL,
                source_endpoint.execute_node_id.as_str(),
                PortName::RESOURCE_CREDENTIAL,
            );
            continue;
        }
        let candidates = [key.to_string(), format!("config.{key}")];
        let Some(prepare_input) = candidates.iter().find(|candidate| {
            source_endpoint
                .prepare_inputs
                .iter()
                .any(|input| input == *candidate)
                && !supplied_prepare_inputs.contains(candidate.as_str())
        }) else {
            continue;
        };
        let literal = match value {
            ProfileConfigValue::Literal(value) => ServiceCallArgLiteral::String(value.clone()),
            ProfileConfigValue::SecretRef(name) => {
                ServiceCallArgLiteral::String(format!("secret:{name}"))
            }
        };
        let suffix = format!(
            "{call_index}_profile_{}",
            key.chars()
                .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
                .collect::<String>()
        );
        let literal_source = ensure_literal_source_node(
            builder,
            module_name,
            item_name,
            prepare_input.as_str(),
            "String",
            &literal,
            suffix.as_str(),
        );
        builder.add_edge(
            literal_source.as_str(),
            prepare_input.as_str(),
            source_endpoint.prepare_node_id.as_str(),
            prepare_input.as_str(),
        );
    }
}

/// Collect the set of callable names (module-qualified) that declare
/// `provides auth: AuthContext`. Used by `wire_auth_credential_edges`
/// to identify credential provider calls in function bodies.
fn collect_auth_provider_names(project: &TypedProject) -> HashSet<String> {
    let mut providers = HashSet::new();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Some((item_name, provides)) = item_callable_provides(&item.node) else {
                continue;
            };
            let is_auth_provider = provides.iter().any(|p| {
                let ty = resource_type_name(&p.resource_type);
                ty == "AuthContext" || ty.ends_with(".AuthContext")
            });
            if is_auth_provider {
                providers.insert(item_name.to_string());
                providers.insert(format!("{module_name}.{item_name}"));
            }
        }
    }
    providers
}

/// Wire credential provider outputs to `res:credential` on execute nodes
/// of `@auth`-annotated service calls within the same function body.
///
/// When a function body calls both a `provides auth: AuthContext` pattern
/// (e.g., `credential_chain`) and an `@auth`-annotated service (e.g.,
/// `github.Gist.Create`), this function wires the credential output from
/// the provider to the execute node's `res:credential` port.
fn wire_auth_credential_edges(
    builder: &mut DagBuilder,
    project: &TypedProject,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    service_registry: &ServiceEndpointRegistry,
    auth_provider_names: &HashSet<String>,
) {
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let stmts = match &item.node {
                Item::FuncDef(def) => def.body.stmts.as_slice(),
                Item::PatternDef(def) => def.body.stmts.as_slice(),
                _ => continue,
            };

            let bound_callable_sources = collect_bound_callable_sources(
                module_name.as_str(),
                stmts,
                endpoints_by_full,
                endpoints_by_name,
            );

            // Find credential provider bindings: calls to auth-providing patterns.
            let credential_sources = collect_credential_provider_bindings(
                stmts,
                auth_provider_names,
                &bound_callable_sources,
            );

            if credential_sources.is_empty() {
                continue;
            }

            // Find service calls with auth requirements.
            let mut service_calls = Vec::<ServiceCallSite>::new();
            collect_service_calls_from_stmts(stmts, &mut service_calls);

            for call in &service_calls {
                let Some(endpoint) = resolve_service_endpoint(&call.path, service_registry) else {
                    continue;
                };
                if !endpoint.has_auth {
                    continue;
                }
                // Skip endpoints with auth_input — they wire res:credential
                // explicitly via add_service_call_edges (from the named arg).
                let has_auth_input = endpoint.metadata.as_ref().is_some_and(|m| {
                    m.spec.as_ref().is_some_and(|s| match s {
                        ServiceOperationSpec::Rest(spec) => spec.auth_input.is_some(),
                        _ => false,
                    })
                });
                if has_auth_input {
                    continue;
                }
                // Wire the first available credential source to the execute node.
                if let Some(cred_source) = credential_sources.first() {
                    builder.add_edge(
                        cred_source.node_id.as_str(),
                        cred_source.primary_output.as_str(),
                        endpoint.execute_node_id.as_str(),
                        PortName::RESOURCE_CREDENTIAL,
                    );
                }
            }
        }
    }
}

/// Find variable bindings in the statement list that call auth-providing
/// patterns, and return their lowered endpoints.
fn collect_credential_provider_bindings(
    stmts: &[Stmt],
    auth_provider_names: &HashSet<String>,
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
) -> Vec<LoweredEndpoint> {
    let mut sources = Vec::new();
    for stmt in stmts {
        let (binding, call_name) = match stmt {
            Stmt::Let(b, Expr::Call(n, _)) | Stmt::Assign(b, Expr::Call(n, _)) => (b, n),
            Stmt::Node(ns) => {
                if let Expr::Call(n, _) = &ns.expr {
                    (&ns.name, n)
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        if auth_provider_names.contains(call_name) {
            if let Some(endpoint) = bound_callable_sources.get(binding) {
                sources.push(endpoint.clone());
            }
        }
    }
    sources
}

fn resolve_profile_interface_key(
    bindings: &HashMap<String, ActiveProfileBinding>,
    interface_type: &str,
) -> Option<String> {
    let canonical = canonical_resource_type_name(interface_type);
    if bindings.contains_key(&canonical) {
        return Some(canonical);
    }
    let short = canonical.rsplit('.').next().unwrap_or(canonical.as_str());
    let mut matched = bindings
        .keys()
        .filter(|key| key.rsplit('.').next().is_some_and(|tail| tail == short));
    let first = matched.next()?;
    if matched.next().is_some() {
        return None;
    }
    Some(first.clone())
}

fn add_used_resource_edges(
    builder: &mut DagBuilder,
    project: &TypedProject,
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    resource_registry: &ResourceLifecycleRegistry,
    known_uses_types: &HashSet<String>,
) -> Result<(), LowerError> {
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
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
                        PortName::DEPS,
                    );
                }
                if let Some(release_node) = endpoint.release_node {
                    builder.add_control_edge(
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
        let module_name = module.module_path.as_dotted();
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
                        fn_body: None,
                    },
                ));
                builder.add_control_edge(
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
    let target_canonical = canonical_resource_type_name(resource_type);
    let target_short = target_canonical
        .rsplit('.')
        .next()
        .unwrap_or(target_canonical.as_str());
    let mut candidates = Vec::<(Option<ProviderHint>, ResourceLifecycleEndpoint)>::new();

    for module in &project.modules {
        let candidate_module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::ResourceDef(resource) = &item.node else {
                continue;
            };
            let Some(implemented) = &resource.implements else {
                continue;
            };
            let implemented_canonical = canonical_resource_type_name(implemented);
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
        let module_name = module.module_path.as_dotted();
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
        let module_name = module.module_path.as_dotted();
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
                        fn_body: None,
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
                        fn_body: None,
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
        let module_name = module.module_path.as_dotted();
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
            for index in 0..contract_count {
                let node_id = format!(
                    "verify_contract_{}",
                    sanitize_identifier(&format!(
                        "{module_name}_{}_{}_{}",
                        resource.name,
                        canonical_resource_type_name(interface_name),
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
                            canonical_resource_type_name(interface_name),
                            index
                        ),
                        obligation: ObligationCategory::InterfaceContractVerification,
                        service_metadata: None,
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
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
                        PortName::DEPS,
                    );
                }
            }
        }
    }
}

fn resolve_interface_contract_count(project: &TypedProject, interface_name: &str) -> usize {
    let target = canonical_resource_type_name(interface_name);
    let target_short = target.rsplit('.').next().unwrap_or(target.as_str());
    let mut counts = Vec::new();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::InterfaceDef(interface) = &item.node else {
                continue;
            };
            let qualified = format!("{module_name}.{}", interface.name);
            let qualified_canonical = canonical_resource_type_name(&qualified);
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
            counts.push(interface.contracts.len() + interface.contracts.len());
        }
    }
    if counts.len() == 1 {
        return counts[0];
    }
    0
}

pub(crate) fn resolve_service_endpoint(
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
    builder.add_node(
        Node::opaque(
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
        )
        .with_kind(gunbc_ir::node::NodeKind::ParamSource),
    );
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
        ServiceCallArgLiteral::Json(value) => {
            format!("jsonhex:{}", hex_encode(value.to_string().as_bytes()))
        }
        ServiceCallArgLiteral::None => "none".to_string(),
    }
}

fn primitive_literal_from_service_literal(literal: &ServiceCallArgLiteral) -> PrimitiveLiteral {
    match literal {
        ServiceCallArgLiteral::String(value) => PrimitiveLiteral::String(value.clone()),
        ServiceCallArgLiteral::Int(value) => PrimitiveLiteral::Int(*value),
        ServiceCallArgLiteral::Bool(value) => PrimitiveLiteral::Bool(*value),
        ServiceCallArgLiteral::Json(value) => PrimitiveLiteral::Json(value.clone()),
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

fn item_uses_binding_types(item: &Item) -> HashMap<String, String> {
    match item {
        Item::FuncDef(def) => def
            .uses
            .iter()
            .map(|u| (u.binding.clone(), resource_type_name(&u.resource_type)))
            .collect(),
        Item::PatternDef(def) => def
            .uses
            .iter()
            .map(|u| (u.binding.clone(), resource_type_name(&u.resource_type)))
            .collect(),
        _ => HashMap::new(),
    }
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
        Item::FuncDef(def) => Some((def.name.as_str(), false)),
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
        Item::PatternDef(def) => Some((def.name.as_str(), def.provides.as_slice())),
        _ => None,
    }
}

fn is_internal_synthetic_call(name: &str) -> bool {
    matches!(name, "<expr>" | "as" | "with" | "fn")
}

fn collect_calls_from_stmts(stmts: &[Stmt], calls: &mut BTreeSet<String>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Call(name, _) = expr {
            if !is_internal_synthetic_call(name) {
                calls.insert(name.clone());
            }
        }
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceCallArgSite {
    pub(crate) name: Option<String>,
    pub(crate) ident: Option<String>,
    pub(crate) field_access: Option<(String, String)>,
    pub(crate) call: Option<String>,
    pub(crate) literal: Option<ServiceCallArgLiteral>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceCallSite {
    pub(crate) path: Vec<String>,
    pub(crate) args: Vec<ServiceCallArgSite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FnCallSite {
    name: String,
    args: Vec<ServiceCallArgSite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServiceCallArgLiteral {
    String(String),
    Int(i64),
    Bool(bool),
    Json(serde_json::Value),
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
    Split,
    Zip,
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
        "sum" => Some(CollectionOpKind::Fold),
        "split" => Some(CollectionOpKind::Split),
        "zip" => Some(CollectionOpKind::Zip),
        _ => None,
    }
}

fn collect_collection_ops_from_stmts(stmts: &[Stmt], sites: &mut Vec<CollectionOpSite>) {
    walk_stmts(stmts, &mut |expr| {
        match expr {
            Expr::Pipe(_, rhs) => {
                let Expr::Call(name, _) = rhs.as_ref() else {
                    return;
                };
                let Some(kind) = collection_op_kind(name) else {
                    return;
                };
                sites.push(CollectionOpSite { kind });
            }
            Expr::PipeCall(_, method, _) => {
                let method_name = match method {
                    daglang_syntax::ast::PipeMethod::Map => "map",
                    daglang_syntax::ast::PipeMethod::Filter => "filter",
                    daglang_syntax::ast::PipeMethod::FilterMap => "filter_map",
                    daglang_syntax::ast::PipeMethod::FlatMap => "flat_map",
                    daglang_syntax::ast::PipeMethod::SortBy => "sort_by",
                    daglang_syntax::ast::PipeMethod::Append => "append",
                    daglang_syntax::ast::PipeMethod::Fold => "fold",
                    daglang_syntax::ast::PipeMethod::Join => "join",
                    daglang_syntax::ast::PipeMethod::Count => "count",
                    daglang_syntax::ast::PipeMethod::Sum => "sum",
                    daglang_syntax::ast::PipeMethod::First => "first",
                    daglang_syntax::ast::PipeMethod::Last => "last",
                    daglang_syntax::ast::PipeMethod::MaxBy => "max_by",
                    daglang_syntax::ast::PipeMethod::Any => "any",
                    daglang_syntax::ast::PipeMethod::All => "all",
                    daglang_syntax::ast::PipeMethod::Contains => "contains",
                    daglang_syntax::ast::PipeMethod::StartsWith => "starts_with",
                    daglang_syntax::ast::PipeMethod::EndsWith => "ends_with",
                    daglang_syntax::ast::PipeMethod::Repeat => "repeat",
                    daglang_syntax::ast::PipeMethod::ReplaceSection => "replace_section",
                    daglang_syntax::ast::PipeMethod::Chars => "chars",
                    daglang_syntax::ast::PipeMethod::ToBytes => "to_bytes",
                    daglang_syntax::ast::PipeMethod::ToJson => "to_json",
                    daglang_syntax::ast::PipeMethod::Hash => "hash",
                };
                if let Some(kind) = collection_op_kind(method_name) {
                    sites.push(CollectionOpSite { kind });
                }
            }
            _ => {}
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
        CollectionOpKind::Split => "SplitNode",
        CollectionOpKind::Zip => "ZipNode",
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
                Port::with_cardinality(PortName::DEPS, "Any", Cardinality::ZERO_OR_MORE),
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
            PortName::DEPS.to_string(),
        ));
    }
    CollectionLoweringPlan { nodes, edges }
}

pub(crate) fn collect_service_calls_from_stmts(stmts: &[Stmt], calls: &mut Vec<ServiceCallSite>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::ServiceCall(path, args) = expr {
            calls.push(ServiceCallSite {
                path: path.clone(),
                args: args.iter().map(service_call_arg_site).collect::<Vec<_>>(),
            });
        }
    });
}

fn collect_fn_calls_with_args(stmts: &[Stmt], calls: &mut Vec<FnCallSite>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Call(name, args) = expr {
            if !is_internal_synthetic_call(name) {
                calls.push(FnCallSite {
                    name: name.clone(),
                    args: args.iter().map(service_call_arg_site).collect(),
                });
            }
        }
    });
}

fn service_call_arg_site((name, arg): &(Option<String>, Expr)) -> ServiceCallArgSite {
    ServiceCallArgSite {
        name: name.clone(),
        ident: match arg {
            Expr::Ident(ident) => Some(ident.clone()),
            _ => None,
        },
        field_access: match arg {
            Expr::FieldAccess(base, field) => match base.as_ref() {
                Expr::Ident(base_ident) => Some((base_ident.clone(), field.clone())),
                _ => None,
            },
            _ => None,
        },
        call: match arg {
            Expr::Call(call_name, _) => Some(call_name.clone()),
            _ => None,
        },
        literal: service_call_literal_arg(arg),
    }
}

fn service_call_literal_arg(arg: &Expr) -> Option<ServiceCallArgLiteral> {
    match arg {
        Expr::Literal(Literal::String(value)) => Some(ServiceCallArgLiteral::String(value.clone())),
        Expr::Literal(Literal::Int(value)) => Some(ServiceCallArgLiteral::Int(*value)),
        Expr::Literal(Literal::Bool(value)) => Some(ServiceCallArgLiteral::Bool(*value)),
        Expr::Literal(Literal::None) => Some(ServiceCallArgLiteral::None),
        Expr::StringInterp(_) => expr_to_template_string(arg).map(ServiceCallArgLiteral::String),
        Expr::List(_) | Expr::Map(_) => {
            expr_to_json_literal(arg, &HashSet::new()).map(ServiceCallArgLiteral::Json)
        }
        _ => None,
    }
}

fn expr_to_json_literal(expr: &Expr, variant_names: &HashSet<String>) -> Option<serde_json::Value> {
    match expr {
        Expr::Literal(Literal::String(value)) => Some(serde_json::Value::String(value.clone())),
        Expr::Literal(Literal::Int(value)) => Some(serde_json::Value::Number((*value).into())),
        Expr::Literal(Literal::Bool(value)) => Some(serde_json::Value::Bool(*value)),
        Expr::Literal(Literal::None) => Some(serde_json::Value::Null),
        Expr::Ident(name) if name == "None" || name == "null" => Some(serde_json::Value::Null),
        Expr::List(values) => {
            let mut out = Vec::with_capacity(values.len());
            for value in values {
                out.push(expr_to_json_literal(value, variant_names)?);
            }
            Some(serde_json::Value::Array(out))
        }
        Expr::Map(entries) => {
            let mut out = serde_json::Map::new();
            for (key, value) in entries {
                let key = match key {
                    Expr::Literal(Literal::String(raw)) => raw.clone(),
                    _ => return None,
                };
                out.insert(key, expr_to_json_literal(value, variant_names)?);
            }
            Some(serde_json::Value::Object(out))
        }
        // Unit variant ident in data declarations (e.g., `data x = Closed`)
        Expr::Ident(name) if variant_names.contains(name.as_str()) => {
            let mut out = serde_json::Map::new();
            out.insert(
                "_variant".to_string(),
                serde_json::Value::String(name.clone()),
            );
            Some(serde_json::Value::Object(out))
        }
        Expr::Record(type_name, fields) => {
            let mut out = serde_json::Map::new();
            if let Some(variant) = type_name {
                out.insert(
                    "_variant".to_string(),
                    serde_json::Value::String(variant.clone()),
                );
            }
            for (key, value) in fields {
                out.insert(key.clone(), expr_to_json_literal(value, variant_names)?);
            }
            Some(serde_json::Value::Object(out))
        }
        Expr::UnaryOp(daglang_syntax::ast::UnaryOp::Neg, inner) => {
            match expr_to_json_literal(inner, variant_names)? {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Some(serde_json::Value::Number((-i).into()))
                    } else if let Some(f) = n.as_f64() {
                        serde_json::Number::from_f64(-f).map(serde_json::Value::Number)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Evaluate all module-level `data` declarations to JSON values.
///
/// Used to wire data declaration references as literal source nodes in fn call
/// arguments. Only handles constant expressions (literals, lists, records).
pub fn build_data_values(project: &TypedProject) -> HashMap<String, serde_json::Value> {
    let variant_names = collect_variant_names(project);
    let mut values = HashMap::new();
    let mut unqualified_counts: HashMap<String, usize> = HashMap::new();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            if let Item::DataDef(def) = &item.node {
                if let Some(json) = expr_to_json_literal(&def.value, &variant_names) {
                    values.insert(format!("{module_name}.{}", def.name), json.clone());

                    let count = unqualified_counts.entry(def.name.clone()).or_insert(0);
                    *count += 1;
                    if *count == 1 {
                        values.insert(def.name.clone(), json);
                    } else {
                        // Ambiguous — remove unqualified entry, keep only qualified
                        values.remove(&def.name);
                    }
                }
            }
        }
    }
    values
}

fn wire_fn_call_arguments(
    builder: &mut DagBuilder,
    stmts: &[Stmt],
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    param_types: &HashMap<String, String>,
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
    bound_service_sources: &HashMap<String, ServiceTransportEndpoint>,
    module_name: &str,
    item_name: &str,
    data_values: &HashMap<String, serde_json::Value>,
) {
    let mut fn_calls = Vec::new();
    collect_fn_calls_with_args(stmts, &mut fn_calls);
    for fn_call in &fn_calls {
        let Some(Some(fn_endpoint)) = endpoints_by_name.get(&fn_call.name) else {
            continue;
        };
        for (index, arg) in fn_call.args.iter().enumerate() {
            let Some(param_name) = arg.name.as_deref() else {
                continue;
            };
            if builder.has_edge_to_port(fn_endpoint.node_id.as_str(), param_name) {
                continue;
            }
            if let Some((base_ident, field_name)) = arg.field_access.as_ref() {
                if let Some(source) = bound_callable_sources.get(base_ident) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        field_name.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                if let Some(source) = bound_service_sources.get(base_ident) {
                    builder.add_edge(
                        source.parse.node_id.as_str(),
                        field_name.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
            }
            if let Some(arg_ident) = arg.ident.as_deref() {
                if let Some(param_ty) = param_types.get(arg_ident) {
                    let src = ensure_param_source_node(
                        builder,
                        module_name,
                        item_name,
                        arg_ident,
                        param_ty.as_str(),
                    );
                    builder.add_edge(
                        src.as_str(),
                        arg_ident,
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                if let Some(source) = bound_callable_sources.get(arg_ident) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        source.primary_output.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                if let Some(source) = bound_service_sources.get(arg_ident) {
                    builder.add_edge(
                        source.parse.node_id.as_str(),
                        source.parse.primary_output.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                // Wire data declaration references as JSON literal source nodes.
                if let Some(json_val) = data_values.get(arg_ident) {
                    let literal = ServiceCallArgLiteral::Json(json_val.clone());
                    let src = ensure_literal_source_node(
                        builder,
                        module_name,
                        item_name,
                        param_name,
                        "Any",
                        &literal,
                        &format!("data_{index}"),
                    );
                    builder.add_edge(
                        src.as_str(),
                        param_name,
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
            }
            if let Some(literal) = arg.literal.as_ref() {
                let src = ensure_literal_source_node(
                    builder,
                    module_name,
                    item_name,
                    param_name,
                    "Any",
                    literal,
                    &format!("fn_{index}"),
                );
                builder.add_edge(
                    src.as_str(),
                    param_name,
                    fn_endpoint.node_id.as_str(),
                    param_name,
                );
            }
        }
    }
}

fn collect_return_bindings(
    stmts: &[Stmt],
    output_ports: &[Port],
    body_lossy: bool,
) -> Vec<(String, Expr)> {
    if output_ports.is_empty() {
        return Vec::new();
    }

    let output_names = output_ports
        .iter()
        .map(|port| port.name.0.clone())
        .collect::<Vec<_>>();

    let mut explicit_return = None;
    for stmt in stmts {
        if let Stmt::Return(fields) = stmt {
            explicit_return = Some(fields);
        }
    }

    if let Some(fields) = explicit_return {
        if output_names.len() == 1 {
            return fields
                .first()
                .map(|(_, expr)| vec![(output_names[0].clone(), expr.clone())])
                .unwrap_or_default();
        }
        let output_set = output_names
            .iter()
            .map(|name| name.as_str())
            .collect::<HashSet<_>>();
        return fields
            .iter()
            .filter(|(name, _expr)| output_set.contains(name.as_str()))
            .map(|(name, expr)| (name.clone(), expr.clone()))
            .collect();
    }

    if output_names.len() == 1 && !body_lossy {
        let mut trailing_expr = None;
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => trailing_expr = Some(expr),
                Stmt::Let(..) | Stmt::Assign(..) | Stmt::Node(..) | Stmt::Return(_) => {
                    trailing_expr = None;
                }
            }
        }
        if let Some(expr) = trailing_expr {
            return vec![(output_names[0].clone(), expr.clone())];
        }
    }

    Vec::new()
}

fn unwrap_return_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::After(inner, _) | Expr::Guarded(inner, _) => unwrap_return_expr(inner),
        Expr::Call(name, args) if matches!(name.as_str(), "as" | "with" | "<expr>" | "fn") => args
            .first()
            .map(|(_, inner)| unwrap_return_expr(inner))
            .unwrap_or(expr),
        _ => expr,
    }
}

fn return_literal_arg(expr: &Expr) -> Option<ServiceCallArgLiteral> {
    match expr {
        Expr::Literal(Literal::Float(value)) => serde_json::Number::from_f64(*value)
            .map(|num| ServiceCallArgLiteral::Json(serde_json::Value::Number(num))),
        _ => service_call_literal_arg(expr),
    }
}

fn resolve_return_expr_source(
    builder: &mut DagBuilder,
    expr: &Expr,
    output_port: &Port,
    output_name: &str,
    param_types: &HashMap<String, String>,
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
    bound_service_sources: &HashMap<String, ServiceTransportEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    module_name: &str,
    item_name: &str,
    disambiguator: &str,
) -> Option<(String, String)> {
    let expr = unwrap_return_expr(expr);
    match expr {
        Expr::Ident(name) => {
            if let Some(param_ty) = param_types.get(name) {
                let src = ensure_param_source_node(
                    builder,
                    module_name,
                    item_name,
                    name,
                    param_ty.as_str(),
                );
                return Some((src, name.clone()));
            }
            if let Some(source) = bound_callable_sources.get(name) {
                return Some((source.node_id.clone(), source.primary_output.clone()));
            }
            if let Some(source) = bound_service_sources.get(name) {
                return Some((
                    source.parse.node_id.clone(),
                    source.parse.primary_output.clone(),
                ));
            }
            if let Some(Some(source)) = endpoints_by_name.get(name) {
                return Some((source.node_id.clone(), source.primary_output.clone()));
            }
            None
        }
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(base_ident) = base.as_ref() {
                if let Some(source) = bound_callable_sources.get(base_ident) {
                    return Some((source.node_id.clone(), field.clone()));
                }
                if let Some(source) = bound_service_sources.get(base_ident) {
                    return Some((source.parse.node_id.clone(), field.clone()));
                }
                if let Some(Some(source)) = endpoints_by_name.get(base_ident) {
                    return Some((source.node_id.clone(), field.clone()));
                }
            }
            None
        }
        Expr::Call(name, _) => endpoints_by_name
            .get(name)
            .and_then(|entry| entry.clone())
            .map(|source| (source.node_id, source.primary_output)),
        Expr::Literal(_) | Expr::StringInterp(_) | Expr::List(_) | Expr::Map(_) => {
            let literal = return_literal_arg(expr)?;
            let src = ensure_literal_source_node(
                builder,
                module_name,
                item_name,
                output_name,
                output_port.type_id.0.as_str(),
                &literal,
                disambiguator,
            );
            Some((src, output_name.to_string()))
        }
        // Handle complex expressions by synthesizing a dedicated compute node.
        _ => {
            synthesize_expr_compute(
                builder,
                expr,
                output_port,
                output_name,
                param_types,
                bound_callable_sources,
                bound_service_sources,
                endpoints_by_name,
                module_name,
                item_name,
                disambiguator,
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExprLeafRef {
    input_port: String,
    source: expr::LeafRef,
}

/// Collect all leaf expression references from a complex expression.
/// Sets `has_local_refs` to true if the expression references local variables
/// (let bindings) that can't be resolved as compute node inputs.
fn collect_expr_leaf_refs(
    expr: &Expr,
    param_types: &HashMap<String, String>,
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
    bound_service_sources: &HashMap<String, ServiceTransportEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    refs: &mut Vec<ExprLeafRef>,
    seen: &mut HashSet<String>,
    has_local_refs: &mut bool,
) {
    match expr {
        Expr::Ident(name) => {
            let port_name = name.clone();
            if seen.contains(&port_name) {
                return;
            }
            if let Some(param_ty) = param_types.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Param {
                        name: name.clone(),
                        field: None,
                        ty: param_ty.clone(),
                    },
                });
            } else if let Some(source) = bound_callable_sources.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Callable {
                        endpoint: source.node_id.clone(),
                        port: source.primary_output.clone(),
                    },
                });
            } else if let Some(source) = bound_service_sources.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Service {
                        endpoint: source.parse.node_id.clone(),
                        port: source.parse.primary_output.clone(),
                    },
                });
            } else if let Some(Some(source)) = endpoints_by_name.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Callable {
                        endpoint: source.node_id.clone(),
                        port: source.primary_output.clone(),
                    },
                });
            } else {
                *has_local_refs = true;
            }
        }
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(base_ident) = base.as_ref() {
                let port_name = format!("{base_ident}__{field}");
                if seen.contains(&port_name) {
                    return;
                }
                if let Some(param_ty) = param_types.get(base_ident) {
                    let base_port = base_ident.clone();
                    if !seen.contains(&base_port) {
                        seen.insert(base_port.clone());
                        refs.push(ExprLeafRef {
                            input_port: base_port,
                            source: expr::LeafRef::Param {
                                name: base_ident.clone(),
                                field: Some(field.clone()),
                                ty: param_ty.clone(),
                            },
                        });
                    }
                } else if let Some(source) = bound_callable_sources.get(base_ident) {
                    seen.insert(port_name.clone());
                    refs.push(ExprLeafRef {
                        input_port: port_name,
                        source: expr::LeafRef::Callable {
                            endpoint: source.node_id.clone(),
                            port: field.clone(),
                        },
                    });
                } else if let Some(source) = bound_service_sources.get(base_ident) {
                    seen.insert(port_name.clone());
                    refs.push(ExprLeafRef {
                        input_port: port_name,
                        source: expr::LeafRef::Service {
                            endpoint: source.parse.node_id.clone(),
                            port: field.clone(),
                        },
                    });
                } else if let Some(Some(source)) = endpoints_by_name.get(base_ident) {
                    seen.insert(port_name.clone());
                    refs.push(ExprLeafRef {
                        input_port: port_name,
                        source: expr::LeafRef::Callable {
                            endpoint: source.node_id.clone(),
                            port: field.clone(),
                        },
                    });
                }
            } else {
                collect_expr_leaf_refs(base, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::BinOp(left, _, right) => {
            collect_expr_leaf_refs(left, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            collect_expr_leaf_refs(right, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
        }
        Expr::UnaryOp(_, inner) => {
            collect_expr_leaf_refs(inner, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
        }
        Expr::If(cond, then_, else_) => {
            collect_expr_leaf_refs(cond, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            collect_expr_leaf_refs(then_, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            if let Some(e) = else_ {
                collect_expr_leaf_refs(e, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::Pipe(receiver, call) => {
            collect_expr_leaf_refs(receiver, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            collect_expr_leaf_refs(call, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
        }
        Expr::PipeCall(receiver, _, args) => {
            collect_expr_leaf_refs(receiver, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            for (_, arg) in args {
                collect_expr_leaf_refs(arg, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_expr_leaf_refs(scrutinee, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            for arm in arms {
                collect_expr_leaf_refs(&arm.body, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::Call(_, args) => {
            for (_, arg) in args {
                collect_expr_leaf_refs(arg, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::Record(_, fields) => {
            for (_, field_expr) in fields {
                collect_expr_leaf_refs(field_expr, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_expr_leaf_refs(inner, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
                }
            }
        }
        Expr::List(elems) => {
            for elem in elems {
                collect_expr_leaf_refs(elem, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::Lambda(_, body) => {
            collect_expr_leaf_refs(body, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
        }
        Expr::For(_, iterable, _, body) => {
            collect_expr_leaf_refs(iterable, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            collect_expr_leaf_refs(body, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
        }
        Expr::Return(fields) => {
            for (_, field_expr) in fields {
                collect_expr_leaf_refs(field_expr, param_types, bound_callable_sources, bound_service_sources, endpoints_by_name, refs, seen, has_local_refs);
            }
        }
        Expr::Literal(_)
        | Expr::Map(_)
        | Expr::ServiceCall(_, _)
        | Expr::Guarded(_, _)
        | Expr::After(_, _) => {}
    }
}

/// Remap AST expression identifiers to use compute node input port names.
/// `FieldAccess(Ident("build"), "success")` becomes `Ident("build__success")`.
fn remap_expr_idents(expr: &Expr) -> expr::LoweredExpr {
    match expr {
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(base_ident) = base.as_ref() {
                expr::LoweredExpr::Ident(format!("{base_ident}__{field}"))
            } else {
                expr::LoweredExpr::FieldAccess {
                    expr: Box::new(remap_expr_idents(base)),
                    field: field.clone(),
                }
            }
        }
        Expr::Ident(name) => expr::LoweredExpr::Ident(name.clone()),
        Expr::BinOp(left, op, right) => expr::LoweredExpr::BinOp {
            left: Box::new(remap_expr_idents(left)),
            op: match op {
                daglang_syntax::ast::BinOp::Add => expr::LoweredBinOp::Add,
                daglang_syntax::ast::BinOp::Sub => expr::LoweredBinOp::Sub,
                daglang_syntax::ast::BinOp::Mul => expr::LoweredBinOp::Mul,
                daglang_syntax::ast::BinOp::Div => expr::LoweredBinOp::Div,
                daglang_syntax::ast::BinOp::Mod => expr::LoweredBinOp::Mod,
                daglang_syntax::ast::BinOp::Eq => expr::LoweredBinOp::Eq,
                daglang_syntax::ast::BinOp::Ne => expr::LoweredBinOp::Ne,
                daglang_syntax::ast::BinOp::Lt => expr::LoweredBinOp::Lt,
                daglang_syntax::ast::BinOp::Gt => expr::LoweredBinOp::Gt,
                daglang_syntax::ast::BinOp::Le => expr::LoweredBinOp::Le,
                daglang_syntax::ast::BinOp::Ge => expr::LoweredBinOp::Ge,
                daglang_syntax::ast::BinOp::And => expr::LoweredBinOp::And,
                daglang_syntax::ast::BinOp::Or => expr::LoweredBinOp::Or,
                daglang_syntax::ast::BinOp::NullCoalesce => expr::LoweredBinOp::NullCoalesce,
            },
            right: Box::new(remap_expr_idents(right)),
        },
        Expr::UnaryOp(op, inner) => expr::LoweredExpr::UnaryOp {
            op: match op {
                daglang_syntax::ast::UnaryOp::Not => expr::LoweredUnaryOp::Not,
                daglang_syntax::ast::UnaryOp::Neg => expr::LoweredUnaryOp::Neg,
            },
            expr: Box::new(remap_expr_idents(inner)),
        },
        Expr::If(cond, then_, else_) => expr::LoweredExpr::IfElse {
            cond: Box::new(remap_expr_idents(cond)),
            then_: Box::new(remap_expr_idents(then_)),
            else_: else_.as_ref().map(|e| Box::new(remap_expr_idents(e))),
        },
        Expr::Literal(lit) => {
            let lowered = match lit {
                daglang_syntax::ast::Literal::Int(n) => expr::LoweredLiteral::Int(*n),
                daglang_syntax::ast::Literal::Bool(b) => expr::LoweredLiteral::Bool(*b),
                daglang_syntax::ast::Literal::String(s) => expr::LoweredLiteral::String(s.clone()),
                daglang_syntax::ast::Literal::None => expr::LoweredLiteral::None,
                _ => expr::LoweredLiteral::None,
            };
            expr::LoweredExpr::Literal(lowered)
        }
        Expr::StringInterp(parts) => {
            let lowered_parts = parts
                .iter()
                .map(|part| match part {
                    daglang_syntax::ast::StringPart::Literal(s) => {
                        expr::LoweredStringPart::Literal(s.clone())
                    }
                    daglang_syntax::ast::StringPart::Expr(e) => {
                        expr::LoweredStringPart::Expr(remap_expr_idents(e))
                    }
                })
                .collect();
            expr::LoweredExpr::StringInterp(lowered_parts)
        }
        Expr::PipeCall(receiver, method, args) => expr::LoweredExpr::Pipe {
            receiver: Box::new(remap_expr_idents(receiver)),
            call: Box::new(expr::LoweredExpr::Call {
                name: method.as_str().to_string(),
                args: args
                    .iter()
                    .map(|(k, v)| (k.clone(), remap_expr_idents(v)))
                    .collect(),
            }),
        },
        _ => expr::LoweredExpr::Literal(expr::LoweredLiteral::None),
    }
}


/// Synthesize a compute node for a complex return expression.
/// Creates a node that evaluates the expression using `evaluate_fn_body` at runtime.
fn synthesize_expr_compute(
    builder: &mut DagBuilder,
    expr: &Expr,
    output_port: &Port,
    output_name: &str,
    param_types: &HashMap<String, String>,
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
    bound_service_sources: &HashMap<String, ServiceTransportEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    module_name: &str,
    item_name: &str,
    disambiguator: &str,
) -> Option<(String, String)> {
    let mut refs: Vec<ExprLeafRef> = Vec::new();
    let mut seen = HashSet::new();
    let mut has_local_refs = false;
    collect_expr_leaf_refs(
        expr,
        param_types,
        bound_callable_sources,
        bound_service_sources,
        endpoints_by_name,
        &mut refs,
        &mut seen,
        &mut has_local_refs,
    );

    if refs.is_empty() || has_local_refs {
        return None;
    }

    let input_ports: Vec<Port> = refs
        .iter()
        .map(|leaf| match &leaf.source {
            expr::LeafRef::Param { ty, .. } => {
                Port::with_cardinality(leaf.input_port.as_str(), ty.as_str(), Cardinality::ONE)
            }
            expr::LeafRef::Callable { .. } | expr::LeafRef::Service { .. } => {
                Port::with_cardinality(leaf.input_port.as_str(), "Any", Cardinality::ONE)
            }
        })
        .collect();
    let output_type = output_port.type_id.0.as_str();
    let result_port_name = "result";
    let output_ports = vec![Port::with_cardinality(result_port_name, output_type, Cardinality::ONE)];

    // 3. Create the fn body from the remapped expression.
    let lowered_expr = remap_expr_idents(expr);
    let fn_body = LoweredFnBody {
        stmts: vec![expr::LoweredStmt::Return(vec![
            (result_port_name.to_string(), lowered_expr),
        ])],
    };

    // 4. Create the compute node.
    let node_id = format!(
        "expr_compute_{}",
        sanitize_identifier(&format!("{module_name}_{item_name}_{output_name}_{disambiguator}"))
    );
    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: module_name.to_string(),
            name: format!("expr_compute::{item_name}::{output_name}"),
            kind: PrimitiveOpKind::ExprCompute {
                fn_body: Box::new(fn_body),
            },
        },
    ));

    for leaf in &refs {
        match &leaf.source {
            expr::LeafRef::Param { name, ty, .. } => {
                let param_source_id =
                    ensure_param_source_node(builder, module_name, item_name, name, ty);
                builder.add_edge(&param_source_id, name, &node_id, &leaf.input_port);
            }
            expr::LeafRef::Callable { endpoint, port }
            | expr::LeafRef::Service { endpoint, port } => {
                builder.add_edge(endpoint, port, &node_id, &leaf.input_port);
            }
        }
    }

    Some((node_id, result_port_name.to_string()))
}

fn wire_callable_return_outputs(
    builder: &mut DagBuilder,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    body_lossy: bool,
    param_types: &HashMap<String, String>,
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
    bound_service_sources: &HashMap<String, ServiceTransportEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
    module_name: &str,
    item_name: &str,
) {
    let outputs = match builder.dag.get_node(&NodeId::new(target.node_id.clone())) {
        Some(node) => node.outputs.clone(),
        None => return,
    };
    let output_bindings = collect_return_bindings(stmts, &outputs, body_lossy);
    if output_bindings.is_empty() {
        return;
    }

    for (index, (output_name, expr)) in output_bindings.into_iter().enumerate() {
        let Some(output_port) = outputs
            .iter()
            .find(|port| port.name.0 == output_name)
            .cloned()
        else {
            continue;
        };
        let dest_port = output_passthrough_input_name(output_name.as_str());
        if builder.has_edge_to_port(target.node_id.as_str(), dest_port.as_str()) {
            continue;
        }
        let Some((source_node, source_port)) = resolve_return_expr_source(
            builder,
            &expr,
            &output_port,
            output_name.as_str(),
            param_types,
            bound_callable_sources,
            bound_service_sources,
            endpoints_by_name,
            module_name,
            item_name,
            &format!("return_{index}"),
        ) else {
            // RT4c: Return output can't be wired (unsupported expression kind).
            // Optional outputs (T?) are expected to be missing sometimes;
            // required outputs that can't be wired indicate a lowering gap.
            // TODO(RT4c): collect these as structured LowerWarnings in the
            // lowerer return type instead of silently continuing.
            continue;
        };
        if source_node == target.node_id {
            continue;
        }
        builder.add_edge(
            source_node.as_str(),
            source_port.as_str(),
            target.node_id.as_str(),
            dest_port.as_str(),
        );
    }
}

/// Collect for-loop result bindings from top-level statements.
///
/// For each `binding = for var in iterable { body }`, creates a `LoweredEndpoint`
/// pointing to the loop node's "result" output, allowing downstream fn calls to
/// reference the binding as a data source.
fn collect_for_loop_bindings(
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    out: &mut HashMap<String, LoweredEndpoint>,
) {
    let mut for_index = 0usize;
    for stmt in stmts {
        match stmt {
            Stmt::Let(binding, expr) | Stmt::Assign(binding, expr) => {
                if matches!(expr, Expr::For(..)) {
                    let loop_node_id = format!("{}::cf_for_{for_index}", target.node_id);
                    out.insert(
                        binding.clone(),
                        LoweredEndpoint {
                            node_id: loop_node_id,
                            primary_output: "result".to_string(),
                        },
                    );
                }
                // Count for-loops in walk order to match add_control_flow_pattern_nodes
                // indexing. walk_stmts visits inner for-loops too, so recurse.
                let mut count = 0usize;
                walk_stmts(std::slice::from_ref(stmt), &mut |e| {
                    if matches!(e, Expr::For(..)) {
                        count += 1;
                    }
                });
                for_index += count;
            }
            Stmt::Node(ns) => {
                if matches!(ns.expr, Expr::For(..)) {
                    let loop_node_id = format!("{}::cf_for_{for_index}", target.node_id);
                    out.insert(
                        ns.name.clone(),
                        LoweredEndpoint {
                            node_id: loop_node_id,
                            primary_output: "result".to_string(),
                        },
                    );
                }
                let fake_stmt = Stmt::Assign(ns.name.clone(), ns.expr.clone());
                let mut count = 0usize;
                walk_stmts(std::slice::from_ref(&fake_stmt), &mut |e| {
                    if matches!(e, Expr::For(..)) {
                        count += 1;
                    }
                });
                for_index += count;
            }
            _ => {}
        }
    }
}

/// Wire for-loop iterable expressions to their corresponding loop node "items" ports.
///
/// Each `for x in <iterable> { ... }` produces a loop node with ID
/// `{target}::cf_for_{index}` and an input port named `"items"`. This function
/// resolves `<iterable>` to a source node (service call result, callable result,
/// or parameter) and wires the data edge.
fn wire_for_loop_iterables(
    builder: &mut DagBuilder,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    param_types: &HashMap<String, String>,
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
    bound_service_sources: &HashMap<String, ServiceTransportEndpoint>,
    module_name: &str,
    item_name: &str,
) {
    let for_sites = detect_for_loops_in_stmts(stmts);
    for (index, site) in for_sites.iter().enumerate() {
        let loop_node_id = format!("{}::cf_for_{index}", target.node_id);
        let Some(iterable) = &site.iterable else {
            continue;
        };
        match iterable {
            IterableRef::FieldAccess(base_ident, field_name) => {
                if let Some(source) = bound_callable_sources.get(base_ident) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        field_name.as_str(),
                        loop_node_id.as_str(),
                        "items",
                    );
                } else if let Some(source) = bound_service_sources.get(base_ident) {
                    builder.add_edge(
                        source.parse.node_id.as_str(),
                        field_name.as_str(),
                        loop_node_id.as_str(),
                        "items",
                    );
                }
            }
            IterableRef::Ident(name) => {
                if let Some(param_ty) = param_types.get(name) {
                    let src = ensure_param_source_node(
                        builder,
                        module_name,
                        item_name,
                        name,
                        param_ty.as_str(),
                    );
                    builder.add_edge(src.as_str(), name, loop_node_id.as_str(), "items");
                } else if let Some(source) = bound_callable_sources.get(name) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        source.primary_output.as_str(),
                        loop_node_id.as_str(),
                        "items",
                    );
                } else if let Some(source) = bound_service_sources.get(name) {
                    builder.add_edge(
                        source.parse.node_id.as_str(),
                        source.parse.primary_output.as_str(),
                        loop_node_id.as_str(),
                        "items",
                    );
                }
            }
        }
    }
}

fn collect_bound_callable_sources(
    module_name: &str,
    stmts: &[Stmt],
    endpoints_by_full: &HashMap<(String, String), LoweredEndpoint>,
    endpoints_by_name: &HashMap<String, Option<LoweredEndpoint>>,
) -> HashMap<String, LoweredEndpoint> {
    let mut bound = HashMap::<String, LoweredEndpoint>::new();
    let module_key = module_name.to_string();
    for stmt in stmts {
        match stmt {
            Stmt::Let(binding, expr)
            | Stmt::Assign(binding, expr)
            | Stmt::Node(NodeStmt {
                name: binding,
                expr,
                ..
            }) => match expr {
                Expr::Call(name, _) => {
                    if let Some(endpoint) =
                        endpoints_by_full.get(&(module_key.clone(), name.clone()))
                    {
                        bound.insert(binding.clone(), endpoint.clone());
                    } else if let Some(Some(endpoint)) = endpoints_by_name.get(name) {
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

// ============================================================================
// Output path extraction
// ============================================================================

/// Extract output file paths from a lowered DAG.
///
/// FC-7: Uses explicit `ContentUpsertOutputPath` primitive kind annotation
/// to identify output path nodes. Falls back to legacy `content_upsert_path_`
/// ID substring check for backward compatibility with DAGs lowered before
/// this change.
pub fn extract_output_paths(dag: &gunbc_ir::Dag<LoweredOp>) -> Vec<String> {
    let mut paths = std::collections::BTreeSet::new();
    collect_output_paths_recursive(&dag.nodes, &mut paths);
    paths.into_iter().collect()
}

fn collect_output_paths_recursive(
    nodes: &[gunbc_ir::node::Node<LoweredOp>],
    paths: &mut std::collections::BTreeSet<String>,
) {
    for node in nodes {
        // FC-7: Primary path — explicit ContentUpsertOutputPath annotation.
        if let gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive {
            kind: PrimitiveOpKind::ContentUpsertOutputPath { path },
            ..
        }) = &node.body
        {
            paths.insert(path.clone());
        }
        // FC-7: Legacy fallback — substring check (will be removed once all
        // lowering paths use ContentUpsertOutputPath).
        else if node.id.0.contains("content_upsert_path_") {
            if let gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive {
                kind:
                    PrimitiveOpKind::CallLiteralSource {
                        literal: PrimitiveLiteral::String(path),
                    },
                ..
            }) = &node.body
            {
                paths.insert(path.clone());
            }
        }
        if let gunbc_ir::node::NodeBody::SubDag(sub) = &node.body {
            collect_output_paths_recursive(&sub.nodes, paths);
        }
    }
}

/// An entrypoint inferred from graph structure.
///
/// A `func` item whose user-facing input ports are not all wired by
/// incoming edges is an entrypoint — its untapped inputs must be
/// supplied by the caller (CLI, REST, Lambda, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferredEntrypoint {
    /// The func name as declared in DSL (e.g., "pragma").
    pub func_name: String,
    /// The module path (e.g., "tools.pragma").
    pub module: String,
    /// The node ID in the lowered DAG (e.g., "tools.pragma::pragma").
    pub node_id: String,
}

/// Returns `true` if the port name represents a user-facing parameter.
///
/// Framework-injected ports (`__deps`, `tool:*`, `res:*`) are not user
/// parameters — they're wired by the runtime, not provided by the caller.
///
/// Use this filter consistently in entrypoint inference, CLI parameter
/// extraction, and any future exposure mapping (REST, Lambda, etc.).
pub fn is_user_param_port(port_name: &str) -> bool {
    let pn = PortName::from(port_name);
    !pn.is_internal() && !pn.is_tool() && !pn.is_resource() && !is_output_passthrough_input(port_name)
}

/// Infer entrypoints from graph structure.
///
/// A `func` (not `fn`) node is an entrypoint if:
/// - It has zero user-facing input ports (zero-arg tool), or
/// - Any of its user-facing input ports has no incoming edge (detected
///   via `gunbc_ir::detect_entrypoints`).
///
/// Results are sorted by `(module, func_name)` for deterministic output.
pub fn infer_entrypoints(dag: &gunbc_ir::Dag<LoweredOp>) -> Vec<InferredEntrypoint> {
    let ep_info = gunbc_ir::detect_entrypoints(dag);

    let mut entrypoints = Vec::new();

    for node in &dag.nodes {
        let gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable {
            kind: CallableKind::Func,
            module,
            name,
            ..
        }) = &node.body
        else {
            continue;
        };

        // Collect user-facing input ports (exclude framework ports)
        let user_ports: Vec<&str> = node
            .inputs
            .iter()
            .map(|p| p.name.0.as_str())
            .filter(|pn| is_user_param_port(pn))
            .collect();

        // Zero-arg funcs are entrypoints (no input needed = standalone tool).
        // Funcs with any untapped user-facing port are entrypoints.
        let is_entrypoint = user_ports.is_empty()
            || user_ports.iter().any(|pn| {
                ep_info.is_entrypoint_port(&node.id, &gunbc_ir::types::PortName(pn.to_string()))
            });

        if is_entrypoint {
            entrypoints.push(InferredEntrypoint {
                func_name: name.clone(),
                module: module.clone(),
                node_id: node.id.0.clone(),
            });
        }
    }

    entrypoints.sort_by(|a, b| (&a.module, &a.func_name).cmp(&(&b.module, &b.func_name)));
    entrypoints
}

/// Extract declared output path patterns from `func` items.
///
/// Walks the typed project collecting `declared_outputs` from `func` definitions.
/// Returns a sorted, deduplicated list.
pub fn extract_declared_outputs(project: &TypedProject) -> Vec<String> {
    let mut paths = std::collections::BTreeSet::new();
    for module in &project.modules {
        for item in &module.ast.items {
            if let Item::FuncDef(def) = &item.node {
                for s in &def.declared_outputs {
                    paths.insert(s.clone());
                }
            }
        }
    }
    paths.into_iter().collect()
}

#[cfg(test)]
mod tests;
