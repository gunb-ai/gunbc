//! **Stage 4 — Lower**: Transforms a `TypedProject` + `LoweringConfig`
//! into a `LowerOutput` containing `Dag<LoweredOp>` and metadata.
//!
//! # Pipeline position
//!
//! - **Before**: [`daglang-typecheck`] has produced a `TypedProject`
//! - **After**: [`daglang-derive`] extracts manifests, obligations, and metadata
//!
//! # Sequential steps
//!
//! 1. Expand patterns (`content_upsert` → read/compare/write chain)
//! 2. Lower service calls to transport triplets (prepare/execute/parse)
//! 3. Lower resource `acquire`/`release` blocks to DAG nodes
//! 4. Lower collection ops (`map`, `filter`, `fold`) to IR-level nodes
//! 5. Resolve `interface` bindings to concrete resources
//! 6. Emit `Dag<LoweredOp>` with `Node`/`Port`/`Edge` IR
//!
//! # Purity
//!
//! Pure — the `env_resolver` callback is injected by the caller and
//! invoked during profile config resolution (`resolve_profile_config_value`).
//! The lowerer itself contains no ambient I/O; all environment access
//! is mediated through the injected callback. No filesystem or network
//! access.
//!
//! # Failure
//!
//! Returns `LowerError` with diagnostic context.

// RT-C4: LoweringConfig groups the 4 boolean/optional lowerer parameters.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use daglang_contract::{Diagnostic, DiagnosticContext};
use daglang_syntax::ast::{
    BackoffStrategy, CapabilityDef, DataDef, Expr, Field, Item, Literal, NodeStmt, OperationDef,
    RateLimitUnit, ServiceDef, Stmt, TransportBinding, TypeBody,
};
use daglang_syntax::ast_utils::{
    canonical_resource_type_name, is_bool_type, is_list_type, is_map_string_string, is_secret_type,
    is_type_expr_optional, resource_type_name, service_call_lookup_keys, type_expr_to_string,
    walk_stmts,
};
use daglang_syntax::span::Span as SyntaxSpan;
use daglang_typecheck::{TypedCallableSignature, TypedItemSignature, TypedProject};
use gunbc_ir::patterns::branch::IfBuilder;
use gunbc_ir::patterns::{BranchBuilder, LoopBuilder, PatternOp};
use gunbc_ir::resource::AccessMode;
use gunbc_ir::transport::middleware::{
    RateLimitAlgorithm, RateLimitConfig, ResponseClassification, ResponseProvider, RetryBackoff,
    RetryConfig, TransportMiddlewareConfig,
};
use gunbc_ir::{
    Cardinality, Dag, DagTopology, Edge, EdgeKind, InputProvenance, Node, NodeId, NodeKind,
    NodeOrigin, OperationKey, Port, PortName, StaticFingerprint,
};
use serde::{Deserialize, Serialize};

pub mod anf;
pub mod eval;
pub mod expr;
pub(crate) mod scope;
pub mod spec;
pub(crate) mod transport;

pub use spec::{
    check_response_completeness, ArgvSegment, AuthRequirement, BodyEntry, ExitCodePattern,
    ExitMappingEntry, FieldSpec, FileOperationSpec, LocalOperationSpec, MockResponseEntry,
    OutputFieldSpec, ResponseCompletenessWarning, ResponseMappingEntry, ResponseStatusPattern,
    RestOperationSpec, ServiceOperationSpec, ShellOperationSpec, ShellOutputParsing,
};

pub use expr::LoweredFnBody;

/// Lowered operation payload for daglang graph nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoweredOp {
    Callable {
        module: String,
        kind: CallableKind,
        name: String,
        obligation: CallableObligation,
        is_interactive: bool,
        resource_target: Option<String>,
        /// Lowered fn body for `CallableKind::Fn` items — `None` for
        /// func/pattern items and non-DSL callables.
        fn_body: Option<Box<LoweredFnBody>>,
    },
    /// Transport nodes (service prepare/execute/parse) with required metadata.
    Transport {
        module: String,
        kind: CallableKind,
        name: String,
        obligation: TransportObligation,
        service_metadata: Box<ServiceCallMetadata>,
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
    /// A pattern-internal operation (LoopUnpack, LoopPack, BranchMerge, etc.).
    /// Wraps `PatternOp` directly — no round-trip conversion needed at resolve time.
    Pattern(PatternOp),
    /// A pattern operation that is not yet supported in daglang lowering.
    /// Produces a structured error at resolution time instead of panicking.
    UnsupportedPattern { name: String },
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallableKind {
    Fn,
    Func,
    Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// C24: Extract a named field from a Map/Record/JSON input.
    /// The base value is always the node's sole declared input port (derived
    /// from the validated node schema at resolve time, not stored here).
    GetField {
        field: String,
    },
    /// C24: String interpolation — `"hello {name}, you have {count} items"`.
    /// Inputs: one port per interpolated expression. Output: concatenated string.
    StringInterpolate {
        /// Static template parts interleaved with input port names.
        /// `parts.len() == input_ports.len() + 1` (first/last are always literal).
        parts: Vec<String>,
        input_ports: Vec<String>,
    },
    /// C24: Binary operation — `a + b`, `a == b`, `a && b`, etc.
    BinaryOp {
        op: crate::expr::LoweredBinOp,
    },
    /// C24: Unary operation — `!x`, `-x`.
    UnaryOp {
        op: crate::expr::LoweredUnaryOp,
    },
    /// C24: Conditional — `if cond { then } else { else_ }`.
    /// Inputs: `condition`, `then`, `else`. Output: selected branch value.
    Conditional,
    /// C24: Match dispatch — pattern match on a scrutinee.
    /// Arms are evaluated in order; first matching arm's body is the output.
    MatchDispatch {
        arms: Vec<crate::expr::LoweredMatchArm>,
        sibling_fns: std::collections::BTreeMap<String, LoweredFnBody>,
    },
    /// C24: Record construction — `{ field1: val1, field2: val2 }`.
    /// Each field maps to an input port; output is a Value::Map.
    RecordConstruct {
        fields: Vec<String>,
    },
    /// C24: Null coalesce — `a ?? b`.
    /// Input: `value`, `default`. Output: `value` if non-null, else `default`.
    NullCoalesce,
    /// C24: Variant construction — `Ok { value: x }` or unit `None`.
    /// Produces Value::Map with `_variant` tag or Value::Str for unit variants.
    VariantConstruct {
        tag: String,
        fields: Vec<String>,
    },
    /// C24: List construction — `[a, b, c]`.
    /// Each element maps to an input port (`elem_0`, `elem_1`, ...).
    /// Output is a Value::List of all elements in order.
    ListConstruct {
        count: usize,
    },
}

impl PrimitiveOpKind {
    /// Returns the required input port names for expression-primitive kinds.
    ///
    /// This is the single authority for which ports each pure-value primitive
    /// requires. Non-expression kinds (source nodes, I/O, metadata) return
    /// `None` — they have their own dedicated resolvers.
    ///
    /// `GetField` also returns `None` because its single input port is
    /// validated by name from the node schema at resolve time (it cannot be
    /// named statically).
    pub fn required_input_ports(&self) -> Option<Vec<String>> {
        match self {
            PrimitiveOpKind::BinaryOp { .. } => Some(vec!["left".into(), "right".into()]),
            PrimitiveOpKind::UnaryOp { .. } => Some(vec!["operand".into()]),
            PrimitiveOpKind::Conditional => Some(vec!["condition".into(), "then".into()]),
            PrimitiveOpKind::NullCoalesce => Some(vec!["value".into(), "default".into()]),
            PrimitiveOpKind::MatchDispatch { .. } => Some(vec!["scrutinee".into()]),
            PrimitiveOpKind::StringInterpolate { input_ports, .. } => Some(input_ports.clone()),
            PrimitiveOpKind::RecordConstruct { fields } => Some(fields.clone()),
            PrimitiveOpKind::VariantConstruct { fields, .. } => Some(fields.clone()),
            PrimitiveOpKind::ListConstruct { count } => {
                Some((0..*count).map(|i| format!("elem_{i}")).collect())
            }
            // GetField: validated separately (single input from node schema)
            PrimitiveOpKind::GetField { .. } => None,
            // Non-expression kinds: source, I/O, metadata
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveLiteral {
    String(String),
    Int(i64),
    Bool(bool),
    Json(serde_json::Value),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Obligation subset valid for `LoweredOp::Callable` nodes.
///
/// Transport obligations (`ServiceTransportPrepare/Execute/Parse`) are
/// structurally excluded — they belong exclusively on `LoweredOp::Transport`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallableObligation {
    None,
    ResourceProvide,
    ResourceAcquire,
    ResourceRelease,
    InterfaceContractVerification,
    PureRender,
    PureDataLoad,
    PureGeneric,
}

impl From<CallableObligation> for ObligationCategory {
    fn from(o: CallableObligation) -> Self {
        match o {
            CallableObligation::None => ObligationCategory::None,
            CallableObligation::ResourceProvide => ObligationCategory::ResourceProvide,
            CallableObligation::ResourceAcquire => ObligationCategory::ResourceAcquire,
            CallableObligation::ResourceRelease => ObligationCategory::ResourceRelease,
            CallableObligation::InterfaceContractVerification => {
                ObligationCategory::InterfaceContractVerification
            }
            CallableObligation::PureRender => ObligationCategory::PureRender,
            CallableObligation::PureDataLoad => ObligationCategory::PureDataLoad,
            CallableObligation::PureGeneric => ObligationCategory::PureGeneric,
        }
    }
}

/// Obligation subset valid for `LoweredOp::Transport` nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportObligation {
    Prepare,
    Execute,
    Parse,
}

impl From<TransportObligation> for ObligationCategory {
    fn from(o: TransportObligation) -> Self {
        match o {
            TransportObligation::Prepare => ObligationCategory::ServiceTransportPrepare,
            TransportObligation::Execute => ObligationCategory::ServiceTransportExecute,
            TransportObligation::Parse => ObligationCategory::ServiceTransportParse,
        }
    }
}

// ServiceTransportClass has been moved to gunbc-ir. Re-export for backward
// compatibility with consumers that import it from daglang_lower.
pub use gunbc_ir::ServiceTransportClass;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceCallMetadata {
    pub service: String,
    pub operation: String,
    pub transport: ServiceTransportClass,
    pub idempotent: bool,
    pub readonly: bool,
    /// Full protocol spec extracted from DSL service/operation declarations.
    /// Used by generic protocol interpreters to replace per-service adapters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<ServiceOperationSpec>,
    /// Response provider classification (S45). Stamped from DSL
    /// `config { response_provider: X }` when present, else inferred from
    /// service name substrings. Propagated to `Node.response_provider`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_provider: Option<ResponseProvider>,
}

impl LoweredOp {
    pub fn obligation_category(&self) -> ObligationCategory {
        match self {
            Self::Callable { obligation, .. } => ObligationCategory::from(*obligation),
            Self::Transport { obligation, .. } => ObligationCategory::from(*obligation),
            Self::Primitive { kind, .. } => kind.obligation_category(),
            Self::Collection { .. }
            | Self::Pipeline { .. }
            | Self::Pattern(_)
            | Self::UnsupportedPattern { .. } => ObligationCategory::None,
        }
    }

    pub fn service_call_metadata(&self) -> Option<&ServiceCallMetadata> {
        match self {
            Self::Transport {
                service_metadata, ..
            } => Some(service_metadata),
            Self::Callable { .. }
            | Self::Primitive { .. }
            | Self::Collection { .. }
            | Self::Pipeline { .. }
            | Self::Pattern(_)
            | Self::UnsupportedPattern { .. } => None,
        }
    }
}

impl PrimitiveOpKind {
    /// Returns true for C24 structural expression primitives that are evaluated
    /// by the interpreter, not code-generated or obligation-bearing.
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            Self::StringInterpolate { .. }
                | Self::BinaryOp { .. }
                | Self::UnaryOp { .. }
                | Self::Conditional
                | Self::MatchDispatch { .. }
                | Self::RecordConstruct { .. }
                | Self::NullCoalesce
                | Self::VariantConstruct { .. }
                | Self::ListConstruct { .. }
        )
    }

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
            Self::GetField { .. } => ObligationCategory::None,
            // C24: All remaining structural primitives are pure computation — no obligations.
            _ => {
                debug_assert!(
                    self.is_structural(),
                    "unhandled non-structural primitive: {self:?}"
                );
                ObligationCategory::None
            }
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
    let op = match &node.body {
        gunbc_ir::NodeBody::Opaque(op) => op,
        gunbc_ir::NodeBody::SubDag(..) => return NodeKind::Pure,
    };

    // Collection nodes are stamped before obligation dispatch — they
    // always map to `NodeKind::Collection` regardless of obligation.
    if matches!(op, LoweredOp::Collection { .. }) {
        return NodeKind::Collection;
    }

    let cat = op.obligation_category();

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
        ObligationCategory::ServiceParamSource => NodeKind::ParamSource,
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
        // Stamp transport class and response provider from ServiceCallMetadata
        // so consumers can read them directly from the Node without
        // Any-downcasting LoweredOp (S45/S46).
        if let gunbc_ir::NodeBody::Opaque(ref op) = node.body {
            if let Some(meta) = op.service_call_metadata() {
                node.transport_class = Some(meta.transport);
                if let Some(rp) = meta.response_provider {
                    node.response_provider = Some(rp);
                }
            }
        }
        if let gunbc_ir::NodeBody::SubDag(ref mut inner, _) = node.body {
            stamp_node_kinds(inner);
        }
    }
    // C22: Stamp static fingerprints on transport execute nodes after all
    // nodes and edges are in place.
    stamp_static_fingerprints(dag);
}

/// Returns true if the node ID has a call-site clone suffix (e.g., `_c1`, `_c2`).
///
/// The lowerer creates these suffixed copies via `clone_transport_triplet` when
/// multiple callables reference the same service operation.
fn is_call_site_clone(node_id: &str) -> bool {
    // Match trailing `_cN` where N is one or more digits.
    if let Some(pos) = node_id.rfind("_c") {
        let suffix = &node_id[pos + 2..];
        !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

/// Stamp static fingerprints on transport execute nodes (C22).
///
/// For each node with an `operation_key`, analyzes the incoming edges to
/// determine provenance of each prepare-node input. The fingerprint enables
/// compile-time duplicate detection: two transport nodes with identical
/// fingerprints perform provably identical work.
pub fn stamp_static_fingerprints(dag: &mut Dag<LoweredOp>) {
    // Build edge index: target_node -> [(source_node, source_port, target_port)]
    let edge_index: HashMap<String, Vec<(&str, &str, &str)>> = {
        let mut idx: HashMap<String, Vec<(&str, &str, &str)>> = HashMap::new();
        for edge in &dag.edges {
            idx.entry(edge.to_node.0.clone()).or_default().push((
                &edge.from_node.0,
                &edge.from_port.0,
                &edge.to_port.0,
            ));
        }
        idx
    };

    // Collect literal source node IDs for provenance classification.
    let literal_sources: HashMap<String, Option<String>> = dag
        .nodes
        .iter()
        .filter_map(|n| {
            if let gunbc_ir::NodeBody::Opaque(LoweredOp::Primitive {
                kind: PrimitiveOpKind::CallLiteralSource { literal },
                ..
            }) = &n.body
            {
                Some((n.id.0.clone(), Some(format!("{literal:?}"))))
            } else {
                None
            }
        })
        .collect();

    // Pass 1: collect fingerprints (immutable borrow).
    let mut fingerprints: Vec<(String, StaticFingerprint)> = Vec::new();

    for node in dag.nodes.iter() {
        if node.operation_key.is_none() || node.kind != NodeKind::TransportExecute {
            continue;
        }
        // Skip call-site clones (suffixed _c1, _c2, …) — these are intentional
        // duplicates for different callable scopes, not redundant work.
        if is_call_site_clone(&node.id.0) {
            continue;
        }
        let op_key = node.operation_key.as_ref().unwrap().clone();

        // Find the prepare node that feeds the "request" input.
        let prepare_id = edge_index.get(&node.id.0).and_then(|edges| {
            edges
                .iter()
                .find(|(_, _, to_port)| *to_port == "request")
                .map(|(from_node, _, _)| from_node.to_string())
        });

        let Some(prepare_id) = prepare_id else {
            continue;
        };

        // Analyze prepare node's incoming edges to determine input provenance.
        let prepare_edges = edge_index.get(&prepare_id);
        let mut keys: Vec<(String, InputProvenance)> = Vec::new();

        if let Some(prepare_node) = dag.nodes.iter().find(|n| n.id.0 == prepare_id) {
            for input_port in &prepare_node.inputs {
                let port_name = &input_port.name.0;
                // Skip internal ports (deps, resource, credential).
                if port_name == PortName::DEPS
                    || port_name.starts_with("res:")
                    || port_name == PortName::RESOURCE_CREDENTIAL
                {
                    continue;
                }

                let provenance = if let Some(edges) = prepare_edges {
                    if let Some((source_node, source_port, _)) =
                        edges.iter().find(|(_, _, tp)| *tp == port_name.as_str())
                    {
                        if literal_sources.contains_key(*source_node) {
                            let literal_val = literal_sources
                                .get(*source_node)
                                .and_then(|v| v.clone())
                                .unwrap_or_default();
                            InputProvenance::Literal(literal_val)
                        } else {
                            InputProvenance::Edge {
                                source_node: NodeId::new(*source_node),
                                source_port: PortName::new(*source_port),
                            }
                        }
                    } else {
                        InputProvenance::Dynamic
                    }
                } else {
                    InputProvenance::Dynamic
                };

                keys.push((port_name.clone(), provenance));
            }
        }

        keys.sort_by(|a, b| a.0.cmp(&b.0));
        fingerprints.push((
            node.id.0.clone(),
            StaticFingerprint::with_keys(op_key, keys),
        ));
    }

    // Pass 2: apply fingerprints (mutable borrow).
    for (node_id, fp) in fingerprints {
        if let Some(node) = dag.nodes.iter_mut().find(|n| n.id.0 == node_id) {
            node.static_fingerprint = Some(fp);
        }
    }
}

/// Validate that every top-level `Callable` node with `fn_body: None` and no
/// transport obligation has `__out:{name}` passthrough wiring for all output
/// ports — either via a declared input port or an incoming edge.
///
/// Without this, such nodes resolve to `DeclaredOutputCallableOp` at runtime
/// which hard-errors on missing passthrough inputs. Catching this at lowering
/// time surfaces the bug in the compiler instead of at execution time.
///
/// Note: SubDag-internal nodes are NOT checked because their `__out:` wiring
/// comes through SubDag boundary inference (not edges in the inner DAG).
/// With-transport branch/loop body ops use `fn_body` for passthrough instead.
fn validate_callable_output_wiring(dag: &Dag<LoweredOp>) -> Result<(), LowerError> {
    // Build edge target index: node_id → set of target port names.
    let edge_targets: HashMap<&str, HashSet<&str>> = {
        let mut idx: HashMap<&str, HashSet<&str>> = HashMap::new();
        for edge in &dag.edges {
            idx.entry(edge.to_node.0.as_str())
                .or_default()
                .insert(edge.to_port.0.as_str());
        }
        idx
    };

    for node in &dag.nodes {
        if let gunbc_ir::NodeBody::Opaque(LoweredOp::Callable {
            fn_body: None,
            obligation,
            name,
            ..
        }) = &node.body
        {
            // Transport roles (prepare/execute/parse) and other obligation
            // categories are resolved via dedicated ops — they don't need
            // __out: passthrough ports.
            if *obligation != CallableObligation::None {
                continue;
            }
            let input_names: HashSet<&str> =
                node.inputs.iter().map(|p| p.name.0.as_str()).collect();
            let wired_ports = edge_targets.get(node.id.0.as_str());
            for output_port in &node.outputs {
                let passthrough_name = format!(
                    "{}{}",
                    PortName::OUTPUT_PASSTHROUGH_PREFIX,
                    output_port.name.0
                );
                let has_port = input_names.contains(passthrough_name.as_str());
                let has_edge = wired_ports
                    .map(|ports| ports.contains(passthrough_name.as_str()))
                    .unwrap_or(false);
                if !has_port && !has_edge {
                    return Err(LowerError::MissingCallablePassthrough {
                        node: node.id.0.clone(),
                        name: name.clone(),
                        missing_port: passthrough_name,
                    });
                }
            }
        }
    }
    Ok(())
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
            canonical_kind_for_obligation(ObligationCategory::from(*obligation)).map(str::to_string)
        }
        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Transport { obligation, .. }) => {
            canonical_kind_for_obligation(ObligationCategory::from(*obligation)).map(str::to_string)
        }
        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive { kind, .. }) => {
            canonical_kind_for_obligation(kind.obligation_category()).map(str::to_string)
        }
        gunbc_ir::node::NodeBody::Opaque(_) | gunbc_ir::node::NodeBody::SubDag(..) => None,
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

    fn merge(&mut self, other: EndpointRegistry<T>) {
        for (key, value) in other.by_key {
            self.by_key.entry(key).or_insert(value);
        }
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

    for module in project.modules() {
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
    for module in project.modules() {
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
                        return Err(LowerError::InvalidConcreteBinding {
                            profile: profile_full.clone(),
                            detail: format!("interface `{}` is ambiguous", bind.interface_type),
                        })
                    }
                    NameResolution::Missing => {
                        return Err(LowerError::InvalidConcreteBinding {
                            profile: profile_full.clone(),
                            detail: format!("interface `{}` is unresolved", bind.interface_type),
                        })
                    }
                };
                let resolved_impl = match service_registry.resolve(&bind.implementation_type) {
                    NameResolution::Resolved(full) => full,
                    NameResolution::Ambiguous => {
                        return Err(LowerError::InvalidConcreteBinding {
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
                            return Err(LowerError::InvalidConcreteBinding {
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
                    return Err(LowerError::InvalidConcreteBinding {
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
    env_resolver: &dyn Fn(&str) -> Option<String>,
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
                    registry.by_full.get(full).ok_or_else(|| {
                        LowerError::UnknownConcreteBinding {
                            profile: profile.to_string(),
                        }
                    })?,
                ),
                Some(None) => {
                    return Err(LowerError::AmbiguousConcreteBinding {
                        profile: profile.to_string(),
                    })
                }
                None => {
                    return Err(LowerError::UnknownConcreteBinding {
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
                env_resolver,
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
    env_resolver: &dyn Fn(&str) -> Option<String>,
) -> Result<ProfileConfigValue, LowerError> {
    match expr {
        Expr::Literal(Literal::String(value)) => Ok(ProfileConfigValue::Literal(value.clone())),
        Expr::Ident(name) => Ok(ProfileConfigValue::Literal(name.clone())),
        Expr::Call(name, args) if name == "env" => {
            let env_var = parse_single_string_call_arg(args).ok_or_else(|| {
                LowerError::InvalidConcreteBinding {
                    profile: profile.to_string(),
                    detail: format!(
                        "config `{key}` for `{interface_type}` must be `env(\"VAR\")`"
                    ),
                }
            })?;
            let env_value = env_resolver(env_var.as_str())
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
                LowerError::InvalidConcreteBinding {
                    profile: profile.to_string(),
                    detail: format!(
                        "config `{key}` for `{interface_type}` must be `secret(\"name\")`"
                    ),
                }
            })?;
            Ok(ProfileConfigValue::SecretRef(secret_name))
        }
        _ => Err(LowerError::InvalidConcreteBinding {
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
    for module in project.modules() {
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
    for module in project.modules() {
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

/// Register stub transport triplets for interfaces that lack concrete bindings (IS-4).
///
/// When compiling without a concrete binding, interface capabilities still need transport
/// triplets in the registry so `resolve_service_call_source` can find them. These
/// stubs use `ServiceTransportClass::InterfaceStub` and are DryRun-compatible;
/// real-mode execution will surface a "missing concrete binding" error at the resolver.
fn derive_interface_stub_transport_triplets(
    project: &TypedProject,
    stub_interfaces: &HashSet<String>,
) -> transport::TransportManifest {
    let mut manifest = transport::TransportManifest::new();
    if stub_interfaces.is_empty() {
        return manifest;
    }

    for module in project.modules() {
        let module_name = module.module_path.as_dotted();
        let source_file = module.path.display().to_string();
        for item in &module.ast.items {
            let Item::InterfaceDef(interface) = &item.node else {
                continue;
            };

            if !is_bound_interface_type_name(stub_interfaces, &interface.name) {
                continue;
            }

            for capability in &interface.capabilities {
                let origin = pattern_expansion_origin(
                    &source_file,
                    &module_name,
                    &interface.name,
                    item.span,
                    "interface_stub_transport",
                );
                let metadata = ServiceCallMetadata {
                    service: interface.name.clone(),
                    operation: capability.name.clone(),
                    transport: ServiceTransportClass::InterfaceStub,
                    idempotent: capability.idempotent,
                    readonly: capability.readonly,
                    spec: Some(ServiceOperationSpec::InterfaceStub {
                        interface: interface.name.clone(),
                        capability: capability.name.clone(),
                    }),
                    response_provider: None,
                };

                let suffix = sanitize_identifier(&format!(
                    "{module_name}_{}_{}",
                    interface.name, capability.name
                ));
                let prepare_id = format!("prepare_transport_{suffix}");
                let execute_id = format!("execute_transport_{suffix}");
                let parse_id = format!("parse_transport_{suffix}");

                let prepare_ports = capability_prepare_ports(capability, &metadata);
                let prepare_inputs = prepare_ports
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect::<Vec<_>>();

                // Execute node: TransportRequest → typed capability outputs.
                // In DryRun, boundary mocks supply typed fields directly.
                // In Real mode, the execute op errors because no concrete binding exists.
                let typed_outputs = if capability.outputs.is_empty() {
                    vec![Port::scalar("result", "Unit")]
                } else {
                    capability
                        .outputs
                        .iter()
                        .map(|field| {
                            let ty = type_expr_to_string(&field.ty);
                            Port::scalar(field.name.as_str(), ty.as_str())
                        })
                        .collect::<Vec<_>>()
                };

                let triplet_spec = transport::TransportTripletSpec {
                    module: module_name.clone(),
                    service: interface.name.clone(),
                    operation: capability.name.clone(),
                    metadata: metadata.clone(),
                    prepare_id: prepare_id.clone(),
                    execute_id: execute_id.clone(),
                    parse_id: parse_id.clone(),
                    prepare_inputs: prepare_ports,
                    execute_extra_inputs: vec![],
                    parse_outputs: typed_outputs.clone(),
                    execute_parse_wiring: transport::ExecuteParseWiring::PerField {
                        fields: typed_outputs,
                    },
                    origin: Some(origin.clone()),
                    operation_key: Some(OperationKey::new(&interface.name, &capability.name)),
                };
                transport::emit_triplet_to_manifest(
                    &mut manifest,
                    transport::build_transport_triplet(triplet_spec),
                );

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
                manifest
                    .registry
                    .register(cap_key.clone(), endpoint.clone());
                manifest
                    .registry
                    .register(format!("{module_name}.{cap_key}"), endpoint);
            }
        }
    }
    manifest
}

/// Read-only lookup tables threaded through wiring functions during lowering.
///
/// Bundles the identifiers and registries that most wiring functions need,
/// reducing parameter counts across the module. Constructed per-item within
/// `add_dependency_edges` and similar top-level phases.
struct LoweringContext<'a> {
    source_file: &'a str,
    module_name: &'a str,
    item_name: &'a str,
    item_span: SyntaxSpan,
    param_types: &'a HashMap<String, String>,
    endpoints_by_name: &'a HashMap<String, Option<LoweredEndpoint>>,
    data_values: &'a HashMap<String, serde_json::Value>,
    service_registry: &'a ServiceEndpointRegistry,
    bound_callable_sources: &'a HashMap<String, LoweredEndpoint>,
    bound_service_sources: &'a HashMap<String, ServiceTransportEndpoint>,
    expanded_results: &'a HashMap<String, PatternExpansionResult>,
    /// Local let bindings (non-call) from the current callable body.
    /// Used to resolve local variable references in return expressions.
    local_let_bindings: &'a HashMap<String, &'a Expr>,
    /// The body statements of the current callable, for fn body evaluation.
    body_stmts: &'a [Stmt],
    /// Pre-collected lowered bodies for all pure `fn` definitions in the project.
    /// Passed to structural nodes (e.g. MatchDispatch) as `sibling_fns` so
    /// the runtime evaluator can execute user-defined fn calls.
    all_fn_bodies: &'a std::collections::BTreeMap<String, LoweredFnBody>,
    /// C24: Known sum-type variant names for match arm lowering.
    variant_names: &'a HashSet<String>,
    /// Default expressions for callable parameters, keyed by callable name.
    callable_param_defaults: &'a HashMap<String, Vec<(String, daglang_syntax::ast::Expr)>>,
    /// Full-qualified endpoint lookup for cross-module Call resolution.
    endpoints_by_full: &'a HashMap<(String, String), LoweredEndpoint>,
    /// Resource binding types from `uses` clauses (e.g., `fs: Filesystem`).
    uses_binding_types: &'a HashMap<String, String>,
}

fn user_code_origin(
    source_file: &str,
    module_name: &str,
    item_name: &str,
    span: SyntaxSpan,
) -> NodeOrigin {
    NodeOrigin::UserCode {
        file: source_file.to_string(),
        module: module_name.to_string(),
        item: item_name.to_string(),
        span_start: span.start,
        span_end: span.end,
    }
}

fn pattern_expansion_origin(
    source_file: &str,
    module_name: &str,
    item_name: &str,
    span: SyntaxSpan,
    pattern_kind: &str,
) -> NodeOrigin {
    NodeOrigin::PatternExpansion {
        file: source_file.to_string(),
        module: module_name.to_string(),
        item: item_name.to_string(),
        span_start: span.start,
        span_end: span.end,
        pattern_kind: pattern_kind.to_string(),
    }
}

fn pattern_expansion_origin_for_ctx(ctx: &LoweringContext<'_>, pattern_kind: &str) -> NodeOrigin {
    pattern_expansion_origin(
        ctx.source_file,
        ctx.module_name,
        ctx.item_name,
        ctx.item_span,
        pattern_kind,
    )
}

fn top_level_item_name(item: &Item) -> Option<&str> {
    match item {
        Item::FnDef(def) => Some(def.name.as_str()),
        Item::FuncDef(def) => Some(def.name.as_str()),
        Item::PatternDef(def) => Some(def.name.as_str()),
        Item::ServiceDef(def) => Some(def.name.as_str()),
        Item::ResourceDef(def) => Some(def.name.as_str()),
        Item::InterfaceDef(def) => Some(def.name.as_str()),
        Item::PipelineDef(def) => Some(def.name.as_str()),
        Item::DataDef(def) => Some(def.name.as_str()),
        Item::TestDef(def) => Some(def.name.as_str()),
        Item::ProfileDef(def) => Some(def.name.as_str()),
        Item::TypeDef(def) => Some(def.name.as_str()),
        Item::ExternAssetDecl(def) => Some(def.name.as_str()),
        _ => None,
    }
}

/// Per-node expansion state for pattern lowering.
///
/// Groups the suffix, target, and argument-mapping state that flows through
/// the pattern expansion call chain (expand_pattern_body_node, etc.).
struct PatternNodeEnv<'a> {
    suffix: &'a str,
    arg_map: &'a HashMap<String, &'a Expr>,
    uses_binding_types: &'a HashMap<String, String>,
    node_outputs: &'a HashMap<String, ExpandedNodeOutput>,
}

/// Parameters for recursive pattern expansion (target + pattern registry + depth).
struct PatternExpansionParams<'a> {
    target: &'a LoweredEndpoint,
    all_patterns: &'a HashMap<String, ExpandablePattern<'a>>,
    depth: usize,
}

/// Shared lookup tables for DAG wiring (endpoints, service registry, data values).
struct DagWiringContext<'a> {
    endpoints_by_full: &'a HashMap<(String, String), LoweredEndpoint>,
    endpoints_by_name: &'a HashMap<String, Option<LoweredEndpoint>>,
    service_registry: &'a ServiceEndpointRegistry,
    data_values: &'a HashMap<String, serde_json::Value>,
    variant_names: &'a HashSet<String>,
    /// Default expressions for callable parameters, keyed by callable name.
    /// Used to inject literal source nodes for omitted call args with defaults.
    callable_param_defaults: &'a HashMap<String, Vec<(String, daglang_syntax::ast::Expr)>>,
}

/// Wraps a `Dag` with O(1) deduplication tracking for nodes and edges.
struct DagBuilder {
    dag: Dag<LoweredOp>,
    seen_nodes: HashSet<String>,
    seen_edges: HashSet<(String, String, String, String, EdgeKind)>,
    /// Monotonic counter per (to_node, to_port) for deterministic fan-in ordering.
    fan_in_counts: HashMap<(String, String), usize>,
}

impl DagBuilder {
    fn new() -> Self {
        Self {
            dag: Dag::new(),
            seen_nodes: HashSet::new(),
            seen_edges: HashSet::new(),
            fan_in_counts: HashMap::new(),
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
            let idx = self
                .fan_in_counts
                .entry((to.to_string(), to_port.to_string()))
                .or_insert(0);
            let index = *idx;
            *idx += 1;
            self.dag.add_edge(Edge {
                from_node: NodeId::new(from.to_string()),
                from_port: PortName::new(from_port.to_string()),
                to_node: NodeId::new(to.to_string()),
                to_port: PortName::new(to_port.to_string()),
                index,
                kind,
            });
        }
    }

    fn has_edge_to_port(&self, to_node: &str, to_port: &str) -> bool {
        self.seen_edges
            .iter()
            .any(|(_, _, tn, tp, _)| tn == to_node && tp == to_port)
    }

    /// Apply a transport manifest's nodes and edges to the builder.
    fn apply_manifest(&mut self, manifest: &transport::TransportManifest) {
        for node in &manifest.nodes {
            self.add_node(node.clone());
        }
        for edge in &manifest.edges {
            self.add_edge(
                &edge.from_node,
                &edge.from_port,
                &edge.to_node,
                &edge.to_port,
            );
        }
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
            // Clear the static fingerprint on call-site clones: these are
            // intentional duplicates for different callable scopes, not
            // redundant work that C22 should flag.
            n.static_fingerprint = None;
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
    UnknownConcreteBinding { profile: String },
    /// Requested profile name matches multiple declarations.
    AmbiguousConcreteBinding { profile: String },
    /// Profile declaration contains an invalid bind entry.
    InvalidConcreteBinding { profile: String, detail: String },
    /// A bound interface call requires an active profile selection.
    ProfileRequiredForBoundServiceCall {
        caller: String,
        binding: String,
        interface_type: String,
    },
    /// Active profile does not bind an interface used by a callable.
    MissingConcreteBinding {
        profile: String,
        interface_type: String,
    },
    /// Transport block specifies an unknown file operation.
    InvalidFileOp { operation: String, file_op: String },
    /// Transport block contents are invalid or incomplete.
    InvalidTransportSpec {
        service: String,
        operation: String,
        detail: String,
    },
    /// Active profile uses an env(...) config binding that is not set.
    MissingProfileConfigEnv {
        profile: String,
        interface_type: String,
        key: String,
        env_var: String,
    },
    /// An explicit DSL annotation has an unrecognized value.
    InvalidAnnotation {
        service: String,
        annotation: String,
        detail: String,
    },
    /// No executable declarations were available for lowering.
    NoLowerableItems,
    /// A pure function contains an effectful node statement.
    PureFnContainsEffectfulNode { fn_name: String, node_name: String },
    /// `auth_input` references a field that does not exist in the operation's inputs,
    /// or the referenced field is not of type `Secret`.
    InvalidAuthInput {
        service: String,
        operation: String,
        field_name: String,
        reason: String,
    },
    /// Provider config field is not recognized for the given provider.
    InvalidProviderConfigField {
        service: String,
        field: String,
        known_fields: Vec<String>,
    },
    /// Service has config fields but no known schema prefix.
    UnknownProviderPrefix {
        service: String,
        fields: Vec<String>,
        known_prefixes: Vec<String>,
    },
    /// Provider schema mapping references a type that does not exist.
    UnknownProviderSchemaType { prefix: String, schema: String },
    /// A callable node has `fn_body: None` but no `__out:` passthrough input
    /// for one of its output ports. This would fail at resolve time when the
    /// node becomes a `DeclaredOutputCallableOp`.
    MissingCallablePassthrough {
        node: String,
        name: String,
        missing_port: String,
    },
    /// Expression could not be lowered to a DAG source.
    ExprLower(String),
    /// Data-flow wiring failed during lowering.
    WiringFailure { source_file: String, detail: String },
    /// One or more ports reference type_ids not registered in the TypeRegistry.
    PortTypeValidation(Vec<String>),
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
            Self::InvalidTransportSpec {
                service,
                operation,
                detail,
            } => write!(
                f,
                "invalid transport spec for `{service}.{operation}`: {detail}"
            ),
            Self::UnknownConcreteBinding { profile } => {
                write!(f, "unknown profile `{profile}`")
            }
            Self::AmbiguousConcreteBinding { profile } => {
                write!(f, "ambiguous profile `{profile}`; use fully-qualified profile name")
            }
            Self::InvalidConcreteBinding { profile, detail } => {
                write!(f, "invalid profile binding in `{profile}`: {detail}")
            }
            Self::ProfileRequiredForBoundServiceCall {
                caller,
                binding,
                interface_type,
            } => write!(
                f,
                "bound service call `{binding}` in `{caller}` targets interface `{interface_type}` without a concrete binding"
            ),
            Self::MissingConcreteBinding {
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
            Self::InvalidAnnotation {
                service,
                annotation,
                detail,
            } => write!(
                f,
                "invalid `{annotation}` annotation on `{service}`: {detail}"
            ),
            Self::NoLowerableItems => write!(f, "no callable or pipeline declarations to lower"),
            Self::PureFnContainsEffectfulNode { fn_name, node_name } => write!(
                f,
                "node statement `{node_name}` in pure fn `{fn_name}` is not allowed (pure fns cannot contain effectful nodes)"
            ),
            Self::InvalidAuthInput {
                service,
                operation,
                field_name,
                reason,
            } => write!(
                f,
                "invalid auth_input `{field_name}` for `{service}.{operation}`: {reason}"
            ),
            Self::InvalidProviderConfigField {
                service,
                field,
                known_fields,
            } => write!(
                f,
                "unknown config field `{field}` for service `{service}`; known fields: {}",
                known_fields.join(", ")
            ),
            Self::UnknownProviderPrefix {
                service,
                fields,
                known_prefixes,
            } => write!(
                f,
                "service `{service}` has config fields {:?} but no known provider schema; \
                 known prefixes: {}",
                fields,
                known_prefixes.join(", ")
            ),
            Self::UnknownProviderSchemaType { prefix, schema } => write!(
                f,
                "provider schema mapping `{prefix}` -> `{schema}` is invalid: schema type `{schema}` not found"
            ),
            Self::MissingCallablePassthrough {
                node,
                name,
                missing_port,
            } => write!(
                f,
                "callable node `{node}` (name: `{name}`) has fn_body: None and no `{missing_port}` \
                 input port — it will fail at resolve time"
            ),
            Self::ExprLower(msg) => write!(f, "{msg}"),
            Self::WiringFailure {
                source_file,
                detail,
            } => write!(f, "{source_file}: {detail}"),
            Self::PortTypeValidation(diagnostics) => {
                writeln!(f, "port type validation failed:")?;
                for d in diagnostics {
                    writeln!(f, "  - {d}")?;
                }
                Ok(())
            }
        }
    }
}

impl From<String> for LowerError {
    fn from(s: String) -> Self {
        Self::ExprLower(s)
    }
}

impl LowerError {
    /// Return a stable diagnostic code for this error variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownPattern(..) => "LOW001",
            Self::MissingTransport { .. } => "LOW002",
            Self::InvalidAcquireBlock { .. } => "LOW003",
            Self::UnresolvedInterface { .. } => "LOW004",
            Self::UnresolvedServiceCall { .. } => "LOW005",
            Self::UnresolvedUsedResource { .. } => "LOW006",
            Self::AmbiguousUsedResource { .. } => "LOW007",
            Self::UnresolvedProvidedResource { .. } => "LOW008",
            Self::AmbiguousProvidedResource { .. } => "LOW009",
            Self::UnknownConcreteBinding { .. } => "LOW010",
            Self::AmbiguousConcreteBinding { .. } => "LOW011",
            Self::InvalidConcreteBinding { .. } => "LOW012",
            Self::ProfileRequiredForBoundServiceCall { .. } => "LOW013",
            Self::MissingConcreteBinding { .. } => "LOW014",
            Self::InvalidFileOp { .. } => "LOW015",
            Self::InvalidTransportSpec { .. } => "LOW016",
            Self::MissingProfileConfigEnv { .. } => "LOW017",
            Self::InvalidAnnotation { .. } => "LOW030",
            Self::NoLowerableItems => "LOW018",
            Self::PureFnContainsEffectfulNode { .. } => "LOW019",
            Self::InvalidAuthInput { .. } => "LOW020",
            Self::InvalidProviderConfigField { .. } => "LOW021",
            Self::UnknownProviderPrefix { .. } => "LOW022",
            Self::UnknownProviderSchemaType { .. } => "LOW023",
            Self::MissingCallablePassthrough { .. } => "LOW024",
            Self::ExprLower(..) => "LOW025",
            Self::WiringFailure { .. } => "LOW026",
            Self::PortTypeValidation(..) => "LOW027",
        }
    }

    /// Return an actionable help message for this error, if available.
    pub fn help(&self) -> Option<String> {
        match self {
            Self::MissingTransport { service, operation } => Some(format!(
                "add a `transport rest {{ method: ..., path: ... }}` or \
                 `transport shell {{ argv: [...] }}` block to `{service}.{operation}`"
            )),
            Self::UnresolvedServiceCall { service_call, .. } => Some(format!(
                "check that the service and operation in `{service_call}` are imported \
                 and spelled correctly"
            )),
            Self::UnresolvedInterface { interface } => Some(format!(
                "ensure `{interface}` is defined with `interface` keyword, or check \
                 your imports"
            )),
            Self::InvalidTransportSpec { .. } => Some(
                "verify the transport block has all required fields: `method` and \
                 `path` for REST, `argv` for shell"
                    .into(),
            ),
            Self::InvalidAnnotation { annotation, .. } => Some(format!(
                "check the `{annotation}` value in the DSL config block; see the \
                 FromStr impl for accepted values"
            )),
            Self::NoLowerableItems => Some(
                "ensure the file contains at least one `fn`, `func`, `pattern`, or \
                 `pipeline` declaration"
                    .into(),
            ),
            Self::InvalidAuthInput { field_name, .. } => Some(format!(
                "check that `{field_name}` exists in the operation's inputs and is \
                 of type `Secret`"
            )),
            Self::PortTypeValidation(_) => Some(
                "register missing types in the TypeRegistry or fix the port type_ids \
                 in the DSL source"
                    .into(),
            ),
            _ => None,
        }
    }

    /// Convert to the shared compiler diagnostic shape.
    pub fn to_diagnostic(&self) -> Diagnostic {
        let mut diagnostic =
            Diagnostic::new(self.code(), self.to_string()).with_context(self.diagnostic_context());
        if let Some(help) = self.help() {
            diagnostic = diagnostic.with_help(help);
        }
        diagnostic
    }

    fn diagnostic_context(&self) -> DiagnosticContext {
        match self {
            Self::UnresolvedInterface { interface } => DiagnosticContext::Missing {
                kind: "interface",
                name: interface.clone(),
                available: Vec::new(),
            },
            Self::UnresolvedServiceCall {
                service_call: name, ..
            }
            | Self::UnresolvedUsedResource { binding: name, .. }
            | Self::UnresolvedProvidedResource { binding: name, .. }
            | Self::MissingConcreteBinding {
                interface_type: name,
                ..
            }
            | Self::UnknownPattern(name)
            | Self::UnknownConcreteBinding { profile: name } => DiagnosticContext::Missing {
                kind: "declaration",
                name: name.clone(),
                available: Vec::new(),
            },
            Self::InvalidTransportSpec { detail, .. }
            | Self::InvalidAcquireBlock { reason: detail, .. }
            | Self::InvalidConcreteBinding { detail, .. }
            | Self::InvalidAuthInput { reason: detail, .. } => DiagnosticContext::Unsupported {
                feature: detail.clone(),
            },
            _ => DiagnosticContext::Note(String::new()),
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
    for module in project.modules() {
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

/// Configuration for the lowering pipeline.
///
/// Groups the 4 optional/boolean parameters that control which modules are
/// lowered, how collection nodes are emitted, and which profile is active.
/// Replaces the 11-function combinatorial entry point explosion (C4).
pub struct LoweringConfig<'a> {
    /// If set, only lower callables from these modules.
    pub callable_modules: Option<&'a HashSet<String>>,
    /// Emit explicit collection pipeline nodes (map/filter/fold).
    pub emit_collection_nodes: bool,
    /// Active profile name for interface stub resolution.
    pub active_profile: Option<&'a str>,
    /// Entry module for single-tool lowering.
    pub entry_module: Option<&'a str>,
    /// Permit data-only lowering to produce an empty DAG plus metadata.
    pub allow_empty_dag: bool,
    /// Type registry for cardinality inference on callable ports.
    pub type_registry: Option<&'a gunbc_ir::TypeRegistry>,
    /// Resolves environment variable references in profile config.
    ///
    /// The lowerer is a pure function — it does not read the process
    /// environment. The caller provides this callback to resolve
    /// `env("VAR")` expressions in profile bindings.
    ///
    /// Defaults to a no-op resolver (always returns `None`). Callers that
    /// need process-environment resolution must inject `std::env::var`
    /// explicitly.
    pub env_resolver: &'a dyn Fn(&str) -> Option<String>,
}

impl std::fmt::Debug for LoweringConfig<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoweringConfig")
            .field("callable_modules", &self.callable_modules)
            .field("emit_collection_nodes", &self.emit_collection_nodes)
            .field("active_profile", &self.active_profile)
            .field("entry_module", &self.entry_module)
            .field("allow_empty_dag", &self.allow_empty_dag)
            .field("type_registry", &self.type_registry)
            .field("env_resolver", &"..")
            .finish()
    }
}

/// No-op env resolver: always returns `None`. Used as the default so
/// the lowerer performs no environment I/O.
fn no_op_env_resolver(_: &str) -> Option<String> {
    None
}

impl Default for LoweringConfig<'_> {
    fn default() -> Self {
        Self {
            callable_modules: None,
            emit_collection_nodes: false,
            active_profile: None,
            entry_module: None,
            allow_empty_dag: false,
            type_registry: None,
            env_resolver: &no_op_env_resolver,
        }
    }
}

/// Bundled lower-stage outputs for callers that need more than the DAG itself.
#[derive(Debug, Clone)]
pub struct LowerOutput {
    pub dag: Dag<LoweredOp>,
    pub output_paths: Vec<String>,
    pub inferred_entrypoints: Vec<InferredEntrypoint>,
}

/// Lower a typed project with the given configuration.
pub fn lower_with_config(
    project: &TypedProject,
    config: &LoweringConfig<'_>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_to_output_with_config(project, config).map(|output| output.dag)
}

/// Lower a typed project and return the DAG plus canonical derived outputs.
pub fn lower_to_output_with_config(
    project: &TypedProject,
    config: &LoweringConfig<'_>,
) -> Result<LowerOutput, LowerError> {
    lower_typed_project_impl(
        project,
        config.callable_modules,
        config.emit_collection_nodes,
        config.active_profile,
        config.entry_module,
        config.allow_empty_dag,
        config.type_registry,
        config.env_resolver,
    )
}

pub fn lower_to_output(project: &TypedProject) -> Result<LowerOutput, LowerError> {
    lower_to_output_with_config(project, &LoweringConfig::default())
}

pub fn lower_typed_project(project: &TypedProject) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(project, &LoweringConfig::default())
}

pub fn lower_typed_project_with_profile(
    project: &TypedProject,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            active_profile,
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_with_collection_nodes(
    project: &TypedProject,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            emit_collection_nodes: true,
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_with_profile_and_collection_nodes(
    project: &TypedProject,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            active_profile,
            emit_collection_nodes: true,
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_for_modules(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            callable_modules: Some(callable_modules),
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_for_modules_with_profile(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            callable_modules: Some(callable_modules),
            active_profile,
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_for_modules_with_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            callable_modules: Some(callable_modules),
            emit_collection_nodes: true,
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_for_modules_with_profile_and_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            callable_modules: Some(callable_modules),
            active_profile,
            emit_collection_nodes: true,
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_for_modules_with_entry(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
    entry_module: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            callable_modules: Some(callable_modules),
            active_profile,
            entry_module,
            ..Default::default()
        },
    )
}

pub fn lower_typed_project_for_modules_with_entry_and_collection_nodes(
    project: &TypedProject,
    callable_modules: &HashSet<String>,
    active_profile: Option<&str>,
    entry_module: Option<&str>,
) -> Result<Dag<LoweredOp>, LowerError> {
    lower_with_config(
        project,
        &LoweringConfig {
            callable_modules: Some(callable_modules),
            active_profile,
            entry_module,
            emit_collection_nodes: true,
            ..Default::default()
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn lower_typed_project_impl(
    project: &TypedProject,
    callable_modules: Option<&HashSet<String>>,
    emit_collection_nodes: bool,
    active_profile: Option<&str>,
    entry_module: Option<&str>,
    allow_empty_dag: bool,
    type_registry: Option<&gunbc_ir::TypeRegistry>,
    env_resolver: &dyn Fn(&str) -> Option<String>,
) -> Result<LowerOutput, LowerError> {
    let mut builder = DagBuilder::new();
    let mut endpoints_by_full = HashMap::<(String, String), LoweredEndpoint>::new();
    let mut endpoints_by_name = HashMap::<String, Option<LoweredEndpoint>>::new();
    let variant_names = collect_variant_names(project);

    for module in project.modules() {
        let module_name = module.module_path.as_dotted();
        let source_file = module.path.display().to_string();
        let item_spans = module
            .ast
            .items
            .iter()
            .filter_map(|item| {
                top_level_item_name(&item.node).map(|name| (name.to_string(), item.span))
            })
            .collect::<HashMap<_, _>>();
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
                Item::FnDef(def) => {
                    let mut body = expr::lower_fn_body(&def.body, &variant_names);
                    body.param_types = def
                        .params
                        .iter()
                        .map(|p| {
                            (
                                p.name.clone(),
                                daglang_syntax::ast_utils::type_expr_to_string(&p.ty),
                            )
                        })
                        .collect();
                    body.return_type = Some(daglang_syntax::ast_utils::type_expr_to_string(
                        &def.return_type,
                    ));
                    Some((def.name.as_str(), body))
                }
                _ => None,
            })
            .collect();
        for signature in module.signatures {
            match signature {
                TypedItemSignature::Fn(callable) => {
                    if !include_callables {
                        continue;
                    }
                    let body = fn_bodies
                        .get(callable.name.as_str())
                        .map(|b| Box::new(b.clone()));
                    let origin = item_spans
                        .get(&callable.name)
                        .copied()
                        .map(|span| {
                            user_code_origin(&source_file, &module_name, &callable.name, span)
                        })
                        .unwrap_or_else(|| {
                            panic!("missing span for fn `{}` in {}", callable.name, module_name);
                        });
                    let (node, endpoint) = lower_callable(
                        callable,
                        &module_name,
                        CallableKind::Fn,
                        *interactive_by_callable
                            .get(callable.name.as_str())
                            .unwrap_or(&false),
                        body,
                        origin,
                        type_registry,
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
                    let origin = item_spans
                        .get(&callable.name)
                        .copied()
                        .map(|span| {
                            user_code_origin(&source_file, &module_name, &callable.name, span)
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "missing span for func `{}` in {}",
                                callable.name, module_name
                            );
                        });
                    let (node, endpoint) = lower_callable(
                        callable,
                        &module_name,
                        CallableKind::Func,
                        *interactive_by_callable
                            .get(callable.name.as_str())
                            .unwrap_or(&false),
                        None,
                        origin,
                        type_registry,
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
                    let origin = item_spans
                        .get(&callable.name)
                        .copied()
                        .map(|span| {
                            user_code_origin(&source_file, &module_name, &callable.name, span)
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "missing span for pattern `{}` in {}",
                                callable.name, module_name
                            );
                        });
                    let (node, endpoint) = lower_callable(
                        callable,
                        &module_name,
                        CallableKind::Pattern,
                        false,
                        None,
                        origin,
                        type_registry,
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
                    let origin = item_spans
                        .get(name)
                        .copied()
                        .map(|span| user_code_origin(&source_file, &module_name, name, span))
                        .unwrap_or_else(|| {
                            panic!("missing span for pipeline `{}` in {}", name, module_name);
                        });
                    builder.add_node(
                        Node::opaque(
                            node_id,
                            vec![],
                            vec![Port::scalar("stages", "Int")],
                            LoweredOp::Pipeline {
                                module: module_name.clone(),
                                name: name.clone(),
                                stages: *stages,
                                stage_names: stage_names.clone(),
                            },
                        )
                        .with_origin(origin),
                    );
                }
                TypedItemSignature::Type { .. }
                | TypedItemSignature::Service { .. }
                | TypedItemSignature::Resource { .. }
                | TypedItemSignature::Interface { .. } => {}
            }
        }
    }

    let transport_manifest = if callable_modules.is_some() && active_profile.is_none() {
        let required_service_calls = collect_required_service_call_keys(project, callable_modules);
        derive_service_transport_triplets(project, Some(&required_service_calls))?
    } else {
        derive_service_transport_triplets(project, None)?
    };
    builder.apply_manifest(&transport_manifest);
    let mut service_registry = transport_manifest.registry;
    let data_values = build_data_values(project);
    let callable_param_defaults = collect_callable_param_defaults(project);
    let wctx = DagWiringContext {
        endpoints_by_full: &endpoints_by_full,
        endpoints_by_name: &endpoints_by_name,
        service_registry: &service_registry,
        data_values: &data_values,
        variant_names: &variant_names,
        callable_param_defaults: &callable_param_defaults,
    };
    add_dependency_edges(
        &mut builder,
        project,
        &wctx,
        emit_collection_nodes,
        entry_module,
    );
    let profile_registry = collect_profile_binding_registry(project, active_profile)?;
    let active_profile_bindings =
        resolve_active_profile_bindings(&profile_registry, active_profile, env_resolver)?;
    let profile_bound_interfaces = collect_profile_bound_interface_names(&profile_registry);
    // IS-3: Collect interfaces needing stub transport.
    let stub_interfaces =
        interfaces_needing_stubs(project, active_profile, &profile_bound_interfaces);
    // IS-4: Derive stub transport triplets so resolve_service_call_source can find them.
    let stub_manifest = derive_interface_stub_transport_triplets(project, &stub_interfaces);
    builder.apply_manifest(&stub_manifest);
    service_registry.merge(stub_manifest.registry);
    let known_interface_types = collect_interface_type_names(project);
    let wctx = DagWiringContext {
        endpoints_by_full: &endpoints_by_full,
        endpoints_by_name: &endpoints_by_name,
        service_registry: &service_registry,
        data_values: &data_values,
        variant_names: &variant_names,
        callable_param_defaults: &callable_param_defaults,
    };
    add_service_call_edges(
        &mut builder,
        project,
        &wctx,
        active_profile_bindings.as_ref(),
        &profile_bound_interfaces,
        &known_interface_types,
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

    if builder.dag.nodes.is_empty() && !allow_empty_dag {
        return Err(LowerError::NoLowerableItems);
    }

    // Embed data declaration values as CallLiteralSource nodes in the DAG.
    // The resolver extracts these at resolution time, eliminating the need
    // to thread data_values through CompileOutput / CachedCompileData.
    embed_data_declaration_nodes(&mut builder, &data_values);

    wire_param_source_inputs(&mut builder);

    let mut dag = builder.into_dag();
    stamp_node_kinds(&mut dag);
    validate_callable_output_wiring(&dag)?;

    // Validation (cardinality, port type IDs, structural invariants) is
    // performed post-lowering by `VerifiedDag::verify()` in the pipeline.
    // That is the single validation authority — lowering produces the DAG,
    // verification gates it. Port type-id validation against a full registry
    // (including DSL-defined types) is deferred to S18.
    let _ = type_registry;

    let output_paths = extract_output_paths(&dag);
    let inferred_entrypoints = infer_entrypoints(&dag);
    Ok(LowerOutput {
        dag,
        output_paths,
        inferred_entrypoints,
    })
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
                    gunbc_ir::node::NodeBody::SubDag(inner, _) => {
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
            gunbc_ir::node::NodeBody::SubDag(..) => "subdag".to_string(),
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline { .. }) => {
                canonical_kind_from_shape(&node.id.0, &node.inputs, &node.outputs, true, None)
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection { kind, .. }) => {
                kind.node_label().to_string()
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable { obligation, .. }) => {
                canonical_kind_from_shape(
                    &node.id.0,
                    &node.inputs,
                    &node.outputs,
                    false,
                    Some(ObligationCategory::from(*obligation)),
                )
            }
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Transport { obligation, .. }) => {
                canonical_kind_from_shape(
                    &node.id.0,
                    &node.inputs,
                    &node.outputs,
                    false,
                    Some(ObligationCategory::from(*obligation)),
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
        }
    }

    fn canonical_label_lowered(node: &Node<LoweredOp>) -> String {
        node.id.0.clone()
    }

    fn canonical_kind_reference<T>(node: &Node<T>) -> String {
        match &node.body {
            gunbc_ir::node::NodeBody::SubDag(..) => "subdag".to_string(),
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
        if candidate_ids.contains("gunbc.auth.patterns::acquire_subject_token") {
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
        if candidate_ids.contains("gunbc.auth.patterns::optional_impersonation") {
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
        if candidate_ids.contains("gunbc.auth.patterns::credential_chain") {
            canonical_nodes.insert("build_credential".to_string());
        }
        build_gcp_credential_canonical_graph(&canonical_nodes, |id| LoweredOp::Callable {
            module: "parity.gcp_credential".to_string(),
            kind: CallableKind::Pattern,
            name: id.to_string(),
            obligation: CallableObligation::None,
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
                    Port::scalar("secret", "String"),
                    Port::scalar("scheme", "String"),
                    Port::optional("header_name", "Optional<String>"),
                    Port::scalar("source_id", "String"),
                    Port::list("required_scopes", "String"),
                ],
                vec![Port::scalar("credential", "Credential")],
            ),
            (
                "net_env",
                vec![],
                vec![Port::scalar("api:network", "NetworkHandle")],
            ),
            (
                "prepare_github_oidc",
                vec![
                    Port::scalar("audience", "String"),
                    Port::optional("request_token", "Optional<String>"),
                    Port::optional("request_url", "Optional<String>"),
                ],
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                ],
            ),
            (
                "execute_github_oidc",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                    Port::resource("api:network", "NetworkHandle", AccessMode::Read),
                ],
                vec![Port::scalar("response", "TransportResponse")],
            ),
            (
                "parse_github_oidc",
                vec![Port::scalar("response", "TransportResponse")],
                vec![Port::scalar("subject_token", "String")],
            ),
            (
                "prepare_sts",
                vec![
                    Port::scalar("subject_token", "String"),
                    Port::scalar("audience", "String"),
                ],
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                ],
            ),
            (
                "execute_sts",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                    Port::resource("api:network", "NetworkHandle", AccessMode::Read),
                ],
                vec![Port::scalar("response", "TransportResponse")],
            ),
            (
                "parse_sts",
                vec![Port::scalar("response", "TransportResponse")],
                vec![
                    Port::scalar("access_token", "String"),
                    Port::scalar("expires_in", "Int"),
                ],
            ),
            (
                "should_impersonate",
                vec![Port::scalar("service_account", "String")],
                vec![Port::scalar("should", "Bool")],
            ),
            (
                "prepare_impersonate",
                vec![
                    Port::scalar("access_token", "String"),
                    Port::scalar("service_account", "String"),
                    Port::optional("lifetime_seconds", "Optional<Int>"),
                    Port::optional("should_impersonate", "Optional<Bool>"),
                ],
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                ],
            ),
            (
                "execute_impersonate",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                    Port::resource("api:network", "NetworkHandle", AccessMode::Read),
                ],
                vec![Port::scalar("response", "TransportResponse")],
            ),
            (
                "parse_impersonate",
                vec![
                    Port::scalar("response", "TransportResponse"),
                    Port::optional("base_access_token", "Optional<String>"),
                ],
                vec![Port::scalar("access_token", "String")],
            ),
            (
                "prepare_secret_access",
                vec![
                    Port::scalar("access_token", "String"),
                    Port::scalar("project", "String"),
                    Port::scalar("secret", "String"),
                    Port::optional("version", "Optional<String>"),
                ],
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                ],
            ),
            (
                "execute_secret_access",
                vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::scalar("skip", "Bool"),
                    Port::resource("api:network", "NetworkHandle", AccessMode::Read),
                ],
                vec![Port::scalar("response", "TransportResponse")],
            ),
            (
                "parse_secret_access",
                vec![Port::scalar("response", "TransportResponse")],
                vec![Port::scalar("secret", "String")],
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

/// Infer an obligation category for `fn` items based on output port type structure.
///
/// Only applies to `CallableKind::Fn` (pure functions). `Func`/`Pattern` callables
/// keep `ObligationCategory::None` (they are classified structurally elsewhere).
///
/// # Classification rules (S14 — structural, not name-prefix)
///
/// 1. **ResourceProvide**: any fn whose output type contains "Handle" or "Env"
///    (structural indicator of resource provision).
/// 2. **PureDataLoad**: fn with zero user-facing inputs and a single `String`
///    output (structural indicator of data loading / constant).
/// 3. **PureRender**: fn with at least one user-facing input and a single
///    `String` output (structural indicator of template/render).
/// 4. **PureGeneric**: everything else.
fn infer_fn_obligation(
    _name: &str,
    kind: CallableKind,
    user_param_count: usize,
    outputs: &[Port],
) -> CallableObligation {
    if kind != CallableKind::Fn {
        return CallableObligation::None;
    }

    // Rule 1: Output type contains "Handle" or "Env" → resource provider.
    let has_handle_output = outputs.iter().any(|p| {
        let ty = p.type_id.0.as_str();
        ty.contains("Handle") || ty.contains("Env")
    });
    if has_handle_output {
        return CallableObligation::ResourceProvide;
    }

    // Rules 2 & 3: Single String output, distinguished by input count.
    let is_string_output = outputs.len() == 1 && outputs[0].type_id.0 == "String";
    if is_string_output {
        if user_param_count == 0 {
            // Zero user inputs + String output → data loader / constant.
            return CallableObligation::PureDataLoad;
        }
        // Non-zero user inputs + String output → renderer / template.
        return CallableObligation::PureRender;
    }

    CallableObligation::PureGeneric
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
    origin: NodeOrigin,
    _type_registry: Option<&gunbc_ir::TypeRegistry>,
) -> (Node<LoweredOp>, LoweredEndpoint) {
    let node_id = lowered_node_id(module_name, &callable.name);
    let outputs = if callable.outputs.is_empty() {
        vec![Port::scalar("return", "Unit")]
    } else {
        callable
            .outputs
            .iter()
            .map(|binding| {
                // Callable outputs are always scalar (ONE cardinality).
                // List<T> describes the value shape, not the port cardinality.
                // Port::typed would incorrectly infer ZERO_OR_MORE for List<T>,
                // causing downstream WrapScalar coercion mismatches.
                Port::scalar(binding.name.as_str(), binding.ty.as_str())
            })
            .collect()
    };
    // Callable inputs are always scalar (ONE cardinality).
    // The fn body evaluator works with single values — List<T> is a single
    // Value::List, not a multi-value port. Using Port::typed here would
    // incorrectly infer ZERO_OR_MORE, causing auto_mock to double-wrap lists.
    let mut inputs = callable
        .params
        .iter()
        .map(|binding| Port::scalar(binding.name.as_str(), binding.ty.as_str()))
        .collect::<Vec<_>>();
    for output in &outputs {
        inputs.push(Port::scalar(
            output_passthrough_input_name(output.name.0.as_str()),
            output.type_id.0.as_str(),
        ));
    }
    inputs.push(Port::list(PortName::DEPS, "Any"));
    let obligation = infer_fn_obligation(&callable.name, kind, callable.params.len(), &outputs);
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
                is_interactive,
                resource_target: None,
                fn_body,
            },
        )
        .with_origin(origin),
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
    wctx: &DagWiringContext<'_>,
    emit_collection_nodes: bool,
    entry_module: Option<&str>,
) {
    for module in project.modules() {
        let module_name = module.module_path.as_dotted();
        let source_file = module.path.display().to_string();
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
                        .map(|param| (param.name.clone(), param.ty.0.clone()))
                        .collect::<HashMap<_, _>>(),
                )),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        for item in &module.ast.items {
            let Some((item_name, stmts)) = item_callable_body(&item.node) else {
                continue;
            };
            let Some(target) = wctx
                .endpoints_by_full
                .get(&(module_name.clone(), item_name.to_string()))
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
                let Some(Some(source)) = wctx.endpoints_by_name.get(&call) else {
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
                let empty_callables = HashMap::new();
                let empty_services = HashMap::new();
                let empty_expanded = HashMap::new();
                let empty_locals = HashMap::new();
                let empty_fn_bodies = std::collections::BTreeMap::new();
                let empty_endpoints_full = HashMap::new();
                let empty_uses = HashMap::new();
                let ctx = LoweringContext {
                    source_file: &source_file,
                    module_name: &module_name,
                    item_name,
                    item_span: item.span,
                    param_types: &param_types,
                    endpoints_by_name: wctx.endpoints_by_name,
                    data_values: wctx.data_values,
                    service_registry: wctx.service_registry,
                    bound_callable_sources: &empty_callables,
                    bound_service_sources: &empty_services,
                    expanded_results: &empty_expanded,
                    local_let_bindings: &empty_locals,
                    body_stmts: &[],
                    all_fn_bodies: &empty_fn_bodies,
                    variant_names: wctx.variant_names,
                    callable_param_defaults: wctx.callable_param_defaults,
                    endpoints_by_full: &empty_endpoints_full,
                    uses_binding_types: &empty_uses,
                };
                let _ = expand_content_upsert_patterns(builder, &ctx, stmts, target);
                expand_non_generic_pattern_calls(builder, project, &ctx, stmts, target);
            }
            if emit_collection_nodes {
                add_collection_pipeline_nodes(builder, &module_name, stmts, target);
            }
            // Skip control flow SubDag nodes for fn items with fn_body.
            // FnBodyCallableOp evaluates control flow (match/if/for) directly —
            // creating redundant SubDag pattern nodes causes failures because
            // their inner op nodes lack __out:result passthrough wiring (which
            // can't be threaded through nested SubDag boundaries).
            let has_fn_body = matches!(&item.node, Item::FnDef(_));
            if !has_fn_body {
                let uses_binding_types = item_uses_binding_types(&item.node);
                let empty_fn_bodies = std::collections::BTreeMap::new();
                let control_flow_ctx = ControlFlowPatternContext {
                    source_file: &source_file,
                    module_name: &module_name,
                    item_name,
                    item_span: item.span,
                    stmts,
                    target,
                    endpoints_by_name: wctx.endpoints_by_name,
                    data_values: wctx.data_values,
                    service_registry: wctx.service_registry,
                    uses_binding_types: &uses_binding_types,
                    all_fn_bodies: &empty_fn_bodies,
                    variant_names: wctx.variant_names,
                    callable_param_defaults: wctx.callable_param_defaults,
                };
                add_control_flow_pattern_nodes(builder, &control_flow_ctx);
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
    body_stmts: Vec<Stmt>,
    /// Service call paths found inside the for-loop body expression.
    /// Each entry is the dot-separated path (e.g., `["fs", "read"]`).
    body_service_call_paths: Vec<Vec<String>>,
}

/// Detected if/else branch with service call paths per branch.
/// Populated from `scope::ScopedBody` via `collect_if_sites_from_scoped`.
#[derive(Debug)]
struct IfBranchSite {
    has_else: bool,
    then_service_call_paths: Vec<Vec<String>>,
    else_service_call_paths: Vec<Vec<String>>,
}

/// Detected match expression with per-arm service call paths.
/// Populated from `scope::ScopedBody` via `collect_match_sites_from_scoped`.
#[derive(Debug)]
struct MatchBranchSite {
    arm_count: usize,
    /// Flattened union of all per-arm paths (for top-level dedup in add_service_call_edges).
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
                body_stmts,
                body,
            } => {
                let iterable_ref = match iterable {
                    scope::ExprRef::Ident(name) => Some(IterableRef::Ident(name.clone())),
                    scope::ExprRef::FieldAccess { base, field } => {
                        Some(IterableRef::FieldAccess(base.clone(), field.clone()))
                    }
                    scope::ExprRef::Opaque => None,
                };
                out.push(ForLoopSite {
                    element_var: element_var.clone(),
                    iterable: iterable_ref,
                    passthrough: passthrough.clone(),
                    body_stmts: body_stmts.clone(),
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
            | scope::ScopedItem::FnCall
            | scope::ScopedItem::Binding => {}
        }
    }
}

fn collect_if_sites_from_scoped(body: &scope::ScopedBody, out: &mut Vec<IfBranchSite>) {
    for item in &body.items {
        match item {
            scope::ScopedItem::IfBranch {
                then_body,
                else_body,
            } => {
                out.push(IfBranchSite {
                    has_else: else_body.is_some(),
                    then_service_call_paths: collect_service_paths_from_scoped_body(then_body),
                    else_service_call_paths: else_body
                        .as_ref()
                        .map(collect_service_paths_from_scoped_body)
                        .unwrap_or_default(),
                });
                collect_if_sites_from_scoped(then_body, out);
                if let Some(else_body) = else_body {
                    collect_if_sites_from_scoped(else_body, out);
                }
            }
            scope::ScopedItem::ForLoop { body, .. } => {
                collect_if_sites_from_scoped(body, out);
            }
            scope::ScopedItem::MatchBranch { arms } => {
                for arm in arms {
                    collect_if_sites_from_scoped(&arm.body, out);
                }
            }
            scope::ScopedItem::ServiceCall(_)
            | scope::ScopedItem::FnCall
            | scope::ScopedItem::Binding => {}
        }
    }
}

fn collect_match_sites_from_scoped(body: &scope::ScopedBody, out: &mut Vec<MatchBranchSite>) {
    for item in &body.items {
        match item {
            scope::ScopedItem::MatchBranch { arms } => {
                let mut all_paths = Vec::new();
                for arm in arms {
                    let arm_paths = collect_service_paths_from_scoped_body(&arm.body);
                    all_paths.extend(arm_paths.clone());
                }
                out.push(MatchBranchSite {
                    arm_count: arms.len(),
                    all_service_call_paths: all_paths,
                });
                for arm in arms {
                    collect_match_sites_from_scoped(&arm.body, out);
                }
            }
            scope::ScopedItem::IfBranch {
                then_body,
                else_body,
            } => {
                collect_match_sites_from_scoped(then_body, out);
                if let Some(else_body) = else_body {
                    collect_match_sites_from_scoped(else_body, out);
                }
            }
            scope::ScopedItem::ForLoop { body, .. } => {
                collect_match_sites_from_scoped(body, out);
            }
            scope::ScopedItem::ServiceCall(_)
            | scope::ScopedItem::FnCall
            | scope::ScopedItem::Binding => {}
        }
    }
}

fn detect_for_loops_in_stmts(stmts: &[Stmt]) -> Vec<ForLoopSite> {
    let scoped = scope::ScopedBody::from_stmts(stmts);
    let mut sites = Vec::new();
    collect_for_loop_sites_from_scoped(&scoped, &mut sites);
    sites
}

fn detect_if_branches_in_stmts(stmts: &[Stmt]) -> Vec<IfBranchSite> {
    let scoped = scope::ScopedBody::from_stmts(stmts);
    let mut sites = Vec::new();
    collect_if_sites_from_scoped(&scoped, &mut sites);
    sites
}

fn detect_match_branches_in_stmts(stmts: &[Stmt]) -> Vec<MatchBranchSite> {
    let scoped = scope::ScopedBody::from_stmts(stmts);
    let mut sites = Vec::new();
    collect_match_sites_from_scoped(&scoped, &mut sites);
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

fn resolve_loop_body_service_endpoint(
    call_path: &[String],
    uses_binding_types: &HashMap<String, String>,
    service_registry: &ServiceEndpointRegistry,
) -> Option<ServiceTransportEndpoint> {
    if let Some(endpoint) = resolve_service_endpoint(call_path, service_registry) {
        return Some(endpoint);
    }
    let binding = call_path.first()?;
    let resource_type = uses_binding_types.get(binding)?;
    if call_path.len() >= 2 {
        let capability = call_path.last()?;
        let cap_key = format!("{resource_type}.{capability}");
        let cap_path: Vec<String> = cap_key.split('.').map(String::from).collect();
        return resolve_service_endpoint(&cap_path, service_registry);
    }
    None
}

fn clone_loop_body_callable_node(
    source_dag: &Dag<LoweredOp>,
    endpoint: &LoweredEndpoint,
    suffix: &str,
) -> Option<(Node<LoweredOp>, LoweredEndpoint)> {
    let mut node = source_dag
        .get_node(&NodeId::new(endpoint.node_id.clone()))?
        .clone();
    let node_id = format!("{}_{}", endpoint.node_id, suffix);
    node.id = NodeId::new(node_id.clone());
    node.static_fingerprint = None;
    Some((
        node,
        LoweredEndpoint {
            node_id,
            primary_output: endpoint.primary_output.clone(),
        },
    ))
}

fn clone_loop_body_transport_triplet(
    builder: &mut DagBuilder,
    source_dag: &Dag<LoweredOp>,
    endpoint: &ServiceTransportEndpoint,
    suffix: &str,
) -> Option<ServiceTransportEndpoint> {
    let new_prepare_id = format!("{}_{suffix}", endpoint.prepare_node_id);
    let new_execute_id = format!("{}_{suffix}", endpoint.execute_node_id);
    let new_parse_id = format!("{}_{suffix}", endpoint.parse.node_id);

    let mut prepare = source_dag
        .get_node(&NodeId::new(endpoint.prepare_node_id.clone()))?
        .clone();
    prepare.id = NodeId::new(new_prepare_id.clone());
    builder.add_node(prepare);

    let mut execute = source_dag
        .get_node(&NodeId::new(endpoint.execute_node_id.clone()))?
        .clone();
    execute.id = NodeId::new(new_execute_id.clone());
    execute.static_fingerprint = None;
    builder.add_node(execute);

    let mut parse = source_dag
        .get_node(&NodeId::new(endpoint.parse.node_id.clone()))?
        .clone();
    parse.id = NodeId::new(new_parse_id.clone());
    builder.add_node(parse);

    builder.add_edge(&new_prepare_id, "request", &new_execute_id, "request");
    builder.add_edge(&new_execute_id, "response", &new_parse_id, "response");

    Some(ServiceTransportEndpoint {
        prepare_node_id: new_prepare_id,
        execute_node_id: new_execute_id,
        parse: LoweredEndpoint {
            node_id: new_parse_id,
            primary_output: endpoint.parse.primary_output.clone(),
        },
        prepare_inputs: endpoint.prepare_inputs.clone(),
        operation_inputs: endpoint.operation_inputs.clone(),
        has_auth: endpoint.has_auth,
        metadata: endpoint.metadata.clone(),
    })
}

fn wire_loop_body_named_args(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    args: &[(Option<String>, Expr)],
    dest_node_id: &str,
) {
    let empty_arg_map = HashMap::new();
    let empty_node_outputs = HashMap::new();
    for (arg_name, arg_expr) in args {
        let Some(arg_name) = arg_name.as_deref() else {
            continue;
        };
        wire_pattern_arg_to_prepare(
            builder,
            ctx,
            arg_name,
            arg_expr,
            dest_node_id,
            &empty_arg_map,
            &empty_node_outputs,
        );
    }
}

fn make_loop_body_dag_from_stmts(
    source_dag: &Dag<LoweredOp>,
    ctx: &ControlFlowPatternContext<'_>,
    callable_node_id: &str,
    index: usize,
    site: &ForLoopSite,
) -> Option<Dag<LoweredOp>> {
    let mut builder = DagBuilder::new();
    let mut param_types = HashMap::<String, String>::new();
    param_types.insert(site.element_var.clone(), "Any".to_string());
    for passthrough in &site.passthrough {
        param_types.insert(passthrough.clone(), "Any".to_string());
    }

    let mut body_inputs = vec![Port::scalar(site.element_var.as_str(), "Any")];
    for passthrough in &site.passthrough {
        body_inputs.push(Port::scalar(passthrough.as_str(), "Any"));
    }
    body_inputs.push(Port::scalar(output_passthrough_input_name("result"), "Any"));
    body_inputs.push(Port::list(PortName::DEPS, "Any"));

    let body_target = LoweredEndpoint {
        node_id: "body_op".to_string(),
        primary_output: "result".to_string(),
    };
    builder.add_node(Node::opaque(
        body_target.node_id.clone(),
        body_inputs,
        vec![Port::scalar("result", "Any")],
        LoweredOp::Callable {
            module: ctx.module_name.to_string(),
            kind: CallableKind::Func,
            name: format!("{callable_node_id}::for_{index}_body"),
            obligation: CallableObligation::None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        },
    ));

    let empty_endpoints = HashMap::<String, Option<LoweredEndpoint>>::new();
    let empty_data = HashMap::<String, serde_json::Value>::new();
    let mut bound_callable_sources = HashMap::<String, LoweredEndpoint>::new();
    let mut bound_service_sources = HashMap::<String, ServiceTransportEndpoint>::new();
    let empty_expanded = HashMap::<String, PatternExpansionResult>::new();
    let empty_locals = HashMap::new();
    let empty_endpoints_full = HashMap::new();

    for (stmt_index, stmt) in site.body_stmts.iter().enumerate() {
        let (binding_name, expr) = match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => (Some(name.as_str()), expr),
            Stmt::Node(node_stmt) => (Some(node_stmt.name.as_str()), &node_stmt.expr),
            Stmt::Expr(_) | Stmt::Return(_) => (
                None,
                match stmt {
                    Stmt::Expr(expr) => expr,
                    _ => continue,
                },
            ),
        };

        match expr {
            Expr::Call(call_name, args) if call_name != "content_upsert" => {
                let Some(Some(endpoint)) = ctx.endpoints_by_name.get(call_name.as_str()) else {
                    continue;
                };
                let suffix = format!("body_call_{index}_{stmt_index}");
                let Some((node, cloned_endpoint)) =
                    clone_loop_body_callable_node(source_dag, endpoint, &suffix)
                else {
                    continue;
                };
                builder.add_node(node);
                let arg_ctx = LoweringContext {
                    source_file: ctx.source_file,
                    module_name: ctx.module_name,
                    item_name: ctx.item_name,
                    item_span: ctx.item_span,
                    param_types: &param_types,
                    endpoints_by_name: &empty_endpoints,
                    data_values: ctx.data_values,
                    service_registry: ctx.service_registry,
                    bound_callable_sources: &bound_callable_sources,
                    bound_service_sources: &bound_service_sources,
                    expanded_results: &empty_expanded,
                    local_let_bindings: &empty_locals,
                    body_stmts: site.body_stmts.as_slice(),
                    all_fn_bodies: ctx.all_fn_bodies,
                    variant_names: ctx.variant_names,
                    callable_param_defaults: ctx.callable_param_defaults,
                    endpoints_by_full: &empty_endpoints_full,
                    uses_binding_types: ctx.uses_binding_types,
                };
                wire_loop_body_named_args(
                    &mut builder,
                    &arg_ctx,
                    args,
                    cloned_endpoint.node_id.as_str(),
                );
                if let Some(binding_name) = binding_name {
                    bound_callable_sources.insert(binding_name.to_string(), cloned_endpoint);
                }
            }
            Expr::ServiceCall(path, args) => {
                let Some(endpoint) = resolve_loop_body_service_endpoint(
                    path,
                    ctx.uses_binding_types,
                    ctx.service_registry,
                ) else {
                    continue;
                };
                let suffix = format!("body_transport_{index}_{stmt_index}");
                let Some(cloned_endpoint) =
                    clone_loop_body_transport_triplet(&mut builder, source_dag, &endpoint, &suffix)
                else {
                    continue;
                };
                let arg_ctx = LoweringContext {
                    source_file: ctx.source_file,
                    module_name: ctx.module_name,
                    item_name: ctx.item_name,
                    item_span: ctx.item_span,
                    param_types: &param_types,
                    endpoints_by_name: &empty_endpoints,
                    data_values: &empty_data,
                    service_registry: ctx.service_registry,
                    bound_callable_sources: &bound_callable_sources,
                    bound_service_sources: &bound_service_sources,
                    expanded_results: &empty_expanded,
                    local_let_bindings: &empty_locals,
                    body_stmts: site.body_stmts.as_slice(),
                    all_fn_bodies: ctx.all_fn_bodies,
                    variant_names: ctx.variant_names,
                    callable_param_defaults: ctx.callable_param_defaults,
                    endpoints_by_full: &empty_endpoints_full,
                    uses_binding_types: ctx.uses_binding_types,
                };
                wire_loop_body_named_args(
                    &mut builder,
                    &arg_ctx,
                    args,
                    cloned_endpoint.prepare_node_id.as_str(),
                );
                if let Some(binding_name) = binding_name {
                    bound_service_sources.insert(binding_name.to_string(), cloned_endpoint);
                }
            }
            _ => {}
        }
    }

    let expansion_ctx = LoweringContext {
        source_file: ctx.source_file,
        module_name: ctx.module_name,
        item_name: ctx.item_name,
        item_span: ctx.item_span,
        param_types: &param_types,
        endpoints_by_name: &empty_endpoints,
        data_values: ctx.data_values,
        service_registry: ctx.service_registry,
        bound_callable_sources: &bound_callable_sources,
        bound_service_sources: &bound_service_sources,
        expanded_results: &empty_expanded,
        local_let_bindings: &empty_locals,
        body_stmts: site.body_stmts.as_slice(),
        all_fn_bodies: ctx.all_fn_bodies,
        variant_names: ctx.variant_names,
        callable_param_defaults: ctx.callable_param_defaults,
        endpoints_by_full: &empty_endpoints_full,
        uses_binding_types: ctx.uses_binding_types,
    };
    let expanded_results = expand_content_upsert_patterns(
        &mut builder,
        &expansion_ctx,
        site.body_stmts.as_slice(),
        &body_target,
    );
    let local_let_bindings =
        collect_local_let_bindings(site.body_stmts.as_slice(), &bound_callable_sources);
    let return_ctx = LoweringContext {
        expanded_results: &expanded_results,
        local_let_bindings: &local_let_bindings,
        ..expansion_ctx
    };
    if wire_callable_return_outputs(
        &mut builder,
        &return_ctx,
        site.body_stmts.as_slice(),
        &body_target,
    )
    .is_err()
    {
        // Loop body return wiring failed — cannot construct body DAG.
        return None;
    }

    if let Some(element_type) = builder
        .dag
        .nodes
        .iter()
        .flat_map(|node| node.inputs.iter())
        .find(|port| port.name.0 == site.element_var && port.type_id.0 != "Any")
        .map(|port| port.type_id.0.clone())
    {
        if let Some(body_node) = builder
            .dag
            .get_node_mut(&NodeId::new(body_target.node_id.clone()))
        {
            for port in &mut body_node.inputs {
                if port.name.0 == site.element_var {
                    port.type_id = element_type.as_str().into();
                }
            }
        }
    }

    if !builder.has_edge_to_port(body_target.node_id.as_str(), "__out:result") {
        return None;
    }

    Some(builder.dag)
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
        // Provide a trivial fn_body that passes element through as result,
        // so FnBodyCallableOp handles it (avoids __out:result passthrough requirement).
        dag.add_node(Node::opaque(
            "body_op",
            inputs,
            vec![Port::scalar("result", "Any")],
            LoweredOp::Callable {
                module: module_name.to_string(),
                kind: CallableKind::Fn,
                name: format!("{callable_node_id}::for_{index}_body"),
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: Some(Box::new(expr::LoweredFnBody::from_stmts(vec![
                    expr::LoweredStmt::Return(vec![(
                        "result".to_string(),
                        expr::LoweredExpr::Ident(element_var.to_string()),
                    )]),
                ]))),
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
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: Some(Box::new(expr::LoweredFnBody::from_stmts(vec![
                    expr::LoweredStmt::Return(vec![(
                        "result".to_string(),
                        expr::LoweredExpr::Ident(last_parse_output.clone()),
                    )]),
                ]))),
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

            let triplet_spec = transport::TransportTripletSpec {
                module: module_name.to_string(),
                service: transport.metadata.service.clone(),
                operation: transport.metadata.operation.clone(),
                metadata: transport.metadata.clone(),
                prepare_id: prepare_id.clone(),
                execute_id: execute_id.clone(),
                parse_id: parse_id.clone(),
                prepare_inputs: prepare_ports,
                execute_extra_inputs: vec![],
                parse_outputs: vec![Port::scalar(transport.parse_output.as_str(), "Any")],
                execute_parse_wiring: transport::ExecuteParseWiring::Response,
                origin: None,
                operation_key: None,
            };
            transport::emit_triplet_to_dag(
                &mut dag,
                transport::build_transport_triplet(triplet_spec),
            );
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
        // Provide a trivial fn_body that passes input through as result,
        // so FnBodyCallableOp handles it (avoids __out:result passthrough requirement).
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
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: Some(Box::new(expr::LoweredFnBody::from_stmts(vec![
                    expr::LoweredStmt::Return(vec![(
                        "result".to_string(),
                        expr::LoweredExpr::Ident("input".to_string()),
                    )]),
                ]))),
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
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: Some(Box::new(expr::LoweredFnBody::from_stmts(vec![
                    expr::LoweredStmt::Return(vec![(
                        "result".to_string(),
                        expr::LoweredExpr::Ident(last_parse_output.clone()),
                    )]),
                ]))),
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

            let triplet_spec = transport::TransportTripletSpec {
                module: module_name.to_string(),
                service: transport.metadata.service.clone(),
                operation: transport.metadata.operation.clone(),
                metadata: transport.metadata.clone(),
                prepare_id: prepare_id.clone(),
                execute_id: execute_id.clone(),
                parse_id: parse_id.clone(),
                prepare_inputs: prepare_ports,
                execute_extra_inputs: vec![],
                parse_outputs: vec![Port::scalar(transport.parse_output.as_str(), "Any")],
                execute_parse_wiring: transport::ExecuteParseWiring::Response,
                origin: None,
                operation_key: None,
            };
            transport::emit_triplet_to_dag(
                &mut dag,
                transport::build_transport_triplet(triplet_spec),
            );
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

struct ControlFlowPatternContext<'a> {
    source_file: &'a str,
    module_name: &'a str,
    item_name: &'a str,
    item_span: SyntaxSpan,
    stmts: &'a [Stmt],
    target: &'a LoweredEndpoint,
    endpoints_by_name: &'a HashMap<String, Option<LoweredEndpoint>>,
    data_values: &'a HashMap<String, serde_json::Value>,
    service_registry: &'a ServiceEndpointRegistry,
    uses_binding_types: &'a HashMap<String, String>,
    all_fn_bodies: &'a std::collections::BTreeMap<String, LoweredFnBody>,
    variant_names: &'a HashSet<String>,
    callable_param_defaults: &'a HashMap<String, Vec<(String, daglang_syntax::ast::Expr)>>,
}

fn add_control_flow_pattern_nodes(builder: &mut DagBuilder, ctx: &ControlFlowPatternContext<'_>) {
    let source_dag = builder.dag.clone();
    let for_sites = detect_for_loops_in_stmts(ctx.stmts);
    for (index, site) in for_sites.iter().enumerate() {
        let node_id = format!("{}::cf_for_{index}", ctx.target.node_id);
        // Resolve body service calls to LoopBodyTransport entries.
        let mut body_transports = Vec::new();
        for call_path in &site.body_service_call_paths {
            if let Some(transport) = resolve_loop_body_service_call(
                call_path,
                ctx.uses_binding_types,
                ctx.service_registry,
            ) {
                body_transports.push(transport);
            }
        }
        let body_dag =
            make_loop_body_dag_from_stmts(&source_dag, ctx, &ctx.target.node_id, index, site)
                .unwrap_or_else(|| {
                    make_loop_body_dag(
                        ctx.module_name,
                        &ctx.target.node_id,
                        index,
                        &site.element_var,
                        &site.passthrough,
                        &body_transports,
                    )
                });
        let element_type = body_dag
            .nodes
            .iter()
            .flat_map(|node| node.inputs.iter())
            .find(|port| port.name.0 == site.element_var && port.type_id.0 != "Any")
            .or_else(|| {
                body_dag
                    .nodes
                    .iter()
                    .flat_map(|node| node.inputs.iter())
                    .find(|port| port.name.0 == site.element_var)
            })
            .map(|port| port.type_id.0.clone())
            .unwrap_or_else(|| "Any".to_string());
        let loop_node = LoopBuilder::new(node_id.clone())
            .with_input("items", "Any", Cardinality::ONE)
            .with_element(&site.element_var, element_type.as_str())
            .with_body(body_dag)
            .with_output("result", "Any")
            .build();
        builder.add_node(loop_node.with_origin(pattern_expansion_origin(
            ctx.source_file,
            ctx.module_name,
            ctx.item_name,
            ctx.item_span,
            "for_loop",
        )));
        builder.add_edge(&node_id, "result", &ctx.target.node_id, PortName::DEPS);
    }

    let if_sites = detect_if_branches_in_stmts(ctx.stmts);
    for (index, site) in if_sites.iter().enumerate() {
        let node_id = format!("{}::cf_if_{index}", ctx.target.node_id);
        // Resolve branch-body service calls to transport entries.
        let mut then_transports = Vec::new();
        for call_path in &site.then_service_call_paths {
            if let Some(transport) = resolve_loop_body_service_call(
                call_path,
                ctx.uses_binding_types,
                ctx.service_registry,
            ) {
                then_transports.push(transport);
            }
        }
        let mut else_transports = Vec::new();
        for call_path in &site.else_service_call_paths {
            if let Some(transport) = resolve_loop_body_service_call(
                call_path,
                ctx.uses_binding_types,
                ctx.service_registry,
            ) {
                else_transports.push(transport);
            }
        }

        if site.has_else {
            let true_dag = make_branch_body_dag(
                ctx.module_name,
                &ctx.target.node_id,
                index,
                "true",
                &then_transports,
            );
            let false_dag = make_branch_body_dag(
                ctx.module_name,
                &ctx.target.node_id,
                index,
                "false",
                &else_transports,
            );
            let branch_node = BranchBuilder::new(node_id.clone())
                .with_true_branch(true_dag)
                .with_false_branch(false_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(branch_node.with_origin(pattern_expansion_origin(
                ctx.source_file,
                ctx.module_name,
                ctx.item_name,
                ctx.item_span,
                "if_branch",
            )));
        } else {
            let then_dag = make_branch_body_dag(
                ctx.module_name,
                &ctx.target.node_id,
                index,
                "then",
                &then_transports,
            );
            let if_node = IfBuilder::new(node_id.clone())
                .with_then(then_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(if_node.with_origin(pattern_expansion_origin(
                ctx.source_file,
                ctx.module_name,
                ctx.item_name,
                ctx.item_span,
                "if_then",
            )));
        }
        builder.add_edge(&node_id, "result", &ctx.target.node_id, PortName::DEPS);
    }

    let match_sites = detect_match_branches_in_stmts(ctx.stmts);
    for (index, site) in match_sites.iter().enumerate() {
        let node_id = format!("{}::cf_match_{index}", ctx.target.node_id);
        // Resolve match-arm service calls to transport entries.
        // NOTE: Currently all arms' transports go into both branches because
        // the match condition isn't wired to the BranchBuilder's condition port.
        // Both branches execute and the fn_body evaluation picks the correct arm.
        // Per-arm transport isolation still requires proper match condition routing.
        let mut match_transports = Vec::new();
        for call_path in &site.all_service_call_paths {
            if let Some(transport) = resolve_loop_body_service_call(
                call_path,
                ctx.uses_binding_types,
                ctx.service_registry,
            ) {
                match_transports.push(transport);
            }
        }
        if site.arm_count > 1 {
            let true_dag = make_branch_body_dag(
                ctx.module_name,
                &ctx.target.node_id,
                index,
                "match_true",
                &match_transports,
            );
            let false_dag = make_branch_body_dag(
                ctx.module_name,
                &ctx.target.node_id,
                index,
                "match_false",
                &match_transports,
            );
            let branch_node = BranchBuilder::new(node_id.clone())
                .with_true_branch(true_dag)
                .with_false_branch(false_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(branch_node.with_origin(pattern_expansion_origin(
                ctx.source_file,
                ctx.module_name,
                ctx.item_name,
                ctx.item_span,
                "match_branch",
            )));
        } else {
            let then_dag = make_branch_body_dag(
                ctx.module_name,
                &ctx.target.node_id,
                index,
                "match_then",
                &match_transports,
            );
            let if_node = IfBuilder::new(node_id.clone())
                .with_then(then_dag)
                .with_output("result", "Any")
                .build();
            builder.add_node(if_node.with_origin(pattern_expansion_origin(
                ctx.source_file,
                ctx.module_name,
                ctx.item_name,
                ctx.item_span,
                "match_then",
            )));
        }
        builder.add_edge(&node_id, "result", &ctx.target.node_id, PortName::DEPS);
    }
}

fn expand_content_upsert_patterns(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
) -> HashMap<String, PatternExpansionResult> {
    let mut bound_callables = HashMap::<String, String>::new();
    let mut expansion_count = 0usize;
    // Track expansion results: binding_name → ExpandedNodeOutput for "written".
    // content_upsert's semantic output "written" means "action ran", not
    // "file was already fresh". Synthesize a per-call output node so
    // `result.written` stays tied to the invocation instead of the shared
    // unsuffixed pattern declaration node.
    let mut expansion_outputs = HashMap::<String, ExpandedNodeOutput>::new();
    let mut expansion_results = HashMap::<String, PatternExpansionResult>::new();

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
                            ctx,
                            expansion_count,
                            args,
                            target,
                            &bound_callables,
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
                    let suffix = expansion_suffix(ctx.item_name, expansion_count);
                    let written_id = format!("content_upsert_written_{suffix}");
                    let written_output = ExpandedNodeOutput {
                        node_id: written_id.clone(),
                        output_port: "result".to_string(),
                    };
                    expansion_outputs.insert(binding.clone(), written_output.clone());
                    expansion_results.insert(
                        binding.clone(),
                        PatternExpansionResult {
                            return_outputs: [("written".to_string(), written_output)]
                                .into_iter()
                                .collect(),
                            last_node_id: written_id,
                        },
                    );
                    expand_single_content_upsert(
                        builder,
                        ctx,
                        expansion_count,
                        args,
                        target,
                        &bound_callables,
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
    // Wire expansion return outputs to the caller's __out: ports.
    wire_expansion_return_outputs(builder, stmts, target, &expansion_outputs);
    expansion_results
}

fn expand_single_content_upsert(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    expansion_count: usize,
    args: &[(Option<String>, Expr)],
    target: &LoweredEndpoint,
    bound_callables: &HashMap<String, String>,
) {
    let suffix = expansion_suffix(ctx.item_name, expansion_count);
    let prepare_read_id = format!("prepare_read_{suffix}");
    let execute_read_id = format!("execute_read_{suffix}");
    let compare_id = format!("compare_{suffix}_content");
    let written_id = format!("content_upsert_written_{suffix}");
    let prepare_write_id = format!("prepare_write_{suffix}");
    let execute_transport_id = format!("execute_{suffix}_transport");
    let prepare_read_inputs = vec![Port::scalar("path", "String")];
    let pattern_origin = pattern_expansion_origin_for_ctx(ctx, "content_upsert");
    builder.add_node(
        Node::opaque(
            prepare_read_id.clone(),
            prepare_read_inputs,
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("skip", "Bool"),
            ],
            LoweredOp::Primitive {
                module: ctx.module_name.to_string(),
                name: format!("content_upsert::{prepare_read_id}"),
                kind: PrimitiveOpKind::IoPrepareFileRead,
            },
        )
        .with_origin(pattern_origin.clone()),
    );
    builder.add_node(
        Node::opaque(
            execute_read_id.clone(),
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("skip", "Bool"),
            ],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: ctx.module_name.to_string(),
                name: format!("content_upsert::{execute_read_id}"),
                kind: PrimitiveOpKind::IoExecuteFileRead,
            },
        )
        .with_origin(pattern_origin.clone()),
    );
    builder.add_node(
        Node::opaque(
            compare_id.clone(),
            vec![
                Port::scalar("expected_content", "String"),
                Port::scalar("response", "TransportResponse"),
            ],
            vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
            LoweredOp::Primitive {
                module: ctx.module_name.to_string(),
                name: format!("content_upsert::{compare_id}"),
                kind: PrimitiveOpKind::CompareEquality,
            },
        )
        .with_origin(pattern_origin.clone()),
    );
    builder.add_node(
        Node::opaque(
            written_id.clone(),
            vec![Port::scalar("operand", "Any")],
            vec![Port::scalar("result", "Bool")],
            LoweredOp::Primitive {
                module: ctx.module_name.to_string(),
                name: format!("content_upsert::{written_id}"),
                kind: PrimitiveOpKind::UnaryOp {
                    op: expr::LoweredUnaryOp::Not,
                },
            },
        )
        .with_origin(pattern_origin.clone()),
    );
    builder.add_node(
        Node::opaque(
            prepare_write_id.clone(),
            vec![
                Port::scalar("content", "String"),
                Port::scalar("path", "String"),
            ],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: ctx.module_name.to_string(),
                name: format!("content_upsert::{prepare_write_id}"),
                kind: PrimitiveOpKind::IoPrepareFileWrite,
            },
        )
        .with_origin(pattern_origin.clone()),
    );
    let execute_transport_inputs = vec![
        Port::scalar("request", "TransportRequest"),
        Port::scalar("skip", "Bool"),
    ];
    builder.add_node(
        Node::opaque(
            execute_transport_id.clone(),
            execute_transport_inputs,
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: ctx.module_name.to_string(),
                name: format!("content_upsert::{execute_transport_id}"),
                kind: PrimitiveOpKind::IoExecuteFileWrite,
            },
        )
        .with_origin(pattern_origin),
    );

    builder.add_edge(&prepare_read_id, "request", &execute_read_id, "request");
    builder.add_edge(&prepare_read_id, "skip", &execute_read_id, "skip");
    builder.add_edge(&execute_read_id, "response", &compare_id, "response");
    builder.add_edge(&compare_id, "fresh", &written_id, "operand");
    builder.add_edge(
        &prepare_write_id,
        "request",
        &execute_transport_id,
        "request",
    );
    builder.add_edge(&compare_id, "skip", &execute_transport_id, "skip");
    builder.add_edge(
        &execute_transport_id,
        "response",
        &target.node_id,
        PortName::DEPS,
    );

    let content_destinations = [
        (compare_id.as_str(), "expected_content"),
        (prepare_write_id.as_str(), "content"),
    ];
    let wired_content = wire_resolved_or_param_source(
        builder,
        ctx.module_name,
        ctx.item_name,
        ctx.param_types,
        resolve_content_source(args, bound_callables, ctx),
        resolve_named_ident_arg(args, "content"),
        &content_destinations,
    );
    // Fallback: if the content arg is a data declaration ident, create a
    // literal source node with the data value — mirrors wire_fn_call_arguments.
    if !wired_content {
        if let Some(ident) = resolve_named_ident_arg(args, "content") {
            if let Some(json_val) = ctx.data_values.get(ident) {
                let literal = ServiceCallArgLiteral::Json(json_val.clone());
                let src = ensure_literal_source_node(
                    builder,
                    ctx.module_name,
                    ctx.item_name,
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
        ctx.module_name,
        ctx.item_name,
        ctx.param_types,
        resolve_path_source(args, bound_callables, ctx),
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
                ctx.module_name,
                ctx.item_name,
                "path",
                "String",
                &literal,
                format!("content_upsert_path_{suffix}").as_str(),
            );
            if let ServiceCallArgLiteral::String(path_str) = &literal {
                let path_annotation_id = format!("output_path_annotation_{suffix}");
                builder.add_node(Node::opaque(
                    path_annotation_id.clone(),
                    vec![],
                    vec![Port::scalar("path", "String")],
                    LoweredOp::Primitive {
                        module: ctx.module_name.to_string(),
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
    ctx: &LoweringContext<'_>,
) -> Option<LoweredEndpoint> {
    let (_, content_expr) = args
        .iter()
        .find(|(name, _)| matches!(name.as_deref(), Some("content")))?;
    resolve_source_expr(content_expr, bound_callables, ctx)
}

fn resolve_path_source(
    args: &[(Option<String>, Expr)],
    bound_callables: &HashMap<String, String>,
    ctx: &LoweringContext<'_>,
) -> Option<LoweredEndpoint> {
    let (_, path_expr) = args
        .iter()
        .find(|(name, _)| matches!(name.as_deref(), Some("path")))?;
    resolve_source_expr(path_expr, bound_callables, ctx)
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
    ctx: &LoweringContext<'_>,
) -> Option<LoweredEndpoint> {
    match expr {
        Expr::FieldAccess(base, field) => {
            let base_endpoint = resolve_source_expr(base, bound_callables, ctx)?;
            Some(LoweredEndpoint {
                node_id: base_endpoint.node_id,
                primary_output: field.clone(),
            })
        }
        _ => {
            let source_name = match expr {
                Expr::Ident(name) => {
                    if let Some(source) = ctx.bound_callable_sources.get(name) {
                        return Some(source.clone());
                    }
                    if let Some(source) = ctx.bound_service_sources.get(name) {
                        return Some(source.parse.clone());
                    }
                    bound_callables
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone())
                }
                Expr::Call(name, _) => name.clone(),
                _ => return None,
            };
            ctx.endpoints_by_name
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
    params: &'a [daglang_syntax::ast::Param],
    type_params: &'a [String],
    body_stmts: &'a [Stmt],
    uses: &'a [daglang_syntax::ast::UsesClause],
}

/// Collect ALL pattern definitions from the project (generic and non-generic).
fn collect_expandable_pattern_defs<'a>(
    project: &'a TypedProject<'_>,
) -> HashMap<String, ExpandablePattern<'a>> {
    let mut patterns = HashMap::new();
    for module in &project.graph().modules {
        for item in &module.ast.items {
            if let Item::PatternDef(def) = &item.node {
                patterns.insert(
                    def.name.clone(),
                    ExpandablePattern {
                        params: &def.params,
                        type_params: &def.type_params,
                        body_stmts: &def.body.stmts,
                        uses: &def.uses,
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
    ctx: &LoweringContext<'_>,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
) {
    let pattern_defs = collect_expandable_pattern_defs(project);
    let mut expansion_count = 0usize;
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

        if call_name == "content_upsert" {
            continue;
        }

        let Some(pattern_def) = pattern_defs.get(call_name) else {
            continue;
        };

        expansion_count += 1;
        let inner_ctx = LoweringContext {
            source_file: ctx.source_file,
            expanded_results: &expanded_results,
            module_name: ctx.module_name,
            item_name: ctx.item_name,
            item_span: ctx.item_span,
            param_types: ctx.param_types,
            endpoints_by_name: ctx.endpoints_by_name,
            data_values: ctx.data_values,
            service_registry: ctx.service_registry,
            bound_callable_sources: ctx.bound_callable_sources,
            bound_service_sources: ctx.bound_service_sources,
            local_let_bindings: ctx.local_let_bindings,
            body_stmts: ctx.body_stmts,
            all_fn_bodies: ctx.all_fn_bodies,
            variant_names: ctx.variant_names,
            callable_param_defaults: ctx.callable_param_defaults,
            endpoints_by_full: ctx.endpoints_by_full,
            uses_binding_types: ctx.uses_binding_types,
        };
        let pexp = PatternExpansionParams {
            target,
            all_patterns: &pattern_defs,
            depth: 0,
        };
        let result = expand_single_pattern(
            builder,
            &inner_ctx,
            expansion_count,
            pattern_def,
            call_args,
            &pexp,
        );
        if let Some(result) = result {
            builder.add_edge(
                &result.last_node_id,
                "fresh",
                &target.node_id,
                PortName::DEPS,
            );
            expanded_results.insert(binding_name.to_string(), result);
        }
    }
    // Wire expansion return outputs to the caller's __out: ports.
    // Flatten the PatternExpansionResult return_outputs into a single
    // mapping: binding.field → ExpandedNodeOutput.
    let mut flat_outputs = HashMap::<String, ExpandedNodeOutput>::new();
    for (binding, result) in &expanded_results {
        for (field, output) in &result.return_outputs {
            flat_outputs.insert(format!("{binding}.{field}"), output.clone());
        }
    }
    wire_expansion_return_outputs(builder, stmts, target, &flat_outputs);
}

/// Wire pattern expansion return outputs to the caller's `__out:` ports.
///
/// Scans the caller's `return { field: binding.sub }` statement and wires
/// matching expansion outputs to `target.__out:field`. This bridges the gap
/// between inline pattern expansion (which creates real nodes) and the
/// callable passthrough protocol (which requires `__out:` edges).
///
/// `expansion_outputs` maps lookup keys to expanded node outputs:
/// - For `content_upsert`: key = `binding_name`, output = synthetic per-call
///   `written` node (`!fresh`) so `binding.written` preserves callsite identity.
/// - For non-generic patterns: key = `binding.field`, output = expansion return output.
///   Return `binding.field` → direct lookup.
fn wire_expansion_return_outputs(
    builder: &mut DagBuilder,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
    expansion_outputs: &HashMap<String, ExpandedNodeOutput>,
) {
    if expansion_outputs.is_empty() {
        return;
    }
    for stmt in stmts {
        if let Stmt::Return(bindings) = stmt {
            for (field_name, expr) in bindings {
                let passthrough_port = output_passthrough_input_name(field_name.as_str());
                if builder.has_edge_to_port(&target.node_id, &passthrough_port) {
                    continue;
                }
                // Match `binding.sub` field access on expansion results.
                if let Expr::FieldAccess(base, sub_field) = expr {
                    if let Expr::Ident(binding) = base.as_ref() {
                        // Try content_upsert style: key = binding_name.
                        // content_upsert returns { written: Bool }, so bind the
                        // field access to the synthesized per-call output node.
                        if let Some(output) = expansion_outputs.get(binding.as_str()) {
                            builder.add_edge(
                                &output.node_id,
                                &output.output_port,
                                &target.node_id,
                                &passthrough_port,
                            );
                            continue;
                        }
                        // Try non-generic pattern style: key = "binding.field".
                        let composite_key = format!("{binding}.{sub_field}");
                        if let Some(output) = expansion_outputs.get(&composite_key) {
                            builder.add_edge(
                                &output.node_id,
                                &output.output_port,
                                &target.node_id,
                                &passthrough_port,
                            );
                        }
                    }
                }
            }
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
    ctx: &LoweringContext<'_>,
    expansion_count: usize,
    pattern: &ExpandablePattern<'_>,
    call_args: &[(Option<String>, Expr)],
    pexp: &PatternExpansionParams<'_>,
) -> Option<PatternExpansionResult> {
    if pexp.depth >= PATTERN_EXPANSION_MAX_DEPTH {
        return None;
    }

    let suffix = expansion_suffix(ctx.item_name, expansion_count);
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
    let mut type_param_map = HashMap::<String, &Expr>::new();
    for type_param in pattern.type_params {
        let lowered = type_param.to_ascii_lowercase();
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
    let caller_uses_binding_types: HashMap<String, String> = call_args
        .iter()
        .filter_map(|(_, expr)| {
            if let Expr::ServiceCall(path, _) = expr {
                let binding = path.first()?;
                None::<(String, String)>.or_else(|| Some((binding.clone(), binding.clone())))
            } else {
                None
            }
        })
        .collect();
    let mut combined_uses = uses_binding_types.clone();
    combined_uses.extend(caller_uses_binding_types);

    // Track nodes created in the expansion for after-edge wiring.
    let mut node_outputs = HashMap::<String, ExpandedNodeOutput>::new();
    let mut last_node_id = String::new();

    for body_stmt in pattern.body_stmts {
        match body_stmt {
            Stmt::Node(ns) => {
                let effective_expr = substitute_type_param(&ns.expr, &type_param_map);
                let expr_ref = effective_expr.as_ref().unwrap_or(&ns.expr);

                let env = PatternNodeEnv {
                    suffix: &suffix,
                    arg_map: &arg_map,
                    uses_binding_types: &combined_uses,
                    node_outputs: &node_outputs,
                };
                let expanded = expand_pattern_body_node(
                    builder, ctx, &env, pexp, &ns.name, expr_ref, &ns.after,
                );
                if let Some(output) = expanded {
                    last_node_id.clone_from(&output.node_id);
                    node_outputs.insert(ns.name.clone(), output);
                }
            }
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                let effective_expr = substitute_type_param(expr, &type_param_map);
                let expr_ref = effective_expr.as_ref().unwrap_or(expr);

                let env = PatternNodeEnv {
                    suffix: &suffix,
                    arg_map: &arg_map,
                    uses_binding_types: &combined_uses,
                    node_outputs: &node_outputs,
                };
                let expanded =
                    expand_pattern_body_node(builder, ctx, &env, pexp, name, expr_ref, &[]);
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
    ctx: &LoweringContext<'_>,
    env: &PatternNodeEnv<'_>,
    pexp: &PatternExpansionParams<'_>,
    node_name: &str,
    expr: &Expr,
    after_deps: &[String],
) -> Option<ExpandedNodeOutput> {
    match expr {
        Expr::ServiceCall(path, args) => {
            expand_service_call_node(builder, ctx, env, node_name, path, args, after_deps)
        }
        Expr::Call(call_name, args) if call_name == "eq" => {
            expand_eq_node(builder, ctx, env, node_name, args, after_deps)
        }
        Expr::Call(call_name, call_args) if pexp.all_patterns.contains_key(call_name.as_str()) => {
            // Recursive pattern expansion: this node calls another pattern.
            let inner_pattern = &pexp.all_patterns[call_name.as_str()];

            // Merge caller's arg_map into the call args so the inner pattern
            // can resolve references to the outer pattern's parameters.
            let mut merged_args: Vec<(Option<String>, Expr)> = Vec::new();
            for (inner_arg_name, inner_arg_expr) in call_args {
                let resolved_expr = resolve_expr_idents(inner_arg_expr, env.arg_map);
                merged_args.push((inner_arg_name.clone(), resolved_expr));
            }

            let inner_pexp = PatternExpansionParams {
                target: pexp.target,
                all_patterns: pexp.all_patterns,
                depth: pexp.depth + 1,
            };
            let inner_result = expand_single_pattern(
                builder,
                ctx,
                pexp.depth + 1,
                inner_pattern,
                &merged_args,
                &inner_pexp,
            );

            // Wire after-dependency edges.
            if let Some(ref result) = inner_result {
                for dep in after_deps {
                    if let Some(dep_output) = env.node_outputs.get(dep) {
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
    ctx: &LoweringContext<'_>,
    env: &PatternNodeEnv<'_>,
    node_name: &str,
    call_path: &[String],
    call_args: &[(Option<String>, Expr)],
    after_deps: &[String],
) -> Option<ExpandedNodeOutput> {
    // Resolve the service call path to a registry key.
    // e.g., ["fs", "read"] with uses fs: Filesystem → "Filesystem.read"
    let binding = call_path.first()?;
    let capability = call_path.last()?;
    let resource_type = env.uses_binding_types.get(binding)?;
    let cap_key = format!("{resource_type}.{capability}");

    let endpoint = ctx.service_registry.get(&cap_key)?;

    // Clone the triplet with a unique suffix.
    let clone_suffix = format!("{}_{node_name}", env.suffix);
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
            ctx,
            arg_name,
            arg_expr,
            &cloned.prepare_node_id,
            env.arg_map,
            env.node_outputs,
        );
    }

    // Wire after-dependency edges.
    for dep in after_deps {
        if let Some(dep_output) = env.node_outputs.get(dep) {
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
    ctx: &LoweringContext<'_>,
    env: &PatternNodeEnv<'_>,
    node_name: &str,
    args: &[(Option<String>, Expr)],
    after_deps: &[String],
) -> Option<ExpandedNodeOutput> {
    let compare_id = format!("compare_{}_{node_name}", env.suffix);

    builder.add_node(Node::opaque(
        compare_id.clone(),
        vec![
            Port::scalar("expected_content", "String"),
            Port::scalar("actual_content", "String"),
        ],
        vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
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
            ctx,
            arg_expr,
            &compare_id,
            dest_port,
            env.arg_map,
            env.node_outputs,
        );
    }

    // Wire after-dependency edges.
    for dep in after_deps {
        if let Some(dep_output) = env.node_outputs.get(dep) {
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
    ctx: &LoweringContext<'_>,
    arg_name: &str,
    arg_expr: &Expr,
    prepare_node_id: &str,
    arg_map: &HashMap<String, &Expr>,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
) {
    wire_pattern_arg_to_node(
        builder,
        ctx,
        arg_expr,
        prepare_node_id,
        arg_name,
        arg_map,
        node_outputs,
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
    ctx: &LoweringContext<'_>,
    arg_expr: &Expr,
    dest_node_id: &str,
    dest_port: &str,
    arg_map: &HashMap<String, &Expr>,
    node_outputs: &HashMap<String, ExpandedNodeOutput>,
) {
    match arg_expr {
        Expr::Ident(name) => {
            if let Some(caller_expr) = arg_map.get(name.as_str()) {
                // Case 1: Pattern parameter → resolve through caller's arg map.
                wire_caller_expr_to_node(builder, ctx, caller_expr, dest_node_id, dest_port);
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
                            ctx,
                            caller_expr,
                            dest_node_id,
                            dest_port,
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
                ctx.module_name,
                ctx.item_name,
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
    ctx: &LoweringContext<'_>,
    caller_expr: &Expr,
    dest_node_id: &str,
    dest_port: &str,
) {
    match caller_expr {
        Expr::Ident(name) => {
            if let Some(param_ty) = ctx.param_types.get(name.as_str()) {
                let param_source = ensure_param_source_node(
                    builder,
                    ctx.module_name,
                    ctx.item_name,
                    name,
                    param_ty.as_str(),
                );
                builder.add_edge(&param_source, name, dest_node_id, dest_port);
            } else if let Some(source) = ctx.bound_callable_sources.get(name.as_str()) {
                builder.add_edge(
                    &source.node_id,
                    &source.primary_output,
                    dest_node_id,
                    dest_port,
                );
            } else if let Some(source) = ctx.bound_service_sources.get(name.as_str()) {
                builder.add_edge(
                    &source.parse.node_id,
                    &source.parse.primary_output,
                    dest_node_id,
                    dest_port,
                );
            } else if let Some(Some(endpoint)) = ctx.endpoints_by_name.get(name.as_str()) {
                builder.add_edge(
                    &endpoint.node_id,
                    &endpoint.primary_output,
                    dest_node_id,
                    dest_port,
                );
            } else if let Some(json_val) = ctx.data_values.get(name.as_str()) {
                let literal = ServiceCallArgLiteral::Json(json_val.clone());
                let src = ensure_literal_source_node(
                    builder,
                    ctx.module_name,
                    ctx.item_name,
                    dest_port,
                    "String",
                    &literal,
                    &format!("pattern_data_{name}"),
                );
                builder.add_edge(&src, dest_port, dest_node_id, dest_port);
            } else if let Some(result) = ctx.expanded_results.get(name.as_str()) {
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
                if let Some(source) = ctx.bound_callable_sources.get(base_name.as_str()) {
                    builder.add_edge(&source.node_id, field, dest_node_id, dest_port);
                } else if let Some(source) = ctx.bound_service_sources.get(base_name.as_str()) {
                    builder.add_edge(&source.parse.node_id, field, dest_node_id, dest_port);
                } else if let Some(Some(endpoint)) = ctx.endpoints_by_name.get(base_name.as_str()) {
                    builder.add_edge(&endpoint.node_id, field, dest_node_id, dest_port);
                } else if let Some(result) = ctx.expanded_results.get(base_name.as_str()) {
                    if let Some(output) = result.return_outputs.get(field.as_str()) {
                        builder.add_edge(
                            &output.node_id,
                            &output.output_port,
                            dest_node_id,
                            dest_port,
                        );
                    }
                } else if let Some(param_ty) = ctx.param_types.get(base_name.as_str()) {
                    let param_source = ensure_param_source_node(
                        builder,
                        ctx.module_name,
                        ctx.item_name,
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
                ctx.module_name,
                ctx.item_name,
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

// NOTE: add_makegen_scaffolding was removed — it looked for a renamed function
// ("render_makefile" → "render_makefile_content") and never ran. Resource ports
// on content_upsert nodes are now handled generically by the resolver
// (needs_transport_resource in core/resolve).

fn derive_service_call_metadata(
    service: &ServiceDef,
    operation: &OperationDef,
    data_registry: &DataRegistry<'_>,
) -> Result<ServiceCallMetadata, LowerError> {
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

    let spec = derive_operation_spec(service, operation, transport, data_registry)?;

    // Auto-derive readonly from HTTP method: GET and HEAD are read-only by definition.
    let readonly = operation.readonly
        || matches!(
            &operation.transport,
            Some(TransportBinding::Rest { method, .. })
                if method.eq_ignore_ascii_case("GET") || method.eq_ignore_ascii_case("HEAD")
        );

    // S45: Explicit annotation is authoritative; unknown values fail fast.
    // Falls back to service-name inference only when no annotation is present.
    let response_provider =
        match service.config.response_provider.as_deref() {
            Some(name) => Some(name.parse::<ResponseProvider>().map_err(|e| {
                LowerError::InvalidAnnotation {
                    service: service.name.clone(),
                    annotation: "response_provider".to_string(),
                    detail: e,
                }
            })?),
            None => infer_response_provider(&service.name),
        };

    Ok(ServiceCallMetadata {
        service: service.name.clone(),
        operation: operation.name.clone(),
        transport,
        idempotent: operation.idempotent,
        readonly,
        spec,
        response_provider,
    })
}

// ============================================================================
// Data registry: compile-time resolution of `data` item values
// ============================================================================

/// Registry of module-level `data` definitions, keyed by both qualified and
/// unqualified names. Used to resolve compile-time constants (e.g., env maps).
type DataRegistry<'a> = HashMap<String, &'a DataDef>;

/// Build a data registry from all modules in the project.
fn build_data_registry<'a>(project: &'a TypedProject<'_>) -> DataRegistry<'a> {
    let mut registry = DataRegistry::new();
    for module in &project.graph().modules {
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
        if field.name == "env" && is_map_string_string(&field.ty) {
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
    field.name == "env" && is_map_string_string(&field.ty)
}

fn is_noncanonical_dot_output_path(path: &str) -> bool {
    if path == "." || path.contains('/') || !path.contains('.') {
        return false;
    }
    if path.chars().any(|ch| {
        ch.is_whitespace()
            || matches!(
                ch,
                '=' | '!'
                    | '<'
                    | '>'
                    | '('
                    | ')'
                    | '"'
                    | '\''
                    | '?'
                    | ':'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '+'
                    | '*'
                    | '%'
                    | '&'
                    | '|'
            )
    }) {
        return false;
    }
    path.split('.').all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    })
}

fn validate_rest_output_field_paths(
    service: &ServiceDef,
    operation: &OperationDef,
) -> Result<(), LowerError> {
    let Some(TransportBinding::Rest { .. }) = &operation.transport else {
        return Ok(());
    };
    for field in &operation.outputs {
        let Some(path) = field.from_path.as_deref() else {
            continue;
        };
        if is_noncanonical_dot_output_path(path) {
            return Err(LowerError::InvalidTransportSpec {
                service: service.name.clone(),
                operation: operation.name.clone(),
                detail: format!(
                    "output field `{}` uses non-canonical dotted path `{}`; use slash-delimited JSON path `{}`",
                    field.name,
                    path,
                    path.replace('.', "/")
                ),
            });
        }
    }
    Ok(())
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
) -> Result<Option<ServiceOperationSpec>, LowerError> {
    match transport {
        ServiceTransportClass::RestNetwork => {
            Ok(derive_rest_spec(service, operation)
                .map(|s| ServiceOperationSpec::Rest(Box::new(s))))
        }
        ServiceTransportClass::ShellLocal => {
            Ok(derive_shell_spec(service, operation, data_registry)?
                .map(ServiceOperationSpec::Shell))
        }
        ServiceTransportClass::FileBoundary => Ok(Some(ServiceOperationSpec::File(
            derive_file_spec(operation)?,
        ))),
        ServiceTransportClass::LocalDirect => Ok(Some(ServiceOperationSpec::Local(
            derive_local_spec(operation),
        ))),
        ServiceTransportClass::InterfaceStub => {
            // Services implementing interfaces with no transport block.
            // Use the service name as the interface name (from `: InterfaceName` syntax).
            Ok(Some(ServiceOperationSpec::InterfaceStub {
                interface: service
                    .implements
                    .clone()
                    .unwrap_or_else(|| service.name.clone()),
                capability: operation.name.clone(),
            }))
        }
        ServiceTransportClass::Unknown => {
            // Transport class could not be determined. This is not an error
            // for abstract services (no transport block) — they get spec: None
            // which the resolver handles as a stub. Concrete services with
            // declared but unrecognized transport would hit MissingTransport
            // earlier in the validation pass.
            Ok(None)
        }
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

/// Derive transport middleware config from service config blocks (TL-12).
fn derive_middleware_config(
    service: &ServiceDef,
    _operation: &OperationDef,
) -> Option<TransportMiddlewareConfig> {
    let config = &service.config;

    // Only produce middleware config if at least one block is defined.
    if config.rate_limits.is_empty() && config.retry.is_none() {
        return None;
    }

    let rate_limit = config.rate_limits.first().map(|rl| {
        // Store raw requests and window — let the runtime do precise math.
        let (requests, window_seconds) = match rl.per {
            RateLimitUnit::Second => (rl.requests as u32, 1_u32),
            RateLimitUnit::Minute => (rl.requests as u32, 60),
            RateLimitUnit::Hour => (rl.requests as u32, 3600),
            RateLimitUnit::Day => (rl.requests as u32, 86400),
        };
        // Burst = 1/10th of requests or 1, whichever is larger.
        let max_burst = (requests / 10).max(1);

        RateLimitConfig {
            scope_key: rl
                .scope
                .clone()
                .unwrap_or_else(|| service.name.replace('.', ":")),
            algorithm: RateLimitAlgorithm::TokenBucket,
            max_burst,
            requests,
            window_seconds,
            honor_retry_after: true,
        }
    });

    let retry = config.retry.as_ref().map(|r| {
        let backoff = match r.backoff {
            BackoffStrategy::Constant => RetryBackoff::Fixed,
            BackoffStrategy::Linear => RetryBackoff::Fixed,
            BackoffStrategy::Exponential => RetryBackoff::Exponential,
        };
        RetryConfig {
            max_attempts: r.max_attempts as u32,
            base_delay_ms: r.base_delay_ms.unwrap_or(100) as u64,
            max_delay_ms: r.max_delay_ms.unwrap_or(10_000) as u64,
            backoff,
            retry_statuses: r.retry_on.iter().map(|s| *s as u16).collect(),
            retry_network_errors: true,
            require_idempotent_or_readonly: false,
            circuit_breaker: None,
        }
    });

    // Derive error shape extraction from error_shape {} blocks (TL-16).
    // When explicit error_shape is declared, the transport layer uses JSON-path
    // extraction instead of hardcoded provider parsing.
    let error_shape = service.config.error_shapes.first().map(|es| {
        gunbc_ir::transport::middleware::ErrorShapeExtraction {
            message_path: es
                .message_path
                .clone()
                .unwrap_or_else(|| ".message".to_string()),
            code_path: es.error_type_path.clone(),
            details_path: None,
        }
    });

    // TL-15: parse_provider_error_shapes is always false — the transport layer
    // uses only error_shape JSON-path extraction.
    let response_classification =
        infer_response_provider(&service.name).map(|provider| ResponseClassification {
            provider,
            prioritize_auth_errors: true,
            parse_provider_error_shapes: false,
            error_shape: error_shape.clone(),
            output_shape: None, // Per-operation output shapes are on RestOperationSpec.
        });

    Some(TransportMiddlewareConfig {
        rate_limit,
        retry,
        credential: None, // Credential config is wired separately via auth_scheme.
        response_classification,
    })
}

// S45: `parse_response_provider` removed — use `ResponseProvider::from_str`
// (in ir/src/transport/middleware.rs) which is the single authority and returns
// `Err` on unknown values instead of silently falling back to inference.

/// Infer the response provider from service name patterns.
fn infer_response_provider(service_name: &str) -> Option<ResponseProvider> {
    let lower = service_name.to_lowercase();
    if lower.starts_with("github.") || lower.contains("gist") {
        Some(ResponseProvider::GitHub)
    } else if lower.starts_with("gcp.") || lower.starts_with("google.") {
        Some(ResponseProvider::Gcp)
    } else if lower.contains("anthropic") || lower.starts_with("llm.anthropic") {
        Some(ResponseProvider::Anthropic)
    } else if lower.contains("openai") || lower.starts_with("llm.openai") {
        Some(ResponseProvider::OpenAi)
    } else {
        None
    }
}

/// Convert AST status pattern to spec status pattern.
fn convert_status_pattern(status: &daglang_syntax::ast::StatusPattern) -> ResponseStatusPattern {
    match status {
        daglang_syntax::ast::StatusPattern::Exact(code) => ResponseStatusPattern::Exact(*code),
        daglang_syntax::ast::StatusPattern::Success2xx => ResponseStatusPattern::Success2xx,
        daglang_syntax::ast::StatusPattern::Redirect3xx => ResponseStatusPattern::Redirect3xx,
        daglang_syntax::ast::StatusPattern::ClientError4xx => ResponseStatusPattern::ClientError4xx,
        daglang_syntax::ast::StatusPattern::ServerError5xx => ResponseStatusPattern::ServerError5xx,
    }
}

/// Convert AST response entries to spec response mapping entries.
fn derive_response_mapping(
    response_entries: &[daglang_syntax::ast::ResponseEntry],
) -> Vec<ResponseMappingEntry> {
    response_entries
        .iter()
        .map(|entry| ResponseMappingEntry {
            status: convert_status_pattern(&entry.status),
            response_type: type_expr_to_string(&entry.response_type),
            description: entry.description.clone(),
        })
        .collect()
}

/// Convert AST exit code to spec exit code pattern.
fn convert_exit_code(code: &daglang_syntax::ast::ExitCode) -> ExitCodePattern {
    match code {
        daglang_syntax::ast::ExitCode::Exact(n) => ExitCodePattern::Exact(*n),
        daglang_syntax::ast::ExitCode::NonZero => ExitCodePattern::NonZero,
    }
}

/// Convert AST exit entries to spec exit mapping entries.
fn derive_exit_mapping(exit_entries: &[daglang_syntax::ast::ExitEntry]) -> Vec<ExitMappingEntry> {
    exit_entries
        .iter()
        .map(|entry| ExitMappingEntry {
            code: convert_exit_code(&entry.code),
            output_type: type_expr_to_string(&entry.output_type),
            description: entry.description.clone(),
        })
        .collect()
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
    let input_fields = derive_input_fields(
        &operation.inputs,
        &path_template,
        &headers,
        auth_input.as_deref(),
    );
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

    // Derive middleware config from rate_limit/retry blocks (TL-12).
    let middleware = derive_middleware_config(service, operation);

    // Derive response mapping from response {} blocks (SL-9).
    let response_mapping = derive_response_mapping(&operation.response);

    // RT-1: Derive mock response entries from mock_response {} blocks.
    let empty_variants = HashSet::new();
    let mock_responses: Vec<spec::MockResponseEntry> = operation
        .mock_responses
        .iter()
        .filter_map(|entry| {
            let body = expr_to_json_literal(&entry.body, &empty_variants)?;
            Some(spec::MockResponseEntry {
                status: entry.status,
                body_json: serde_json::to_string(&body).unwrap_or_default(),
                description: entry.description.clone(),
            })
        })
        .collect();

    // C29: Derive output shape extraction from output fields.
    let output_shape = if output_fields.is_empty() {
        None
    } else {
        Some(gunbc_ir::transport::middleware::OutputShapeExtraction {
            fields: output_fields
                .iter()
                .map(|f| gunbc_ir::transport::middleware::OutputFieldExtraction {
                    name: f.name.clone(),
                    type_id: f.type_id.clone(),
                    json_path: f.json_path.clone(),
                    is_secret: f.is_secret,
                    is_raw_body: f.is_raw_body,
                    is_optional: f.is_optional,
                })
                .collect(),
        })
    };

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
        middleware,
        response_mapping,
        output_shape,
        mock_responses,
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
) -> Result<Option<ShellOperationSpec>, LowerError> {
    let argv_template = match &operation.transport {
        Some(TransportBinding::Shell { argv }) => resolve_argv_exprs(argv),
        _ => return Ok(None),
    };

    let input_fields = derive_input_fields_for_shell(&operation.inputs, &argv_template);
    let output_fields = derive_output_fields(&operation.outputs);
    // S44: Explicit annotation is authoritative; unknown values fail fast.
    // Falls back to inference only when no annotation is present.
    let output_parsing = match operation.output_parsing.as_deref() {
        Some(name) => {
            name.parse::<ShellOutputParsing>()
                .map_err(|e| LowerError::InvalidAnnotation {
                    service: operation.name.clone(),
                    annotation: "output_parsing".to_string(),
                    detail: e,
                })?
        }
        None => infer_shell_output_parsing(&operation.outputs),
    };

    // Extract env from `env: Map<String, String>` input default.
    let env = extract_env_from_inputs(&operation.inputs, data_registry);

    // Derive exit mapping from exit {} blocks (SL-9).
    let exit_mapping = derive_exit_mapping(&operation.exit);

    Ok(Some(ShellOperationSpec {
        argv_template,
        input_fields,
        output_fields,
        output_parsing,
        env,
        exit_mapping,
    }))
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
                type_id,
                default: field.default.as_ref().map(expr_to_default_string),
                is_secret: is_secret_type(&field.ty),
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
                type_id,
                default: field.default.as_ref().map(expr_to_default_string),
                is_secret: is_secret_type(&field.ty),
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
                type_id,
                default: field.default.as_ref().map(expr_to_default_string),
                is_secret: is_secret_type(&field.ty),
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
                type_id: base_type_id,
                json_path,
                is_secret: is_secret_type(&field.ty),
                is_raw_body,
                is_optional: is_type_expr_optional(&field.ty),
            }
        })
        .collect()
}

// S44: `parse_shell_output_parsing` removed — use `ShellOutputParsing::from_str`
// (in spec.rs) which is the single authority and returns `Err` on unknown values
// instead of silently falling back to inference.

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
    if outputs.len() == 1 && is_bool_type(&outputs[0].ty) {
        return ShellOutputParsing::ExitCodeBool;
    }

    // Check if any output is a List type → SplitLines
    if outputs.iter().any(|field| is_list_type(&field.ty)) {
        return ShellOutputParsing::SplitLines;
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
    for module in project.modules() {
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
    // Carry (name, type, is_optional) so we can set optional cardinality for
    // both defaulted and optional (`T?`) inputs.
    let declared_inputs: Vec<(String, String, bool)> = match metadata.spec.as_ref() {
        Some(spec) if !spec.input_fields().is_empty() => spec
            .input_fields()
            .iter()
            .map(|field| {
                (
                    field.name.clone(),
                    field.type_id.clone(),
                    field.default.is_some() || field.type_id.ends_with('?'),
                )
            })
            .collect(),
        _ => operation
            .inputs
            .iter()
            .map(|field| {
                let ty = type_expr_to_string(&field.ty);
                (
                    field.name.clone(),
                    ty,
                    field.default.is_some() || is_type_expr_optional(&field.ty),
                )
            })
            .collect(),
    };
    declared_inputs
        .into_iter()
        .map(|(name, ty, is_optional)| {
            if is_optional {
                Port::optional(name.as_str(), ty.as_str())
            } else {
                Port::scalar(name.as_str(), ty.as_str())
            }
        })
        .collect()
}

fn capability_prepare_ports(
    capability: &CapabilityDef,
    metadata: &ServiceCallMetadata,
) -> Vec<Port> {
    // When a spec with explicit input fields is available (e.g., File operations),
    // use the spec's field declarations. Otherwise fall back to the capability's
    // declared inputs from the interface definition.
    let declared_inputs: Vec<(String, String, bool)> = match metadata.spec.as_ref() {
        Some(spec) if !spec.input_fields().is_empty() => spec
            .input_fields()
            .iter()
            .map(|field| {
                (
                    field.name.clone(),
                    field.type_id.clone(),
                    field.default.is_some() || field.type_id.ends_with('?'),
                )
            })
            .collect(),
        _ => capability
            .inputs
            .iter()
            .map(|field| {
                let ty = type_expr_to_string(&field.ty);
                (
                    field.name.clone(),
                    ty,
                    field.default.is_some() || is_type_expr_optional(&field.ty),
                )
            })
            .collect(),
    };
    declared_inputs
        .into_iter()
        .map(|(name, ty, is_optional)| {
            if is_optional {
                Port::optional(name.as_str(), ty.as_str())
            } else {
                Port::scalar(name.as_str(), ty.as_str())
            }
        })
        .collect()
}

/// SR-8: Validate that provider-specific config fields (those stored in
/// `config.extra`) are recognised for the service's provider prefix.
/// Returns an error for any unrecognised field so that config typos become
/// compile-time errors.
///
/// Validate provider-specific config fields against schemas derived from the DSL.
///
/// Single source of truth: `dsl/std/provider_config.dag` declares
/// `provider_config_schemas` (prefix → schema type mapping) and the schema
/// types themselves (with field declarations). This function reads both
/// to build the validation map dynamically — no hardcoded schema list.
fn validate_provider_config_fields(
    service: &ServiceDef,
    project: &TypedProject,
) -> Result<(), LowerError> {
    if service.config.extra.is_empty() {
        return Ok(());
    }

    // Derive schemas from DSL: read provider_config_schemas data + type defs.
    let schemas = derive_provider_schemas_from_project(project)?;

    // Open-world: when compiled in isolation (e.g., testgen), std/provider_config.dag
    // may not be in the dependency closure. Skip validation rather than rejecting all fields.
    if schemas.is_empty() {
        return Ok(());
    }

    // Find the matching schema for this service's name.
    let matched_schema = schemas
        .iter()
        .filter(|(prefix, _)| service.name.starts_with(prefix.as_str()) || service.name == *prefix)
        .max_by_key(|(prefix, _)| prefix.len());

    match matched_schema {
        Some((_prefix, allowed_fields)) => {
            for field in &service.config.extra {
                if !allowed_fields.contains(&field.name) {
                    return Err(LowerError::InvalidProviderConfigField {
                        service: service.name.clone(),
                        field: field.name.clone(),
                        known_fields: {
                            let mut v = allowed_fields.to_vec();
                            v.sort();
                            v
                        },
                    });
                }
            }
        }
        None => {
            // Interface implementations (stubs/in-memory adapters) can carry
            // service-local config fields that are not provider schemas.
            if service.implements.is_some() {
                return Ok(());
            }
            return Err(LowerError::UnknownProviderPrefix {
                service: service.name.clone(),
                fields: service
                    .config
                    .extra
                    .iter()
                    .map(|f| f.name.clone())
                    .collect(),
                known_prefixes: {
                    let mut v: Vec<String> = schemas.iter().map(|(p, _)| p.clone()).collect();
                    v.sort();
                    v
                },
            });
        }
    }

    Ok(())
}

/// Derive provider config schemas from the DSL project.
///
/// Reads `provider_config_schemas` data declaration for prefix→schema mappings,
/// then resolves each schema type to extract its field names.
fn derive_provider_schemas_from_project(
    project: &TypedProject,
) -> Result<Vec<(String, Vec<String>)>, LowerError> {
    // Collect all type definitions keyed by name.
    let mut type_defs: HashMap<String, &Vec<Field>> = HashMap::new();
    for module in &project.graph().modules {
        for item in &module.ast.items {
            if let Item::TypeDef(td) = &item.node {
                if let TypeBody::Record(fields) = &td.body {
                    type_defs.insert(td.name.clone(), fields);
                }
            }
        }
    }

    // Find the provider_config_schemas data declaration.
    let mut schemas = Vec::new();
    for module in project.modules() {
        for item in &module.ast.items {
            if let Item::DataDef(def) = &item.node {
                if def.name == "provider_config_schemas" {
                    // The value is a list of records: [{prefix: "gcs.", schema: "GcsProviderConfig"}, ...]
                    if let Expr::List(entries) = &def.value {
                        for entry in entries {
                            if let Expr::Record(_, fields) = entry {
                                let prefix = fields.iter().find_map(|(k, v)| {
                                    if k == "prefix" {
                                        if let Expr::Literal(Literal::String(s)) = v {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                });
                                let schema_name = fields.iter().find_map(|(k, v)| {
                                    if k == "schema" {
                                        if let Expr::Literal(Literal::String(s)) = v {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                });
                                if let (Some(prefix), Some(schema)) = (prefix, schema_name) {
                                    let field_names = type_defs
                                        .get(&schema)
                                        .map(|fields| {
                                            fields.iter().map(|f| f.name.clone()).collect()
                                        })
                                        .ok_or_else(|| LowerError::UnknownProviderSchemaType {
                                            prefix: prefix.clone(),
                                            schema: schema.clone(),
                                        })?;
                                    schemas.push((prefix, field_names));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(schemas)
}

fn derive_service_transport_triplets(
    project: &TypedProject,
    required_calls: Option<&HashSet<String>>,
) -> Result<transport::TransportManifest, LowerError> {
    let data_registry = build_data_registry(project);
    let mut manifest = transport::TransportManifest::new();
    for module in project.modules() {
        let module_name = module.module_path.as_dotted();
        let source_file = module.path.display().to_string();
        for item in &module.ast.items {
            let Item::ServiceDef(service) = &item.node else {
                continue;
            };

            // SR-8: Validate provider-specific config fields against
            // schemas derived from dsl/std/provider_config.dag.
            validate_provider_config_fields(service, project)?;

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
                validate_rest_output_field_paths(service, operation)?;
                let service_metadata =
                    derive_service_call_metadata(service, operation, &data_registry)?;
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

                // SC-7: Validate auth_input references a real Secret-typed field.
                if let Some(ref auth_field_name) = service.config.auth_input {
                    if operation.transport.is_some() {
                        let matching_field =
                            operation.inputs.iter().find(|f| f.name == *auth_field_name);
                        match matching_field {
                            None => {
                                return Err(LowerError::InvalidAuthInput {
                                    service: service.name.clone(),
                                    operation: operation.name.clone(),
                                    field_name: auth_field_name.clone(),
                                    reason: format!(
                                        "field `{auth_field_name}` not found in operation inputs"
                                    ),
                                });
                            }
                            Some(field) => {
                                if !is_secret_type(&field.ty) {
                                    let field_type = type_expr_to_string(&field.ty);
                                    return Err(LowerError::InvalidAuthInput {
                                        service: service.name.clone(),
                                        operation: operation.name.clone(),
                                        field_name: auth_field_name.clone(),
                                        reason: format!(
                                            "field must be type `Secret`, found `{field_type}`"
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }

                let suffix = sanitize_identifier(&format!(
                    "{module_name}_{}_{}",
                    service.name, operation.name
                ));
                let origin = pattern_expansion_origin(
                    &source_file,
                    &module_name,
                    &service.name,
                    item.span,
                    "service_transport",
                );
                let prepare_id = format!("prepare_transport_{suffix}");
                let execute_id = format!("execute_transport_{suffix}");
                let parse_id = format!("parse_transport_{suffix}");
                let prepare_ports = service_prepare_ports(operation, &service_metadata);
                let prepare_inputs = prepare_ports
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect::<Vec<_>>();

                let has_auth = matches!(
                    &service_metadata.spec,
                    Some(ServiceOperationSpec::Rest(spec)) if spec.auth_scheme.is_some()
                );
                let execute_extra_inputs = if has_auth {
                    vec![Port::optional(PortName::RESOURCE_CREDENTIAL, "Credential")]
                } else {
                    vec![]
                };
                let parse_outputs = if operation.outputs.is_empty() {
                    vec![Port::scalar("result", "Unit")]
                } else {
                    operation
                        .outputs
                        .iter()
                        .map(|field| {
                            let ty = type_expr_to_string(&field.ty);
                            Port::scalar(field.name.as_str(), ty.as_str())
                        })
                        .collect::<Vec<_>>()
                };

                let triplet_spec = transport::TransportTripletSpec {
                    module: module_name.clone(),
                    service: service.name.clone(),
                    operation: operation.name.clone(),
                    metadata: service_metadata.clone(),
                    prepare_id: prepare_id.clone(),
                    execute_id: execute_id.clone(),
                    parse_id: parse_id.clone(),
                    prepare_inputs: prepare_ports,
                    execute_extra_inputs,
                    parse_outputs,
                    execute_parse_wiring: transport::ExecuteParseWiring::Response,
                    origin: Some(origin),
                    operation_key: Some(OperationKey::new(&service.name, &operation.name)),
                };
                transport::emit_triplet_to_manifest(
                    &mut manifest,
                    transport::build_transport_triplet(triplet_spec),
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
                manifest.registry.register(
                    format!("{}.{}", service.name, operation.name),
                    endpoint.clone(),
                );
                let service_tail = service
                    .name
                    .rsplit('.')
                    .next()
                    .unwrap_or(service.name.as_str());
                manifest.registry.register(
                    format!("{service_tail}.{}", operation.name),
                    endpoint.clone(),
                );
                manifest.registry.register(
                    format!("{}.{}.{}", module_name, service.name, operation.name),
                    endpoint,
                );
            }
        }
    }
    Ok(manifest)
}

/// Collect lowered fn bodies for all pure `fn` definitions in the project.
///
/// These are passed as `sibling_fns` to structural nodes (e.g. MatchDispatch)
/// so the runtime evaluator can execute calls to user-defined pure fns.
fn collect_project_fn_bodies(
    project: &TypedProject,
    variant_names: &HashSet<String>,
) -> Result<std::collections::BTreeMap<String, LoweredFnBody>, LowerError> {
    let mode = expr::ExprLowerMode::Remap;
    let mut fn_bodies = std::collections::BTreeMap::new();
    for module in project.modules() {
        for item in &module.ast.items {
            let Item::FnDef(def) = &item.node else {
                continue;
            };
            // Build the lowered fn body from the fn's stmts.
            let stmts = &def.body.stmts;
            let mut fn_stmts: Vec<expr::LoweredStmt> = Vec::new();
            for stmt in stmts {
                match stmt {
                    Stmt::Node(ns) => {
                        return Err(LowerError::PureFnContainsEffectfulNode {
                            fn_name: def.name.clone(),
                            node_name: ns.name.clone(),
                        });
                    }
                    _ => {
                        fn_stmts.push(expr::lower_stmt_with_mode(stmt, variant_names, mode));
                    }
                }
            }
            // If the fn has params and a trailing expression but no explicit
            // return, wrap the trailing expression as a return.
            if fn_stmts.is_empty() {
                continue;
            }
            let param_types: Vec<(String, String)> = def
                .params
                .iter()
                .map(|p| {
                    (
                        p.name.clone(),
                        daglang_syntax::ast_utils::type_expr_to_string(&p.ty),
                    )
                })
                .collect();
            let return_type = Some(daglang_syntax::ast_utils::type_expr_to_string(
                &def.return_type,
            ));
            let body = LoweredFnBody::with_types(fn_stmts, param_types, return_type);
            fn_bodies.insert(def.name.clone(), body);
        }
    }
    Ok(fn_bodies)
}

/// Collect the subset of `all_fn_bodies` that an expression transitively calls.
fn collect_called_fn_bodies(
    expr: &Expr,
    all_fn_bodies: &std::collections::BTreeMap<String, LoweredFnBody>,
) -> std::collections::BTreeMap<String, LoweredFnBody> {
    let mut called = HashSet::new();
    collect_call_names(expr, &mut called);
    let mut result = std::collections::BTreeMap::new();
    for name in called {
        if let Some(body) = all_fn_bodies.get(&name) {
            result.insert(name, body.clone());
        }
    }
    result
}

fn collect_call_names(expr: &Expr, names: &mut HashSet<String>) {
    match expr {
        Expr::Call(name, args) => {
            names.insert(name.clone());
            for (_, arg) in args {
                collect_call_names(arg, names);
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                collect_call_names(arg, names);
            }
        }
        Expr::FieldAccess(base, _) => collect_call_names(base, names),
        Expr::BinOp(l, _, r) => {
            collect_call_names(l, names);
            collect_call_names(r, names);
        }
        Expr::UnaryOp(_, inner) | Expr::Lambda(_, inner) | Expr::After(inner, _) => {
            collect_call_names(inner, names);
        }
        Expr::If(c, t, e) => {
            collect_call_names(c, names);
            collect_call_names(t, names);
            if let Some(e) = e {
                collect_call_names(e, names);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_call_names(item, names);
            }
        }
        Expr::Record(_, fields) | Expr::Return(fields) => {
            for (_, v) in fields {
                collect_call_names(v, names);
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_call_names(scrutinee, names);
            for arm in arms {
                collect_call_names(&arm.body, names);
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_call_names(inner, names);
                }
            }
        }
        Expr::For(_, iterable, _, body) => {
            collect_call_names(iterable, names);
            match body {
                daglang_syntax::ast::ForBody::Expr(expr) => collect_call_names(expr, names),
                daglang_syntax::ast::ForBody::Block(stmts) => {
                    walk_stmts(stmts, &mut |expr| collect_call_names(expr, names));
                }
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_call_names(inner, names);
            collect_call_names(guard, names);
        }
        Expr::Map(entries) => {
            for (k, v) in entries {
                collect_call_names(k, names);
                collect_call_names(v, names);
            }
        }
        _ => {}
    }
}

fn add_service_call_edges(
    builder: &mut DagBuilder,
    project: &TypedProject,
    wctx: &DagWiringContext<'_>,
    active_profile_bindings: Option<&ActiveProfileBindings>,
    profile_bound_interfaces: &HashSet<String>,
    known_interface_types: &HashSet<String>,
) -> Result<(), LowerError> {
    // Collect all pure fn bodies so structural nodes can call them at runtime.
    let all_fn_bodies = collect_project_fn_bodies(project, wctx.variant_names)?;
    // Track transport endpoint usage across ALL modules and callables so
    // that the second caller to reference the same service operation gets
    // a cloned triplet (_c1, _c2, …) instead of wiring duplicate scalar
    // edges to the original.
    let mut endpoint_use_count: HashMap<String, usize> = HashMap::new();
    // Track fn node usage across ALL modules and callables so that the
    // second caller to reference the same fn item gets a cloned copy.
    // This mirrors the transport triplet cloning pattern and prevents
    // shared fn nodes from receiving inputs from the wrong caller's
    // context after entrypoint slicing.
    let mut fn_node_use_count: HashMap<String, usize> = HashMap::new();
    for module in project.modules() {
        let module_name = module.module_path.as_dotted();
        let source_file = module.path.display().to_string();
        for item in &module.ast.items {
            let Some(callable) = item.node.as_callable() else {
                continue;
            };
            let item_name = callable.name();
            let params = callable.params();
            let stmts = callable.body_stmts();
            let uses_binding_types: HashMap<String, String> = callable
                .uses_clauses()
                .iter()
                .map(|usage| {
                    (
                        usage.binding.clone(),
                        resource_type_name(&usage.resource_type),
                    )
                })
                .collect();
            let Some(target) = wctx
                .endpoints_by_full
                .get(&(module_name.clone(), item_name.to_string()))
            else {
                continue;
            };
            let param_types = params
                .iter()
                .map(|param| (param.name.clone(), type_expr_to_string(&param.ty)))
                .collect::<HashMap<_, _>>();
            let mut bound_callable_sources = collect_bound_callable_sources(
                module_name.as_str(),
                stmts,
                wctx.endpoints_by_full,
                wctx.endpoints_by_name,
            );
            // Clone fn nodes that were already wired by a previous caller.
            // Without cloning, shared fn nodes receive inputs from the first
            // caller only (has_edge_to_port guard in wire_fn_call_arguments),
            // and entrypoint slicing then pulls in the wrong caller's
            // transport nodes via backward reachability through the fn node.
            let mut fn_name_overrides: HashMap<String, LoweredEndpoint> = HashMap::new();
            for stmt in stmts {
                let (binding, fn_name) = match stmt {
                    Stmt::Let(b, expr) | Stmt::Assign(b, expr) => match unwrap_guarded_expr(expr) {
                        Expr::Call(name, _) => (b.as_str(), name.as_str()),
                        _ => continue,
                    },
                    Stmt::Node(node_stmt) => match unwrap_guarded_expr(&node_stmt.expr) {
                        Expr::Call(name, _) => (node_stmt.name.as_str(), name.as_str()),
                        _ => continue,
                    },
                    _ => continue,
                };
                let Some(endpoint) = bound_callable_sources.get(binding) else {
                    continue;
                };
                let count = fn_node_use_count
                    .entry(endpoint.node_id.clone())
                    .or_insert(0);
                *count += 1;
                if *count <= 1 {
                    continue;
                }
                // Only clone fn_body nodes (fn items). Func items use
                // passthrough wiring (__out: ports) which doesn't carry
                // over to clones — cloning them would produce nodes that
                // fail with "missing required declared output passthrough".
                let Some(original_node) = builder
                    .dag
                    .nodes
                    .iter()
                    .find(|n| n.id.0 == endpoint.node_id)
                    .cloned()
                else {
                    continue;
                };
                let has_fn_body = matches!(
                    &original_node.body,
                    gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable {
                        fn_body: Some(_),
                        ..
                    })
                );
                if !has_fn_body {
                    continue;
                }
                let clone_id = format!("{}_fc{}", endpoint.node_id, *count - 1);
                let mut cloned_node = original_node;
                cloned_node.id = clone_id.clone().into();
                builder.add_node(cloned_node);
                let cloned_ep = LoweredEndpoint {
                    node_id: clone_id,
                    primary_output: endpoint.primary_output.clone(),
                };
                fn_name_overrides.insert(fn_name.to_string(), cloned_ep.clone());
                // Redirect the __deps edge from the original fn node to the
                // clone. add_callable_nodes created a __deps edge from the
                // original endpoint to this callable's target, but the clone
                // is the actual data supplier for this caller.
                let original_id = endpoint.node_id.clone();
                let original_output = endpoint.primary_output.clone();
                bound_callable_sources.insert(binding.to_string(), cloned_ep.clone());
                for edge in &mut builder.dag.edges {
                    if edge.from_node.0 == original_id
                        && edge.from_port.0 == original_output
                        && edge.to_node.0 == target.node_id
                        && edge.to_port.0 == PortName::DEPS
                    {
                        edge.from_node = cloned_ep.node_id.clone().into();
                        break;
                    }
                }
            }
            // Build per-callable endpoints_by_name with overrides if any
            // fn nodes were cloned, so wire_fn_call_arguments and service
            // call arg wiring resolve to the correct per-caller clones.
            let effective_endpoints_by_name_owned;
            let endpoints_for_ctx: &HashMap<String, Option<LoweredEndpoint>> =
                if fn_name_overrides.is_empty() {
                    wctx.endpoints_by_name
                } else {
                    let mut m = wctx.endpoints_by_name.clone();
                    for (name, ep) in fn_name_overrides {
                        m.insert(name, Some(ep));
                    }
                    effective_endpoints_by_name_owned = m;
                    &effective_endpoints_by_name_owned
                };
            let caller = format!("{module_name}::{item_name}");
            let mut bound_service_sources = collect_bound_service_sources(
                caller.as_str(),
                stmts,
                &uses_binding_types,
                wctx.service_registry,
                active_profile_bindings,
                profile_bound_interfaces,
                known_interface_types,
            )?;
            let service_local_let_bindings =
                collect_local_let_bindings(stmts, &bound_callable_sources);
            let empty_expanded = HashMap::new();
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
                    wctx.service_registry,
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
                let service_ctx = LoweringContext {
                    source_file: &source_file,
                    module_name: module_name.as_str(),
                    item_name,
                    item_span: item.span,
                    param_types: &param_types,
                    endpoints_by_name: endpoints_for_ctx,
                    data_values: wctx.data_values,
                    service_registry: wctx.service_registry,
                    bound_callable_sources: &bound_callable_sources,
                    bound_service_sources: &bound_service_sources,
                    expanded_results: &empty_expanded,
                    local_let_bindings: &service_local_let_bindings,
                    body_stmts: stmts,
                    all_fn_bodies: &all_fn_bodies,
                    variant_names: wctx.variant_names,
                    callable_param_defaults: wctx.callable_param_defaults,
                    endpoints_by_full: wctx.endpoints_by_full,
                    uses_binding_types: &uses_binding_types,
                };
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
                    if let Err(e) = wire_service_call_arg_to_port(
                        builder,
                        &service_ctx,
                        arg,
                        effective_endpoint.prepare_node_id.as_str(),
                        prepare_input,
                        format!("{call_index}_{index}").as_str(),
                    ) {
                        return Err(LowerError::WiringFailure {
                            source_file: source_file.to_string(),
                            detail: format!(
                                "service call arg `{prepare_input}` wiring failed: {e}"
                            ),
                        });
                    }
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
                            } else if let Some(bound_source) = bound_callable_sources.get(arg_ident)
                            {
                                builder.add_edge(
                                    bound_source.node_id.as_str(),
                                    bound_source.primary_output.as_str(),
                                    effective_endpoint.execute_node_id.as_str(),
                                    PortName::RESOURCE_CREDENTIAL,
                                );
                            } else if let Some(bound_source) = bound_service_sources.get(arg_ident)
                            {
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
                            } else if let Some(bound_source) = bound_service_sources.get(base_ident)
                            {
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
            let local_let_bindings = collect_local_let_bindings(stmts, &augmented_callable_sources);
            let fn_ctx = LoweringContext {
                source_file: &source_file,
                module_name: module_name.as_str(),
                item_name,
                item_span: item.span,
                param_types: &param_types,
                endpoints_by_name: endpoints_for_ctx,
                data_values: wctx.data_values,
                service_registry: wctx.service_registry,
                bound_callable_sources: &augmented_callable_sources,
                bound_service_sources: &bound_service_sources,
                expanded_results: &empty_expanded,
                local_let_bindings: &local_let_bindings,
                body_stmts: stmts,
                all_fn_bodies: &all_fn_bodies,
                variant_names: wctx.variant_names,
                callable_param_defaults: wctx.callable_param_defaults,
                endpoints_by_full: wctx.endpoints_by_full,
                uses_binding_types: &uses_binding_types,
            };
            wire_fn_call_arguments(builder, &fn_ctx, stmts)?;
            // Wire for-loop iterable expressions to loop node "items" ports.
            let loop_ctx = LoweringContext {
                bound_callable_sources: &bound_callable_sources,
                ..fn_ctx
            };
            wire_for_loop_iterables(builder, &loop_ctx, stmts, target);
            // Skip return wiring for fn items: FnBodyCallableOp evaluates
            // the body directly. Enabling passthrough wiring for these items
            // would be unsafe because callable endpoints are shared and
            // argument wiring is first-write-only — multiple call sites could
            // make a function return values from an unrelated invocation.
            if !matches!(&item.node, Item::FnDef(_)) {
                match wire_callable_return_outputs(builder, &fn_ctx, stmts, target) {
                    Ok(()) => {}
                    // Return bindings that reference service-call results or
                    // collection operations cannot be statically wired yet —
                    // the runtime evaluates the body directly for these cases.
                    Err(LowerError::ExprLower(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }
    }
    for module in project.modules() {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::PipelineDef(def) = &item.node else {
                continue;
            };
            let Some(target) = wctx
                .endpoints_by_full
                .get(&(module_name.clone(), def.name.clone()))
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
                        wctx.service_registry,
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

fn wire_service_call_arg_to_port(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    arg: &ServiceCallArgSite,
    dest_node_id: &str,
    dest_port: &str,
    disambiguator: &str,
) -> Result<(), LowerError> {
    if let Some(arg_ident) = arg.ident.as_deref() {
        if let Some(param_ty) = ctx.param_types.get(arg_ident) {
            let param_source = ensure_param_source_node(
                builder,
                ctx.module_name,
                ctx.item_name,
                arg_ident,
                param_ty.as_str(),
            );
            builder.add_edge(param_source.as_str(), arg_ident, dest_node_id, dest_port);
            return Ok(());
        }
        if let Some(bound_source) = ctx.bound_callable_sources.get(arg_ident) {
            builder.add_edge(
                bound_source.node_id.as_str(),
                bound_source.primary_output.as_str(),
                dest_node_id,
                dest_port,
            );
            return Ok(());
        }
        if let Some(bound_source) = ctx.bound_service_sources.get(arg_ident) {
            builder.add_edge(
                bound_source.parse.node_id.as_str(),
                bound_source.parse.primary_output.as_str(),
                dest_node_id,
                dest_port,
            );
            return Ok(());
        }
        if let Some(json_val) = ctx.data_values.get(arg_ident) {
            let literal = ServiceCallArgLiteral::Json(json_val.clone());
            let literal_source = ensure_literal_source_node(
                builder,
                ctx.module_name,
                ctx.item_name,
                dest_port,
                "Any",
                &literal,
                disambiguator,
            );
            builder.add_edge(literal_source.as_str(), dest_port, dest_node_id, dest_port);
            return Ok(());
        }
    }

    if let Some((base_ident, field_name)) = arg.field_access.as_ref() {
        if let Some(bound_source) = ctx.bound_callable_sources.get(base_ident) {
            builder.add_edge(
                bound_source.node_id.as_str(),
                field_name.as_str(),
                dest_node_id,
                dest_port,
            );
            return Ok(());
        }
        if let Some(bound_source) = ctx.bound_service_sources.get(base_ident) {
            builder.add_edge(
                bound_source.parse.node_id.as_str(),
                field_name.as_str(),
                dest_node_id,
                dest_port,
            );
            return Ok(());
        }
        if let Some(param_ty) = ctx.param_types.get(base_ident.as_str()) {
            let param_source = ensure_param_source_node(
                builder,
                ctx.module_name,
                ctx.item_name,
                base_ident,
                param_ty.as_str(),
            );
            builder.add_edge(
                param_source.as_str(),
                field_name.as_str(),
                dest_node_id,
                dest_port,
            );
            return Ok(());
        }
    }

    if let Some(call_name) = arg.call.as_deref() {
        if let Some(Some(call_source)) = ctx.endpoints_by_name.get(call_name) {
            builder.add_edge(
                call_source.node_id.as_str(),
                call_source.primary_output.as_str(),
                dest_node_id,
                dest_port,
            );
            return Ok(());
        }
    }

    if let Some(literal) = arg.literal.as_ref() {
        let literal_source = ensure_literal_source_node(
            builder,
            ctx.module_name,
            ctx.item_name,
            dest_port,
            "Any",
            literal,
            disambiguator,
        );
        builder.add_edge(literal_source.as_str(), dest_port, dest_node_id, dest_port);
        return Ok(());
    }

    Err(LowerError::ExprLower(format!(
        "cannot wire service call argument '{}' on {}.{} (no structural source found)",
        dest_port, ctx.module_name, ctx.item_name
    )))
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
            let capability =
                call_path
                    .last()
                    .cloned()
                    .ok_or_else(|| LowerError::UnresolvedServiceCall {
                        caller: caller.to_string(),
                        service_call: call_path.join("."),
                    })?;
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
            let capability =
                call_path
                    .last()
                    .cloned()
                    .ok_or_else(|| LowerError::UnresolvedServiceCall {
                        caller: caller.to_string(),
                        service_call: call_path.join("."),
                    })?;
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
        return Err(LowerError::MissingConcreteBinding {
            profile: active_profile_bindings.profile_name.clone(),
            interface_type: canonical_resource_type_name(interface_type),
        });
    };
    let Some(binding) = active_profile_bindings
        .by_interface
        .get(interface_key.as_str())
    else {
        return Err(LowerError::MissingConcreteBinding {
            profile: active_profile_bindings.profile_name.clone(),
            interface_type: interface_key,
        });
    };
    let implementation_type = binding.implementation_type.as_str();
    let capability =
        call_path
            .last()
            .cloned()
            .ok_or_else(|| LowerError::UnresolvedServiceCall {
                caller: caller.to_string(),
                service_call: call_path.join("."),
            })?;
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
    for module in project.modules() {
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
    for module in project.modules() {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let stmts = match &item.node {
                Item::FuncDef(def) => def.body.stmts.as_slice(),
                // Patterns are templates expanded inline; their bodies
                // don't produce DAG nodes, so no auth wiring needed.
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
    for module in project.modules() {
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
    for module in project.modules() {
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
                    vec![Port::scalar(
                        provided.binding.as_str(),
                        resource_type.as_str(),
                    )],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!("resource_provide::{}::{}", item_name, provided.binding),
                        obligation: CallableObligation::ResourceProvide,
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

    for module in project.modules() {
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
    for module in project.modules() {
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
    for module in project.modules() {
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
                        obligation: CallableObligation::ResourceAcquire,
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
                        obligation: CallableObligation::ResourceRelease,
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
    for module in project.modules() {
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
                        obligation: CallableObligation::InterfaceContractVerification,
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
    for module in project.modules() {
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
            vec![Port::scalar(param, ty)],
            vec![Port::scalar(param, ty)],
            LoweredOp::Primitive {
                module: module_name.to_string(),
                name: format!("call_param_source::{callable}::{param}"),
                kind: PrimitiveOpKind::CallParamSource {
                    callable: callable.to_string(),
                    param: param.to_string(),
                },
            },
        )
        .with_kind(gunbc_ir::node::NodeKind::ParamSource)
        .with_input_alias(lowered_node_id(module_name, callable), param),
    );
    node_id
}

/// Post-processing: wire param_source inputs from the same sources that feed
/// their parent callable's param input port.
///
/// Param_source nodes are boundary injection points for callable parameters.
/// For the top-level entrypoint, their values arrive via `set_input()`. For
/// inner callables (fn A calls fn B, forwarding a param), the param_source
/// node has no incoming edge — it never receives the value.
///
/// This pass finds each param_source node, identifies the callable it belongs
/// to, and duplicates any incoming edge to that callable's param port so the
/// param_source also receives the value.
fn wire_param_source_inputs(builder: &mut DagBuilder) {
    // Collect param_source info: (param_source_node_id, callable_node_id, param_name)
    let param_sources: Vec<(String, String, String)> = builder
        .dag
        .nodes
        .iter()
        .filter_map(|node| {
            if let gunbc_ir::NodeBody::Opaque(LoweredOp::Primitive {
                module,
                kind: PrimitiveOpKind::CallParamSource { callable, param },
                ..
            }) = &node.body
            {
                let callable_node_id = lowered_node_id(module, callable);
                Some((node.id.0.clone(), callable_node_id, param.clone()))
            } else {
                None
            }
        })
        .collect();

    // For each param_source, find edges feeding into the callable's param port
    // and duplicate them to feed the param_source.
    for (ps_node_id, callable_node_id, param_name) in &param_sources {
        // Skip if the param_source already has an incoming edge (e.g., top-level entrypoint
        // where boundary injection handles it, or already wired by a previous iteration).
        if builder.has_edge_to_port(ps_node_id, param_name) {
            continue;
        }

        // Find edges that deliver data to the callable's param input port.
        let sources: Vec<(String, String)> = builder
            .dag
            .edges
            .iter()
            .filter(|e| {
                e.to_node.0 == *callable_node_id
                    && e.to_port.0 == *param_name
                    && e.kind.carries_data()
            })
            .map(|e| (e.from_node.0.clone(), e.from_port.0.clone()))
            .collect();

        for (from_node, from_port) in sources {
            builder.add_edge(&from_node, &from_port, ps_node_id, param_name);
        }
    }
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
        vec![Port::scalar(param, ty)],
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
    item.as_callable()
        .map(|c| {
            c.uses_clauses()
                .iter()
                .map(|u| (u.binding.clone(), resource_type_name(&u.resource_type)))
                .collect()
        })
        .unwrap_or_default()
}

fn item_callable_body(item: &Item) -> Option<(&str, &[Stmt])> {
    item.as_callable().map(|c| (c.name(), c.body_stmts()))
}

fn item_callable_interactive_flag(item: &Item) -> Option<(&str, bool)> {
    item.as_callable().map(|c| (c.name(), false))
}

fn item_callable_uses(item: &Item) -> Option<(&str, &[daglang_syntax::ast::UsesClause])> {
    let c = item.as_callable()?;
    let uses = c.uses_clauses();
    if uses.is_empty() {
        None
    } else {
        Some((c.name(), uses))
    }
}

fn item_callable_provides(item: &Item) -> Option<(&str, &[daglang_syntax::ast::ProvidesClause])> {
    let c = item.as_callable()?;
    let provides = c.provides_clauses();
    if provides.is_empty() {
        None
    } else {
        Some((c.name(), provides))
    }
}

fn is_internal_synthetic_call(name: &str) -> bool {
    matches!(name, "<expr>" | "as" | "with" | "fn")
}

fn collect_calls_from_stmts(stmts: &[Stmt], calls: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr)
            | Stmt::Assign(_, expr)
            | Stmt::Expr(expr)
            | Stmt::Node(NodeStmt { expr, .. }) => {
                collect_direct_call_names(expr, calls);
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    collect_direct_call_names(expr, calls);
                }
            }
        }
    }
}

fn collect_direct_call_names(expr: &Expr, calls: &mut BTreeSet<String>) {
    match expr {
        Expr::Call(name, args) => {
            if !is_internal_synthetic_call(name) {
                calls.insert(name.clone());
            }
            for (_, arg) in args {
                collect_direct_call_names(arg, calls);
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                collect_direct_call_names(arg, calls);
            }
        }
        Expr::FieldAccess(base, _) => collect_direct_call_names(base, calls),
        Expr::BinOp(left, _, right) => {
            collect_direct_call_names(left, calls);
            collect_direct_call_names(right, calls);
        }
        Expr::UnaryOp(_, inner) | Expr::After(inner, _) => {
            collect_direct_call_names(inner, calls);
        }
        Expr::Guarded(inner, guard) => {
            collect_direct_call_names(inner, calls);
            collect_direct_call_names(guard, calls);
        }
        Expr::If(condition, _, _) => collect_direct_call_names(condition, calls),
        Expr::List(items) => {
            for item in items {
                collect_direct_call_names(item, calls);
            }
        }
        Expr::Record(_, fields) | Expr::Return(fields) => {
            for (_, value) in fields {
                collect_direct_call_names(value, calls);
            }
        }
        Expr::Match(scrutinee, _) => collect_direct_call_names(scrutinee, calls),
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_direct_call_names(inner, calls);
                }
            }
        }
        Expr::For(_, iterable, _, _) => collect_direct_call_names(iterable, calls),
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_direct_call_names(key, calls);
                collect_direct_call_names(value, calls);
            }
        }
        Expr::Block(stmts) => {
            for stmt in stmts {
                match stmt {
                    Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                        collect_direct_call_names(expr, calls);
                    }
                    Stmt::Node(node_stmt) => {
                        collect_direct_call_names(&node_stmt.expr, calls);
                    }
                    Stmt::Return(fields) => {
                        for (_, expr) in fields {
                            collect_direct_call_names(expr, calls);
                        }
                    }
                }
            }
        }
        Expr::Lambda(_, _) | Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ServiceCallArgSite {
    pub(crate) name: Option<String>,
    pub(crate) expr: Expr,
    pub(crate) ident: Option<String>,
    pub(crate) field_access: Option<(String, String)>,
    pub(crate) call: Option<String>,
    pub(crate) literal: Option<ServiceCallArgLiteral>,
}

#[derive(Debug, Clone)]
pub(crate) struct ServiceCallSite {
    pub(crate) path: Vec<String>,
    pub(crate) args: Vec<ServiceCallArgSite>,
}

#[derive(Debug, Clone)]
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

/// Re-export from `gunbc_ir` — the canonical definition now lives in the IR layer.
pub type CollectionOpKind = gunbc_ir::patterns::CollectionKind;

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
    // Delegates to CollectionKind::from_name — single source of truth (S11).
    CollectionOpKind::from_name(name)
}

fn collect_collection_ops_from_stmts(stmts: &[Stmt], sites: &mut Vec<CollectionOpSite>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Call(name, _) = expr {
            if let Some(kind) = collection_op_kind(name) {
                sites.push(CollectionOpSite { kind });
            }
        }
    });
}

fn derive_collection_node_specs(callable_node_id: &str, stmts: &[Stmt]) -> Vec<CollectionNodeSpec> {
    let mut sites = Vec::new();
    collect_collection_ops_from_stmts(stmts, &mut sites);
    // With standalone call syntax (no pipes), walker visits in statement order
    // which is already the correct pipeline order. No reversal needed.
    sites
        .into_iter()
        .enumerate()
        .map(|(index, site)| CollectionNodeSpec {
            node_id: format!("{callable_node_id}::{}_{index}", site.kind.node_label()),
            kind: site.kind,
        })
        .collect()
}

#[derive(Debug)]
struct CollectionLoweringPlan {
    nodes: Vec<Node<LoweredOp>>,
    edges: Vec<(String, String, String, String)>,
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
                Port::scalar("items", "Any"),
                Port::list(PortName::DEPS, "Any"),
            ],
            vec![Port::scalar("items", "Any")],
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
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr)
            | Stmt::Assign(_, expr)
            | Stmt::Expr(expr)
            | Stmt::Node(NodeStmt { expr, .. }) => {
                collect_direct_fn_calls_with_args(expr, calls);
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    collect_direct_fn_calls_with_args(expr, calls);
                }
            }
        }
    }
}

fn collect_direct_fn_calls_with_args(expr: &Expr, calls: &mut Vec<FnCallSite>) {
    match expr {
        Expr::Call(name, args) => {
            if !is_internal_synthetic_call(name) {
                calls.push(FnCallSite {
                    name: name.clone(),
                    args: args.iter().map(service_call_arg_site).collect(),
                });
            }
            for (_, arg) in args {
                collect_direct_fn_calls_with_args(arg, calls);
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                collect_direct_fn_calls_with_args(arg, calls);
            }
        }
        Expr::FieldAccess(base, _) => collect_direct_fn_calls_with_args(base, calls),
        Expr::BinOp(left, _, right) => {
            collect_direct_fn_calls_with_args(left, calls);
            collect_direct_fn_calls_with_args(right, calls);
        }
        Expr::UnaryOp(_, inner) | Expr::After(inner, _) => {
            collect_direct_fn_calls_with_args(inner, calls);
        }
        Expr::Guarded(inner, guard) => {
            collect_direct_fn_calls_with_args(inner, calls);
            collect_direct_fn_calls_with_args(guard, calls);
        }
        Expr::If(condition, _, _) => collect_direct_fn_calls_with_args(condition, calls),
        Expr::List(items) => {
            for item in items {
                collect_direct_fn_calls_with_args(item, calls);
            }
        }
        Expr::Record(_, fields) | Expr::Return(fields) => {
            for (_, value) in fields {
                collect_direct_fn_calls_with_args(value, calls);
            }
        }
        Expr::Match(scrutinee, _) => collect_direct_fn_calls_with_args(scrutinee, calls),
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_direct_fn_calls_with_args(inner, calls);
                }
            }
        }
        Expr::For(_, iterable, _, _) => collect_direct_fn_calls_with_args(iterable, calls),
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_direct_fn_calls_with_args(key, calls);
                collect_direct_fn_calls_with_args(value, calls);
            }
        }
        Expr::Block(stmts) => {
            for stmt in stmts {
                match stmt {
                    Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                        collect_direct_fn_calls_with_args(expr, calls);
                    }
                    Stmt::Node(node_stmt) => {
                        collect_direct_fn_calls_with_args(&node_stmt.expr, calls);
                    }
                    Stmt::Return(fields) => {
                        for (_, expr) in fields {
                            collect_direct_fn_calls_with_args(expr, calls);
                        }
                    }
                }
            }
        }
        Expr::Lambda(_, _) | Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

fn service_call_arg_site((name, arg): &(Option<String>, Expr)) -> ServiceCallArgSite {
    ServiceCallArgSite {
        name: name.clone(),
        expr: arg.clone(),
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
    for module in project.modules() {
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

/// Prefix for data declaration node IDs embedded in the DAG.
///
/// The resolver scans for nodes with this prefix to reconstruct data_values
/// without requiring a sidecar through the compilation pipeline.
pub const DATA_DECL_NODE_PREFIX: &str = "__data_decl::";

/// Embed data declaration values as `CallLiteralSource` nodes in the DAG.
///
/// Each data declaration becomes a standalone source node with ID
/// `__data_decl::{name}` and a `PrimitiveLiteral::Json` payload. The resolver
/// extracts these at resolution time via [`extract_data_values_from_dag`].
fn embed_data_declaration_nodes(
    builder: &mut DagBuilder,
    data_values: &HashMap<String, serde_json::Value>,
) {
    for (name, json_val) in data_values {
        let node_id = format!("{DATA_DECL_NODE_PREFIX}{name}");
        builder.add_node(Node::opaque(
            node_id,
            vec![],
            vec![Port::scalar(name.as_str(), "Json")],
            LoweredOp::Primitive {
                module: "__data".to_string(),
                name: format!("data_decl::{name}"),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::Json(json_val.clone()),
                },
            },
        ));
    }
}

/// Extract data declaration values from embedded DAG nodes.
///
/// Scans for nodes with IDs prefixed by [`DATA_DECL_NODE_PREFIX`] and extracts
/// their `PrimitiveLiteral::Json` payloads, converting to `Value` at the point
/// of extraction to avoid JSON round-trip lossy conversion (S53 fix).
pub fn extract_data_values_from_dag(dag: &Dag<LoweredOp>) -> HashMap<String, gunbc_ir::Value> {
    let mut data_values = HashMap::new();
    for node in &dag.nodes {
        if let Some(name) = node.id.0.strip_prefix(DATA_DECL_NODE_PREFIX) {
            if let gunbc_ir::NodeBody::Opaque(LoweredOp::Primitive {
                kind:
                    PrimitiveOpKind::CallLiteralSource {
                        literal: PrimitiveLiteral::Json(json),
                    },
                ..
            }) = &node.body
            {
                data_values.insert(name.to_string(), json_to_value(json));
            }
        }
    }
    data_values
}

/// Convert a `serde_json::Value` to a `gunbc_ir::Value`.
///
/// Recursively converts objects to `Value::Map` and arrays to `Value::List`.
fn json_to_value(json: &serde_json::Value) -> gunbc_ir::Value {
    match json {
        serde_json::Value::Null => gunbc_ir::Value::Unit,
        serde_json::Value::Bool(b) => gunbc_ir::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                gunbc_ir::Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                gunbc_ir::Value::Float(f)
            } else {
                gunbc_ir::Value::Str(n.to_string())
            }
        }
        serde_json::Value::String(s) => gunbc_ir::Value::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            gunbc_ir::Value::List(std::sync::Arc::new(arr.iter().map(json_to_value).collect()))
        }
        serde_json::Value::Object(map) => {
            let btree: std::collections::BTreeMap<String, gunbc_ir::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            gunbc_ir::Value::Map(btree)
        }
    }
}

/// Collect default expressions for callable parameters across all modules.
///
/// Returns a map from callable name → vec of (param_name, default_expr) for
/// params that have default values. Used by `wire_fn_call_arguments` to inject
/// literal source nodes for omitted call args.
fn collect_callable_param_defaults(
    project: &TypedProject,
) -> HashMap<String, Vec<(String, daglang_syntax::ast::Expr)>> {
    let mut defaults = HashMap::new();
    for module in project.modules() {
        for item in &module.ast.items {
            let Some(callable) = item.node.as_callable() else {
                continue;
            };
            let param_defaults: Vec<(String, daglang_syntax::ast::Expr)> = callable
                .params()
                .iter()
                .filter_map(|param| {
                    param
                        .default
                        .as_ref()
                        .map(|expr| (param.name.clone(), expr.clone()))
                })
                .collect();
            if !param_defaults.is_empty() {
                defaults.insert(callable.name().to_string(), param_defaults);
            }
        }
    }
    defaults
}

fn wire_fn_call_arguments(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    stmts: &[Stmt],
) -> Result<(), LowerError> {
    let mut fn_calls = Vec::new();
    collect_fn_calls_with_args(stmts, &mut fn_calls);
    for fn_call in &fn_calls {
        let Some(Some(fn_endpoint)) = ctx.endpoints_by_name.get(&fn_call.name) else {
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
                if let Some(source) = ctx.bound_callable_sources.get(base_ident) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        field_name.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                if let Some(source) = ctx.bound_service_sources.get(base_ident) {
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
                if let Some(param_ty) = ctx.param_types.get(arg_ident) {
                    let src = ensure_param_source_node(
                        builder,
                        ctx.module_name,
                        ctx.item_name,
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
                if let Some(source) = ctx.bound_callable_sources.get(arg_ident) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        source.primary_output.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                if let Some(source) = ctx.bound_service_sources.get(arg_ident) {
                    builder.add_edge(
                        source.parse.node_id.as_str(),
                        source.parse.primary_output.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                // Wire data declaration references as JSON literal source nodes.
                if let Some(json_val) = ctx.data_values.get(arg_ident) {
                    let literal = ServiceCallArgLiteral::Json(json_val.clone());
                    let src = ensure_literal_source_node(
                        builder,
                        ctx.module_name,
                        ctx.item_name,
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
                    ctx.module_name,
                    ctx.item_name,
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
                continue;
            }

            let output_port = Port::scalar(param_name, "Any");
            let expr_for_arg = arg
                .ident
                .as_deref()
                .and_then(|ident| ctx.local_let_bindings.get(ident).copied())
                .unwrap_or(&arg.expr);
            match lower_expr(
                builder,
                ctx,
                expr_for_arg,
                &output_port,
                param_name,
                &format!("arg_{index}"),
            ) {
                Ok((src_node, src_port)) => {
                    builder.add_edge(
                        src_node.as_str(),
                        src_port.as_str(),
                        fn_endpoint.node_id.as_str(),
                        param_name,
                    );
                    continue;
                }
                Err(e) => {
                    if let Some((src_node, src_port)) = synthesize_expr_value_fallback(
                        builder,
                        ctx,
                        expr_for_arg,
                        &output_port,
                        param_name,
                        &format!("arg_{index}"),
                    ) {
                        builder.add_edge(
                            src_node.as_str(),
                            src_port.as_str(),
                            fn_endpoint.node_id.as_str(),
                            param_name,
                        );
                        continue;
                    }
                    return Err(LowerError::from(format!(
                        "cannot wire fn call argument `{}` in {}.{}: {e}",
                        param_name, ctx.module_name, ctx.item_name
                    )));
                }
            }
        }
        // Inject default values for callable params that were omitted from the call.
        if let Some(param_defaults) = ctx.callable_param_defaults.get(&fn_call.name) {
            for (param_name, default_expr) in param_defaults {
                if builder.has_edge_to_port(fn_endpoint.node_id.as_str(), param_name) {
                    continue;
                }
                if let Some(json_val) = expr_to_json_literal(default_expr, ctx.variant_names) {
                    let literal = ServiceCallArgLiteral::Json(json_val);
                    let src = ensure_literal_source_node(
                        builder,
                        ctx.module_name,
                        ctx.item_name,
                        param_name,
                        "Any",
                        &literal,
                        &format!("default_{param_name}"),
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
    Ok(())
}

fn collect_return_bindings(stmts: &[Stmt], output_ports: &[Port]) -> Vec<(String, Expr)> {
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

    if output_names.len() == 1 {
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

fn lower_expr(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    expr: &Expr,
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    let expr = unwrap_return_expr(expr);
    match expr {
        Expr::Ident(name) => {
            if let Some(param_ty) = ctx.param_types.get(name) {
                let src = ensure_param_source_node(
                    builder,
                    ctx.module_name,
                    ctx.item_name,
                    name,
                    param_ty.as_str(),
                );
                return Ok((src, name.clone()));
            }
            if let Some(source) = ctx.bound_callable_sources.get(name) {
                return Ok((source.node_id.clone(), source.primary_output.clone()));
            }
            if let Some(source) = ctx.bound_service_sources.get(name) {
                return Ok((
                    source.parse.node_id.clone(),
                    source.parse.primary_output.clone(),
                ));
            }
            if let Some(result) = ctx.expanded_results.get(name.as_str()) {
                if let Some(first_output) = result.return_outputs.values().next() {
                    return Ok((
                        first_output.node_id.clone(),
                        first_output.output_port.clone(),
                    ));
                }
            }
            if let Some(Some(source)) = ctx.endpoints_by_name.get(name) {
                return Ok((source.node_id.clone(), source.primary_output.clone()));
            }
            if let Some(let_expr) = ctx.local_let_bindings.get(name) {
                return lower_expr(
                    builder,
                    ctx,
                    let_expr,
                    output_port,
                    output_name,
                    disambiguator,
                );
            }
            Err(LowerError::from(format!(
                "cannot lower expression in {}.{}: unresolved ident `{name}`",
                ctx.module_name, ctx.item_name
            )))
        }
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(base_ident) = base.as_ref() {
                if let Some(source) = ctx.bound_callable_sources.get(base_ident) {
                    return Ok((source.node_id.clone(), field.clone()));
                }
                if let Some(source) = ctx.bound_service_sources.get(base_ident) {
                    return Ok((source.parse.node_id.clone(), field.clone()));
                }
                if let Some(result) = ctx.expanded_results.get(base_ident.as_str()) {
                    if let Some(output) = result.return_outputs.get(field.as_str()) {
                        return Ok((output.node_id.clone(), output.output_port.clone()));
                    }
                }
                if let Some(Some(source)) = ctx.endpoints_by_name.get(base_ident) {
                    return Ok((source.node_id.clone(), field.clone()));
                }
                if let Some(bound_expr) = ctx.local_let_bindings.get(base_ident.as_str()) {
                    let any_port = Port::scalar("result", "Any");
                    let (base_node, base_port) = lower_expr(
                        builder,
                        ctx,
                        bound_expr,
                        &any_port,
                        &format!("{output_name}_base"),
                        &format!("{disambiguator}_base_expr"),
                    )?;
                    return synthesize_get_field_on_resolved(
                        builder,
                        ctx,
                        &base_node,
                        &base_port,
                        field,
                        output_port,
                        output_name,
                        disambiguator,
                    )
                    .ok_or_else(|| {
                        LowerError::from(format!(
                            "cannot lower expression in {}.{}: field access on resolved base",
                            ctx.module_name, ctx.item_name
                        ))
                    });
                }
                // C24: For parameters, synthesize a GetField node.
                if let Some(param_ty) = ctx.param_types.get(base_ident) {
                    return synthesize_get_field(
                        builder,
                        ctx,
                        base_ident,
                        param_ty,
                        field,
                        output_port,
                        disambiguator,
                    )
                    .ok_or_else(|| {
                        LowerError::from(format!(
                            "cannot lower expression in {}.{}: get field on param `{base_ident}`",
                            ctx.module_name, ctx.item_name
                        ))
                    });
                }
            }
            // C24: For complex base expressions, try recursive resolution.
            // If the base resolves, add a structural GetField node.
            let any_port = Port::scalar("result", "Any");
            let (base_node, base_port) = lower_expr(
                builder,
                ctx,
                base,
                &any_port,
                &format!("{output_name}_base"),
                &format!("{disambiguator}_base"),
            )?;
            synthesize_get_field_on_resolved(
                builder,
                ctx,
                &base_node,
                &base_port,
                field,
                output_port,
                output_name,
                disambiguator,
            )
            .ok_or_else(|| {
                LowerError::from(format!(
                    "cannot lower expression in {}.{}: field access on complex base",
                    ctx.module_name, ctx.item_name
                ))
            })
        }
        Expr::Call(name, _) => ctx
            .endpoints_by_name
            .get(name)
            .and_then(|entry| entry.clone())
            .map(|source| (source.node_id, source.primary_output))
            .ok_or_else(|| {
                LowerError::from(format!(
                    "cannot lower expression in {}.{}: unresolved call `{name}`",
                    ctx.module_name, ctx.item_name
                ))
            }),
        Expr::Literal(_) | Expr::Map(_) => {
            let literal = return_literal_arg(expr).ok_or_else(|| {
                LowerError::from(format!(
                    "cannot lower expression in {}.{}: non-literal in literal position",
                    ctx.module_name, ctx.item_name
                ))
            })?;
            let src = ensure_literal_source_node(
                builder,
                ctx.module_name,
                ctx.item_name,
                output_name,
                output_port.type_id.0.as_str(),
                &literal,
                disambiguator,
            );
            Ok((src, output_name.to_string()))
        }
        // C24: String interpolation — try literal path first, then structural.
        Expr::StringInterp(parts) => {
            if let Some(literal) = return_literal_arg(expr) {
                let src = ensure_literal_source_node(
                    builder,
                    ctx.module_name,
                    ctx.item_name,
                    output_name,
                    output_port.type_id.0.as_str(),
                    &literal,
                    disambiguator,
                );
                Ok((src, output_name.to_string()))
            } else {
                synthesize_string_interpolate(
                    builder,
                    ctx,
                    parts,
                    output_port,
                    output_name,
                    disambiguator,
                )
            }
        }
        // C24: List literal — try literal path first, then structural.
        Expr::List(elements) => {
            if let Some(literal) = return_literal_arg(expr) {
                let src = ensure_literal_source_node(
                    builder,
                    ctx.module_name,
                    ctx.item_name,
                    output_name,
                    output_port.type_id.0.as_str(),
                    &literal,
                    disambiguator,
                );
                Ok((src, output_name.to_string()))
            } else {
                synthesize_list_construct(
                    builder,
                    ctx,
                    elements,
                    output_port,
                    output_name,
                    disambiguator,
                )
            }
        }
        // C24-P1: Direct BinOp → BinaryOp structural node.
        Expr::BinOp(left, op, right) => {
            // Sub-expressions may have different types from the BinOp result
            // (e.g., `a + b > 5` — operands are Int, result is Bool).
            let any_port = Port::scalar("result", "Any");
            let left_source = lower_expr(
                builder,
                ctx,
                left,
                &any_port,
                &format!("{output_name}_lhs"),
                &format!("{disambiguator}_lhs"),
            );
            let right_source = lower_expr(
                builder,
                ctx,
                right,
                &any_port,
                &format!("{output_name}_rhs"),
                &format!("{disambiguator}_rhs"),
            );
            match (left_source, right_source) {
                (Ok((l_node, l_port)), Ok((r_node, r_port))) => synthesize_binary_op(
                    builder,
                    ctx,
                    op,
                    &l_node,
                    &l_port,
                    &r_node,
                    &r_port,
                    output_port,
                    output_name,
                    disambiguator,
                )
                .ok_or_else(|| {
                    LowerError::from(format!(
                        "cannot lower expression in {}.{}",
                        ctx.module_name, ctx.item_name
                    ))
                }),
                _ => Err(LowerError::from(format!(
                    "cannot resolve operand in {}.{}",
                    ctx.module_name, ctx.item_name
                ))),
            }
        }
        // C24-P1: Direct Match → MatchDispatch structural node.
        Expr::Match(scrutinee, arms) => synthesize_match_dispatch(
            builder,
            ctx,
            scrutinee,
            arms,
            output_port,
            output_name,
            disambiguator,
        ),
        // C24-P2: If/Else → Conditional structural node.
        Expr::If(cond, then_, else_) => synthesize_conditional(
            builder,
            ctx,
            cond,
            then_,
            else_.as_deref(),
            output_port,
            output_name,
            disambiguator,
        ),
        // C24-P2: UnaryOp → UnaryOp structural node.
        Expr::UnaryOp(op, inner) => {
            let any_port = Port::scalar("result", "Any");
            let inner_source = lower_expr(
                builder,
                ctx,
                inner,
                &any_port,
                &format!("{output_name}_inner"),
                &format!("{disambiguator}_inner"),
            );
            match inner_source {
                Ok((src_node, src_port)) => synthesize_unary_op(
                    builder,
                    ctx,
                    op,
                    &src_node,
                    &src_port,
                    output_port,
                    output_name,
                    disambiguator,
                )
                .ok_or_else(|| {
                    LowerError::from(format!(
                        "cannot lower expression in {}.{}",
                        ctx.module_name, ctx.item_name
                    ))
                }),
                Err(_) => Err(LowerError::from(format!(
                    "cannot resolve operand in {}.{}",
                    ctx.module_name, ctx.item_name
                ))),
            }
        }
        // C24-P2: Tagged variant record → VariantConstruct; plain record → RecordConstruct.
        Expr::Record(Some(tag), fields)
            if ctx.variant_names.contains(tag.as_str()) || tag == "Some" || tag == "None" =>
        {
            synthesize_variant_construct(
                builder,
                ctx,
                tag,
                fields,
                output_port,
                output_name,
                disambiguator,
            )
        }
        Expr::Record(_, fields) => synthesize_record_construct(
            builder,
            ctx,
            fields,
            output_port,
            output_name,
            disambiguator,
        ),
        Expr::For(..) => Err(LowerError::from(format!(
            "unsupported expression in {}.{}: for not yet structuralized",
            ctx.module_name, ctx.item_name
        ))),
        Expr::ServiceCall(_path, _args) => Err(LowerError::from(format!(
            "unsupported expression in {}.{}: ServiceCall",
            ctx.module_name, ctx.item_name
        ))),
        Expr::Lambda(_params, _body) => Err(LowerError::from(format!(
            "unsupported expression in {}.{}: Lambda",
            ctx.module_name, ctx.item_name
        ))),
        Expr::Guarded(_inner, _guard) => Err(LowerError::from(format!(
            "unsupported expression in {}.{}: Guarded",
            ctx.module_name, ctx.item_name
        ))),
        Expr::After(_inner, _deps) => Err(LowerError::from(format!(
            "unsupported expression in {}.{}: After",
            ctx.module_name, ctx.item_name
        ))),
        Expr::Return(_fields) => Err(LowerError::from(format!(
            "unsupported expression in {}.{}: Return",
            ctx.module_name, ctx.item_name
        ))),
        Expr::Block(_stmts) => Err(LowerError::from(format!(
            "unsupported expression in {}.{}: Block",
            ctx.module_name, ctx.item_name
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExprLeafRef {
    input_port: String,
    source: expr::LeafRef,
}

/// C10: Collect binding names introduced by a match arm pattern.
fn collect_pattern_bindings(pattern: &daglang_syntax::ast::Pattern, seen: &mut HashSet<String>) {
    match pattern {
        daglang_syntax::ast::Pattern::Ident(name) => {
            // Only treat as a binding if it's lowercase (variant names are uppercase).
            if name.chars().next().is_some_and(|c| c.is_lowercase()) {
                seen.insert(name.clone());
            }
        }
        daglang_syntax::ast::Pattern::Variant(_, fields) => {
            for (_, inner) in fields {
                collect_pattern_bindings(inner, seen);
            }
        }
        daglang_syntax::ast::Pattern::Wildcard | daglang_syntax::ast::Pattern::Literal(_) => {}
    }
}

/// Collect all leaf expression references from a complex expression.
/// Sets `has_local_refs` to true if the expression references local variables
/// (let bindings) that can't be resolved as compute node inputs.
fn collect_expr_leaf_refs(
    expr: &Expr,
    ctx: &LoweringContext<'_>,
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
            if name == "None" || name == "null" || ctx.variant_names.contains(name.as_str()) {
                return;
            }
            if let Some(param_ty) = ctx.param_types.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Param {
                        name: name.clone(),
                        field: None,
                        ty: param_ty.clone(),
                    },
                });
            } else if let Some(source) = ctx.bound_callable_sources.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Callable {
                        endpoint: source.node_id.clone(),
                        port: source.primary_output.clone(),
                    },
                });
            } else if let Some(source) = ctx.bound_service_sources.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Service {
                        endpoint: source.parse.node_id.clone(),
                        port: source.parse.primary_output.clone(),
                    },
                });
            } else if let Some(Some(source)) = ctx.endpoints_by_name.get(name) {
                seen.insert(port_name.clone());
                refs.push(ExprLeafRef {
                    input_port: port_name,
                    source: expr::LeafRef::Callable {
                        endpoint: source.node_id.clone(),
                        port: source.primary_output.clone(),
                    },
                });
            } else if let Some(bound_expr) = ctx.local_let_bindings.get(name) {
                // C10: Resolve through local let binding. The let stmt will be
                // included in the fn body; here we collect its transitive DAG
                // dependencies. Only recurse if not already visited (prevents
                // infinite recursion on cyclic/self-referential bindings).
                if seen.insert(port_name) {
                    collect_expr_leaf_refs(bound_expr, ctx, refs, seen, has_local_refs);
                }
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
                if let Some(param_ty) = ctx.param_types.get(base_ident) {
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
                } else if let Some(source) = ctx.bound_callable_sources.get(base_ident) {
                    seen.insert(port_name.clone());
                    refs.push(ExprLeafRef {
                        input_port: port_name,
                        source: expr::LeafRef::Callable {
                            endpoint: source.node_id.clone(),
                            port: field.clone(),
                        },
                    });
                } else if let Some(source) = ctx.bound_service_sources.get(base_ident) {
                    seen.insert(port_name.clone());
                    refs.push(ExprLeafRef {
                        input_port: port_name,
                        source: expr::LeafRef::Service {
                            endpoint: source.parse.node_id.clone(),
                            port: field.clone(),
                        },
                    });
                } else if let Some(Some(source)) = ctx.endpoints_by_name.get(base_ident) {
                    seen.insert(port_name.clone());
                    refs.push(ExprLeafRef {
                        input_port: port_name,
                        source: expr::LeafRef::Callable {
                            endpoint: source.node_id.clone(),
                            port: field.clone(),
                        },
                    });
                } else if let Some(bound_expr) = ctx.local_let_bindings.get(base_ident.as_str()) {
                    // C10: Field access on a local let binding. Resolve
                    // transitively through the binding's expression to capture
                    // its DAG dependencies. Only recurse if not already visited.
                    if seen.insert(base_ident.to_string()) {
                        collect_expr_leaf_refs(bound_expr, ctx, refs, seen, has_local_refs);
                    }
                }
            } else {
                collect_expr_leaf_refs(base, ctx, refs, seen, has_local_refs);
            }
        }
        Expr::BinOp(left, _, right) => {
            collect_expr_leaf_refs(left, ctx, refs, seen, has_local_refs);
            collect_expr_leaf_refs(right, ctx, refs, seen, has_local_refs);
        }
        Expr::UnaryOp(_, inner) => {
            collect_expr_leaf_refs(inner, ctx, refs, seen, has_local_refs);
        }
        Expr::If(cond, then_, else_) => {
            collect_expr_leaf_refs(cond, ctx, refs, seen, has_local_refs);
            collect_expr_leaf_refs(then_, ctx, refs, seen, has_local_refs);
            if let Some(e) = else_ {
                collect_expr_leaf_refs(e, ctx, refs, seen, has_local_refs);
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_expr_leaf_refs(scrutinee, ctx, refs, seen, has_local_refs);
            for arm in arms {
                // C10: Match arm bindings are scoped to the arm — clone `seen`
                // so bindings from one arm don't leak into subsequent arms.
                let mut arm_seen = seen.clone();
                collect_pattern_bindings(&arm.pattern, &mut arm_seen);
                collect_expr_leaf_refs(&arm.body, ctx, refs, &mut arm_seen, has_local_refs);
            }
        }
        Expr::Call(_, args) => {
            for (_, arg) in args {
                collect_expr_leaf_refs(arg, ctx, refs, seen, has_local_refs);
            }
        }
        Expr::Record(_, fields) => {
            for (_, field_expr) in fields {
                collect_expr_leaf_refs(field_expr, ctx, refs, seen, has_local_refs);
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_expr_leaf_refs(inner, ctx, refs, seen, has_local_refs);
                }
            }
        }
        Expr::List(elems) => {
            for elem in elems {
                collect_expr_leaf_refs(elem, ctx, refs, seen, has_local_refs);
            }
        }
        Expr::Lambda(params, body) => {
            // C10: Lambda parameters are locally scoped — clone `seen` so
            // bindings don't persist for subsequent expressions.
            let mut lambda_seen = seen.clone();
            for param in params {
                lambda_seen.insert(param.clone());
            }
            collect_expr_leaf_refs(body, ctx, refs, &mut lambda_seen, has_local_refs);
        }
        Expr::For(binding, iterable, _, body) => {
            // C10: Iterable uses parent scope; binding is scoped to body only.
            collect_expr_leaf_refs(iterable, ctx, refs, seen, has_local_refs);
            let mut body_seen = seen.clone();
            body_seen.insert(binding.clone());
            match body {
                daglang_syntax::ast::ForBody::Expr(expr) => {
                    collect_expr_leaf_refs(expr, ctx, refs, &mut body_seen, has_local_refs);
                }
                daglang_syntax::ast::ForBody::Block(stmts) => {
                    for stmt in stmts {
                        match stmt {
                            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                                collect_expr_leaf_refs(
                                    expr,
                                    ctx,
                                    refs,
                                    &mut body_seen,
                                    has_local_refs,
                                );
                                body_seen.insert(name.clone());
                            }
                            Stmt::Node(node_stmt) => {
                                collect_expr_leaf_refs(
                                    &node_stmt.expr,
                                    ctx,
                                    refs,
                                    &mut body_seen,
                                    has_local_refs,
                                );
                                if let Some(guard) = &node_stmt.when_guard {
                                    collect_expr_leaf_refs(
                                        guard,
                                        ctx,
                                        refs,
                                        &mut body_seen,
                                        has_local_refs,
                                    );
                                }
                                body_seen.insert(node_stmt.name.clone());
                            }
                            Stmt::Expr(expr) => {
                                collect_expr_leaf_refs(
                                    expr,
                                    ctx,
                                    refs,
                                    &mut body_seen,
                                    has_local_refs,
                                );
                            }
                            Stmt::Return(fields) => {
                                for (_, expr) in fields {
                                    collect_expr_leaf_refs(
                                        expr,
                                        ctx,
                                        refs,
                                        &mut body_seen,
                                        has_local_refs,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Expr::Return(fields) => {
            for (_, field_expr) in fields {
                collect_expr_leaf_refs(field_expr, ctx, refs, seen, has_local_refs);
            }
        }
        Expr::Block(stmts) => {
            for stmt in stmts {
                match stmt {
                    Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                        collect_expr_leaf_refs(expr, ctx, refs, seen, has_local_refs);
                        seen.insert(name.clone());
                    }
                    Stmt::Expr(expr) => {
                        collect_expr_leaf_refs(expr, ctx, refs, seen, has_local_refs);
                    }
                    Stmt::Node(node_stmt) => {
                        collect_expr_leaf_refs(&node_stmt.expr, ctx, refs, seen, has_local_refs);
                        if let Some(guard) = &node_stmt.when_guard {
                            collect_expr_leaf_refs(guard, ctx, refs, seen, has_local_refs);
                        }
                        seen.insert(node_stmt.name.clone());
                    }
                    Stmt::Return(fields) => {
                        for (_, expr) in fields {
                            collect_expr_leaf_refs(expr, ctx, refs, seen, has_local_refs);
                        }
                    }
                }
            }
        }
        Expr::Literal(_)
        | Expr::Map(_)
        | Expr::ServiceCall(_, _)
        | Expr::Guarded(_, _)
        | Expr::After(_, _) => {}
    }
}

/// C24: Synthesize a GetField node to extract a named field from a parameter.
fn synthesize_get_field(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    base_param: &str,
    param_ty: &str,
    field: &str,
    output_port: &Port,
    disambiguator: &str,
) -> Option<(String, String)> {
    let output_type = output_port.type_id.0.as_str();
    let result_port_name = "result";
    let input_ports = vec![Port::scalar(base_param, param_ty)];
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "get_field_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, base_param, field, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("get_field::{}::{}::{}", ctx.item_name, base_param, field),
            kind: PrimitiveOpKind::GetField {
                field: field.to_string(),
            },
        },
    ));

    // Wire from param source to GetField input.
    let param_source_id = ensure_param_source_node(
        builder,
        ctx.module_name,
        ctx.item_name,
        base_param,
        param_ty,
    );
    builder.add_edge(&param_source_id, base_param, &node_id, base_param);

    Some((node_id, result_port_name.to_string()))
}

/// C24-P1: Synthesize a BinaryOp structural node.
/// Both operands must already be resolved to (node_id, port) sources.
#[allow(clippy::too_many_arguments)]
fn synthesize_binary_op(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    op: &daglang_syntax::ast::BinOp,
    left_node: &str,
    left_port: &str,
    right_node: &str,
    right_port: &str,
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Option<(String, String)> {
    let lowered_op = expr::lower_binop(op);
    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();

    let input_ports = vec![Port::scalar("left", "Any"), Port::scalar("right", "Any")];
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "binary_op_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("binary_op::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::BinaryOp { op: lowered_op },
        },
    ));

    builder.add_edge(left_node, left_port, &node_id, "left");
    builder.add_edge(right_node, right_port, &node_id, "right");

    Some((node_id, result_port_name.to_string()))
}

fn wire_hoisted_callable_args(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    call_name: &str,
    endpoint: &LoweredEndpoint,
    args: &[(Option<String>, Expr)],
    output_name: &str,
    disambiguator: &str,
) -> Result<(), LowerError> {
    let Some(node) = builder
        .dag
        .get_node(&NodeId::new(endpoint.node_id.clone()))
        .cloned()
    else {
        return Ok(());
    };

    let param_ports: Vec<String> = node
        .inputs
        .iter()
        .filter(|port| is_user_param_port(port.name.0.as_str()))
        .map(|port| port.name.0.clone())
        .collect();

    for (index, (arg_name, arg_expr)) in args.iter().enumerate() {
        let Some(param_name) = arg_name.clone().or_else(|| param_ports.get(index).cloned()) else {
            continue;
        };
        if builder.has_edge_to_port(endpoint.node_id.as_str(), param_name.as_str()) {
            continue;
        }
        let arg_port = Port::scalar(param_name.as_str(), "Any");
        let (src_node, src_port) = lower_expr(
            builder,
            ctx,
            arg_expr,
            &arg_port,
            &format!("{output_name}_{param_name}"),
            &format!("{disambiguator}_{param_name}_{index}"),
        )?;
        builder.add_edge(
            src_node.as_str(),
            src_port.as_str(),
            endpoint.node_id.as_str(),
            param_name.as_str(),
        );
    }

    if let Some(param_defaults) = ctx.callable_param_defaults.get(call_name) {
        for (param_name, default_expr) in param_defaults {
            if builder.has_edge_to_port(endpoint.node_id.as_str(), param_name.as_str()) {
                continue;
            }
            if let Some(json_val) = expr_to_json_literal(default_expr, ctx.variant_names) {
                let literal = ServiceCallArgLiteral::Json(json_val);
                let src = ensure_literal_source_node(
                    builder,
                    ctx.module_name,
                    ctx.item_name,
                    param_name.as_str(),
                    "Any",
                    &literal,
                    &format!("{disambiguator}_default_{param_name}"),
                );
                builder.add_edge(
                    src.as_str(),
                    param_name.as_str(),
                    endpoint.node_id.as_str(),
                    param_name.as_str(),
                );
            }
        }
    }

    Ok(())
}

fn synthesize_callable_expr_value(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    call_name: &str,
    args: &[(Option<String>, Expr)],
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    let endpoint = ctx
        .endpoints_by_name
        .get(call_name)
        .and_then(|entry| entry.clone())
        .ok_or_else(|| {
            LowerError::from(format!(
                "callable endpoint not found for {call_name} in {}.{}",
                ctx.module_name, ctx.item_name
            ))
        })?;
    wire_hoisted_callable_args(
        builder,
        ctx,
        call_name,
        &endpoint,
        args,
        output_name,
        disambiguator,
    )?;

    let node = builder
        .dag
        .get_node(&NodeId::new(endpoint.node_id.clone()))
        .cloned()
        .ok_or_else(|| {
            LowerError::from(format!(
                "node not found for endpoint {} in {}.{}",
                endpoint.node_id, ctx.module_name, ctx.item_name
            ))
        })?;
    let output_fields: Vec<String> = node
        .outputs
        .iter()
        .filter(|port| !port.name.is_internal())
        .map(|port| port.name.0.clone())
        .collect();

    if output_fields.len() == 1 && output_fields[0] == "return" {
        return Ok((endpoint.node_id, "return".to_string()));
    }

    let input_ports: Vec<Port> = output_fields
        .iter()
        .map(|field| Port::scalar(field.as_str(), "Any"))
        .collect();
    let output_ports = vec![Port::scalar("result", "Any")];
    let node_id = format!(
        "record_construct_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("record_construct::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::RecordConstruct {
                fields: output_fields.clone(),
            },
        },
    ));
    for field in &output_fields {
        builder.add_edge(
            endpoint.node_id.as_str(),
            field.as_str(),
            &node_id,
            field.as_str(),
        );
    }

    Ok((node_id, "result".to_string()))
}

fn helper_leaf_input_port(leaf: &ExprLeafRef) -> String {
    match &leaf.source {
        expr::LeafRef::Param {
            name,
            field: Some(field),
            ..
        } => format!("{name}__{field}"),
        _ => leaf.input_port.clone(),
    }
}

fn synthesize_expr_value_fallback(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    expr: &Expr,
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Option<(String, String)> {
    let mut refs: Vec<ExprLeafRef> = Vec::new();
    let mut seen = HashSet::new();
    let mut has_local_refs = false;
    collect_expr_leaf_refs(expr, ctx, &mut refs, &mut seen, &mut has_local_refs);
    if has_local_refs {
        return None;
    }

    let mut input_ports = Vec::new();
    let mut seen_ports = HashSet::new();
    for leaf in &refs {
        let input_port = helper_leaf_input_port(leaf);
        if !seen_ports.insert(input_port.clone()) {
            continue;
        }
        let ty = match &leaf.source {
            expr::LeafRef::Param {
                ty, field: None, ..
            } => ty.as_str(),
            _ => "Any",
        };
        input_ports.push(Port::scalar(input_port.as_str(), ty));
    }

    let result_port_name = "result";
    let output_ports = vec![Port::scalar(
        result_port_name,
        output_port.type_id.0.as_str(),
    )];
    let node_id = format!(
        "expr_value_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );
    let fn_body = expr::lower_fn_body_with_mode(
        &daglang_syntax::ast::FnBody {
            stmts: vec![Stmt::Return(vec![("result".to_string(), expr.clone())])],
        },
        ctx.variant_names,
        expr::ExprLowerMode::Remap,
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Callable {
            module: ctx.module_name.to_string(),
            kind: CallableKind::Fn,
            name: format!("{}::expr_value::{}", ctx.item_name, output_name),
            obligation: CallableObligation::None,
            is_interactive: false,
            resource_target: None,
            fn_body: Some(Box::new(fn_body)),
        },
    ));

    let mut wired_ports = HashSet::new();
    for leaf in &refs {
        let input_port = helper_leaf_input_port(leaf);
        if !wired_ports.insert(input_port.clone()) {
            continue;
        }
        match &leaf.source {
            expr::LeafRef::Param {
                name,
                field: Some(field),
                ty,
            } => {
                let any_port = Port::scalar("result", "Any");
                let (src_node, src_port) = synthesize_get_field(
                    builder,
                    ctx,
                    name,
                    ty,
                    field,
                    &any_port,
                    &format!("{disambiguator}_{input_port}"),
                )?;
                builder.add_edge(&src_node, &src_port, &node_id, input_port.as_str());
            }
            expr::LeafRef::Param {
                name,
                field: None,
                ty,
            } => {
                let param_source_id =
                    ensure_param_source_node(builder, ctx.module_name, ctx.item_name, name, ty);
                builder.add_edge(&param_source_id, name, &node_id, input_port.as_str());
            }
            expr::LeafRef::Callable { endpoint, port }
            | expr::LeafRef::Service { endpoint, port } => {
                builder.add_edge(endpoint, port, &node_id, input_port.as_str());
            }
        }
    }

    Some((node_id, result_port_name.to_string()))
}

#[allow(clippy::type_complexity)]
fn lower_match_arm_for_dispatch(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    arm: &daglang_syntax::ast::MatchArm,
    output_name: &str,
    disambiguator: &str,
    arm_index: usize,
) -> Result<(expr::LoweredMatchArm, Option<(String, String, String)>), LowerError> {
    if let Expr::Call(call_name, args) = &arm.body {
        let input_port = format!("arm_body_{arm_index}");
        let (src_node, src_port) = synthesize_callable_expr_value(
            builder,
            ctx,
            call_name.as_str(),
            args,
            output_name,
            &format!(
                "{disambiguator}_arm_{arm_index}_{}",
                sanitize_identifier(call_name)
            ),
        )?;
        let mut hoisted_arm = arm.clone();
        hoisted_arm.body = Expr::Ident(input_port.clone());
        return Ok((
            expr::lower_match_arm(
                &hoisted_arm,
                ctx.variant_names,
                expr::ExprLowerMode::Standard,
            ),
            Some((input_port, src_node, src_port)),
        ));
    }

    Ok((
        expr::lower_match_arm(arm, ctx.variant_names, expr::ExprLowerMode::Standard),
        None,
    ))
}

/// C24-P1: Synthesize a MatchDispatch structural node.
/// Resolves the scrutinee and collects leaf refs from arm bodies.
fn synthesize_match_dispatch(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    scrutinee: &Expr,
    arms: &[daglang_syntax::ast::MatchArm],
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    // Collect all leaf refs from the entire match expression.
    let whole_expr = Expr::Match(Box::new(scrutinee.clone()), arms.to_vec());
    let mut refs: Vec<ExprLeafRef> = Vec::new();
    let mut seen = HashSet::new();
    let mut has_local_refs = false;
    collect_expr_leaf_refs(&whole_expr, ctx, &mut refs, &mut seen, &mut has_local_refs);

    if has_local_refs {
        return Err(LowerError::from(format!(
            "match dispatch has local refs in {}.{}",
            ctx.module_name, ctx.item_name
        )));
    }

    // Build input ports: "scrutinee" + all leaf refs from arm bodies (deduplicated).
    let mut input_ports = vec![Port::scalar("scrutinee", "Any")];
    let mut seen_ports = HashSet::new();
    seen_ports.insert("scrutinee".to_string());
    for leaf in &refs {
        if !seen_ports.insert(leaf.input_port.clone()) {
            continue;
        }
        let ty = match &leaf.source {
            expr::LeafRef::Param { ty, .. } => ty.as_str(),
            _ => "Any",
        };
        input_ports.push(Port::scalar(leaf.input_port.as_str(), ty));
    }

    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let mut hoisted_arm_sources = Vec::new();
    let mut lowered_arms: Vec<expr::LoweredMatchArm> = Vec::with_capacity(arms.len());
    for (arm_index, arm) in arms.iter().enumerate() {
        let (lowered, source) =
            lower_match_arm_for_dispatch(builder, ctx, arm, output_name, disambiguator, arm_index)?;
        if let Some(source) = source {
            hoisted_arm_sources.push(source);
        }
        lowered_arms.push(lowered);
    }
    for (input_port, _, _) in &hoisted_arm_sources {
        if seen_ports.insert(input_port.clone()) {
            input_ports.push(Port::scalar(input_port.as_str(), "Any"));
        }
    }

    // Wire scrutinee input — use Any-typed port (scrutinee type differs from match result).
    let any_port = Port::scalar("result", "Any");
    let (scrutinee_node, scrutinee_port) = lower_expr(
        builder,
        ctx,
        scrutinee,
        &any_port,
        &format!("{output_name}_scrutinee"),
        &format!("{disambiguator}_scrutinee"),
    )?;

    let node_id = format!(
        "match_dispatch_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    // Collect sibling fn bodies that arm expressions may call at runtime.
    let sibling_fns: std::collections::BTreeMap<String, LoweredFnBody> =
        collect_called_fn_bodies(&whole_expr, ctx.all_fn_bodies)
            .into_iter()
            .collect();

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("match_dispatch::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::MatchDispatch {
                arms: lowered_arms,
                sibling_fns,
            },
        },
    ));
    builder.add_edge(&scrutinee_node, &scrutinee_port, &node_id, "scrutinee");

    // Wire all other leaf ref inputs (skip already-wired ports).
    let mut wired_ports = HashSet::new();
    wired_ports.insert("scrutinee".to_string());
    for leaf in &refs {
        if !wired_ports.insert(leaf.input_port.clone()) {
            continue;
        }
        match &leaf.source {
            expr::LeafRef::Param { name, ty, .. } => {
                let param_source_id =
                    ensure_param_source_node(builder, ctx.module_name, ctx.item_name, name, ty);
                builder.add_edge(&param_source_id, name, &node_id, &leaf.input_port);
            }
            expr::LeafRef::Callable { endpoint, port }
            | expr::LeafRef::Service { endpoint, port } => {
                builder.add_edge(endpoint, port, &node_id, &leaf.input_port);
            }
        }
    }
    for (input_port, src_node, src_port) in &hoisted_arm_sources {
        if wired_ports.insert(input_port.clone()) {
            builder.add_edge(
                src_node.as_str(),
                src_port.as_str(),
                &node_id,
                input_port.as_str(),
            );
        }
    }

    Ok((node_id, result_port_name.to_string()))
}

/// C24-P2: Synthesize a Conditional structural node.
/// Resolves condition, then, and optional else branches.
#[allow(clippy::too_many_arguments)]
fn synthesize_conditional(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    cond: &Expr,
    then_: &Expr,
    else_: Option<&Expr>,
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    // Collect all leaf refs from the entire if/else expression.
    let whole_expr = Expr::If(
        Box::new(cond.clone()),
        Box::new(then_.clone()),
        else_.map(|e| Box::new(e.clone())),
    );
    let mut refs: Vec<ExprLeafRef> = Vec::new();
    let mut seen = HashSet::new();
    let mut has_local_refs = false;
    collect_expr_leaf_refs(&whole_expr, ctx, &mut refs, &mut seen, &mut has_local_refs);

    if has_local_refs {
        return Err(LowerError::from(format!(
            "conditional has local refs in {}.{}",
            ctx.module_name, ctx.item_name
        )));
    }

    // Build input ports: "condition", "then", "else" + all leaf refs.
    let mut input_ports = vec![
        Port::scalar("condition", "Bool"),
        Port::scalar("then", "Any"),
    ];
    if else_.is_some() {
        input_ports.push(Port::scalar("else", "Any"));
    }

    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    // Wire condition — use Bool-typed port (condition is always Bool).
    let bool_port = Port::scalar("result", "Bool");
    let (cond_node, cond_port) = lower_expr(
        builder,
        ctx,
        cond,
        &bool_port,
        &format!("{output_name}_cond"),
        &format!("{disambiguator}_cond"),
    )?;

    // Wire then branch.
    let (then_node, then_port) = lower_expr(
        builder,
        ctx,
        then_,
        output_port,
        &format!("{output_name}_then"),
        &format!("{disambiguator}_then"),
    )?;

    let else_source = if let Some(else_expr) = else_ {
        let source = lower_expr(
            builder,
            ctx,
            else_expr,
            output_port,
            &format!("{output_name}_else"),
            &format!("{disambiguator}_else"),
        )?;
        Some(source)
    } else {
        None
    };

    let node_id = format!(
        "conditional_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("conditional::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::Conditional,
        },
    ));
    builder.add_edge(&cond_node, &cond_port, &node_id, "condition");
    builder.add_edge(&then_node, &then_port, &node_id, "then");

    // Wire else branch.
    if let Some((else_node, else_port)) = else_source {
        builder.add_edge(&else_node, &else_port, &node_id, "else");
    }

    Ok((node_id, result_port_name.to_string()))
}

/// C24-P2: Synthesize a UnaryOp structural node.
#[allow(clippy::too_many_arguments)]
fn synthesize_unary_op(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    op: &daglang_syntax::ast::UnaryOp,
    inner_node: &str,
    inner_port: &str,
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Option<(String, String)> {
    let lowered_op = match op {
        daglang_syntax::ast::UnaryOp::Not => expr::LoweredUnaryOp::Not,
        daglang_syntax::ast::UnaryOp::Neg => expr::LoweredUnaryOp::Neg,
    };
    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();

    let input_ports = vec![Port::scalar("operand", "Any")];
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "unary_op_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("unary_op::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::UnaryOp { op: lowered_op },
        },
    ));

    builder.add_edge(inner_node, inner_port, &node_id, "operand");

    Some((node_id, result_port_name.to_string()))
}

/// Synthesize a VariantConstruct structural node for tagged sum-type constructors.
/// Like RecordConstruct but carries the variant tag for correct `_variant` tagging at runtime.
fn synthesize_variant_construct(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    tag: &str,
    fields: &[(String, Expr)],
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    // Unit variant (no fields) → emit a literal source with the tag string.
    if fields.is_empty() {
        let literal = ServiceCallArgLiteral::String(tag.to_string());
        let src = ensure_literal_source_node(
            builder,
            ctx.module_name,
            ctx.item_name,
            output_name,
            output_port.type_id.0.as_str(),
            &literal,
            disambiguator,
        );
        return Ok((src, output_name.to_string()));
    }

    // Payload variant — resolve each field, then emit VariantConstruct node.
    let any_port = Port::scalar("result", "Any");
    let mut field_sources: Vec<(String, String, String)> = Vec::new();
    for (field_name, field_expr) in fields {
        let (src_node, src_port) = lower_expr(
            builder,
            ctx,
            field_expr,
            &any_port,
            &format!("{output_name}_{field_name}"),
            &format!("{disambiguator}_{field_name}"),
        )?;
        field_sources.push((field_name.clone(), src_node, src_port));
    }

    let field_names: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();
    let input_ports: Vec<Port> = field_names
        .iter()
        .map(|name| Port::scalar(name.as_str(), "Any"))
        .collect();

    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "variant_construct_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("variant_construct::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::VariantConstruct {
                tag: tag.to_string(),
                fields: field_names,
            },
        },
    ));

    for (field_name, src_node, src_port) in &field_sources {
        builder.add_edge(src_node, src_port, &node_id, field_name);
    }

    Ok((node_id, result_port_name.to_string()))
}

/// C24-P2: Synthesize a RecordConstruct structural node.
/// Each field's expression is resolved to a source and wired as a named input.
fn synthesize_record_construct(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    fields: &[(String, Expr)],
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    // Resolve each field to a source.
    // Use Any-typed port for field values (individual fields differ from the record type).
    let any_port = Port::scalar("result", "Any");
    let mut field_sources: Vec<(String, String, String)> = Vec::new();
    for (field_name, field_expr) in fields {
        let (src_node, src_port) = lower_expr(
            builder,
            ctx,
            field_expr,
            &any_port,
            &format!("{output_name}_{field_name}"),
            &format!("{disambiguator}_{field_name}"),
        )?;
        field_sources.push((field_name.clone(), src_node, src_port));
    }

    let field_names: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();
    let input_ports: Vec<Port> = field_names
        .iter()
        .map(|name| Port::scalar(name.as_str(), "Any"))
        .collect();

    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "record_construct_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("record_construct::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::RecordConstruct {
                fields: field_names,
            },
        },
    ));

    for (field_name, src_node, src_port) in &field_sources {
        builder.add_edge(src_node, src_port, &node_id, field_name);
    }

    Ok((node_id, result_port_name.to_string()))
}

/// C24: Synthesize a ListConstruct node for list literals with resolvable elements.
/// Each element is resolved recursively; if all resolve, a structural node is emitted.
fn synthesize_list_construct(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    elements: &[Expr],
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    let any_port = Port::scalar("result", "Any");

    // Resolve each element recursively.
    let mut elem_sources: Vec<(String, String, String)> = Vec::new();
    for (i, elem) in elements.iter().enumerate() {
        let port_name = format!("elem_{i}");
        let (node, port) = lower_expr(
            builder,
            ctx,
            elem,
            &any_port,
            &format!("{output_name}_{port_name}"),
            &format!("{disambiguator}_{port_name}"),
        )?;
        elem_sources.push((port_name, node, port));
    }

    let input_ports: Vec<Port> = elem_sources
        .iter()
        .map(|(name, _, _)| Port::scalar(name.as_str(), "Any"))
        .collect();
    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "list_construct_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("list_construct::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::ListConstruct {
                count: elements.len(),
            },
        },
    ));

    for (port_name, src_node, src_port) in &elem_sources {
        builder.add_edge(src_node, src_port, &node_id, port_name);
    }

    Ok((node_id, result_port_name.to_string()))
}

/// C24: Synthesize a StringInterpolate node for string interpolations with variable refs.
/// Each interpolated expression becomes an input port.
fn synthesize_string_interpolate(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    string_parts: &[daglang_syntax::ast::StringPart],
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Result<(String, String), LowerError> {
    let any_port = Port::scalar("result", "Any");

    let mut parts: Vec<String> = Vec::new();
    let mut input_port_names: Vec<String> = Vec::new();
    let mut input_sources: Vec<(String, String, String)> = Vec::new();

    // Walk through parts: literals become template strings, exprs become input ports.
    let mut current_literal = String::new();
    for (i, part) in string_parts.iter().enumerate() {
        match part {
            daglang_syntax::ast::StringPart::Literal(s) => {
                current_literal.push_str(s);
            }
            daglang_syntax::ast::StringPart::Expr(expr) => {
                parts.push(std::mem::take(&mut current_literal));
                let port_name = format!("interp_{i}");
                let (node, port) = lower_expr(
                    builder,
                    ctx,
                    expr,
                    &any_port,
                    &format!("{output_name}_{port_name}"),
                    &format!("{disambiguator}_{port_name}"),
                )?;
                input_sources.push((port_name.clone(), node, port));
                input_port_names.push(port_name);
            }
        }
    }
    // Final trailing literal.
    parts.push(current_literal);

    let input_ports: Vec<Port> = input_port_names
        .iter()
        .map(|name| Port::scalar(name.as_str(), "Any"))
        .collect();
    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "string_interpolate_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("string_interpolate::{}::{}", ctx.item_name, output_name),
            kind: PrimitiveOpKind::StringInterpolate {
                parts,
                input_ports: input_port_names,
            },
        },
    ));

    for (port_name, src_node, src_port) in &input_sources {
        builder.add_edge(src_node, src_port, &node_id, port_name);
    }

    Ok((node_id, result_port_name.to_string()))
}

/// C24: Synthesize a GetField node for a complex base expression (not a direct parameter).
/// Recursively resolves the base, then extracts the field from its output.
#[allow(clippy::too_many_arguments)]
fn synthesize_get_field_on_resolved(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    base_node: &str,
    base_port: &str,
    field: &str,
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Option<(String, String)> {
    let result_port_name = "result";
    let output_type = output_port.type_id.0.as_str();
    let input_port_name = "base";
    let input_ports = vec![Port::scalar(input_port_name, "Any")];
    let output_ports = vec![Port::scalar(result_port_name, output_type)];

    let node_id = format!(
        "get_field_{}",
        sanitize_identifier(&format!(
            "{}_{}_{}_{}_{}",
            ctx.module_name, ctx.item_name, field, output_name, disambiguator
        ))
    );

    builder.add_node(Node::opaque(
        node_id.clone(),
        input_ports,
        output_ports,
        LoweredOp::Primitive {
            module: ctx.module_name.to_string(),
            name: format!("get_field::{}::{}::{}", ctx.item_name, output_name, field),
            kind: PrimitiveOpKind::GetField {
                field: field.to_string(),
            },
        },
    ));

    builder.add_edge(base_node, base_port, &node_id, input_port_name);

    Some((node_id, result_port_name.to_string()))
}

#[must_use = "ignoring wiring errors can silently drop return bindings"]
fn wire_callable_return_outputs(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
) -> Result<(), LowerError> {
    let outputs = match builder.dag.get_node(&NodeId::new(target.node_id.clone())) {
        Some(node) => node.outputs.clone(),
        None => return Ok(()),
    };
    let output_bindings = collect_return_bindings(stmts, &outputs);
    if output_bindings.is_empty() {
        return Ok(());
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
        let (source_node, source_port) = lower_expr(
            builder,
            ctx,
            &expr,
            &output_port,
            output_name.as_str(),
            &format!("return_{index}"),
        )
        .map_err(|e| {
            LowerError::ExprLower(format!(
                "wire_callable_return_outputs: `{}` output `{}` cannot be lowered: {e}",
                target.node_id, output_name
            ))
        })?;
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
    Ok(())
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
    ctx: &LoweringContext<'_>,
    stmts: &[Stmt],
    target: &LoweredEndpoint,
) {
    let for_sites = detect_for_loops_in_stmts(stmts);
    for (index, site) in for_sites.iter().enumerate() {
        let loop_node_id = format!("{}::cf_for_{index}", target.node_id);
        let Some(iterable) = &site.iterable else {
            continue;
        };
        match iterable {
            IterableRef::FieldAccess(base_ident, field_name) => {
                if let Some(source) = ctx.bound_callable_sources.get(base_ident) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        field_name.as_str(),
                        loop_node_id.as_str(),
                        "items",
                    );
                } else if let Some(source) = ctx.bound_service_sources.get(base_ident) {
                    builder.add_edge(
                        source.parse.node_id.as_str(),
                        field_name.as_str(),
                        loop_node_id.as_str(),
                        "items",
                    );
                }
            }
            IterableRef::Ident(name) => {
                if let Some(param_ty) = ctx.param_types.get(name) {
                    let src = ensure_param_source_node(
                        builder,
                        ctx.module_name,
                        ctx.item_name,
                        name,
                        param_ty.as_str(),
                    );
                    builder.add_edge(src.as_str(), name, loop_node_id.as_str(), "items");
                } else if let Some(source) = ctx.bound_callable_sources.get(name) {
                    builder.add_edge(
                        source.node_id.as_str(),
                        source.primary_output.as_str(),
                        loop_node_id.as_str(),
                        "items",
                    );
                } else if let Some(source) = ctx.bound_service_sources.get(name) {
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

/// Collect local let bindings from statements that are NOT tracked by
/// `collect_bound_callable_sources` (i.e., non-call, non-alias expressions).
/// These are let bindings with evaluable expressions like pipe chains,
/// binary ops, if/match, literals, etc.
fn collect_local_let_bindings<'a>(
    stmts: &'a [Stmt],
    bound_callable_sources: &HashMap<String, LoweredEndpoint>,
) -> HashMap<String, &'a Expr> {
    let mut bindings = HashMap::new();
    for stmt in stmts {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                // Skip call-bound let stmts (tracked by bound_callable_sources)
                // and direct Call/ServiceCall expressions (may not be evaluable
                // by the simple evaluator if the callee isn't a built-in).
                // Unwrap Guarded/After wrappers before checking — guarded service
                // calls like `run = svc.Op() [when cond]` are still service calls.
                let inner = unwrap_guarded_expr(expr);
                let is_dag_level_call = match inner {
                    Expr::Call(callee, _) => {
                        // Intrinsic collection functions (map, filter, fold, etc.)
                        // are evaluated in the expression evaluator, not as DAG
                        // callable nodes. Include them in local let bindings.
                        !daglang_eval::eval::is_intrinsic_call(callee)
                    }
                    Expr::ServiceCall(_, _) => true,
                    _ => false,
                };
                if !bound_callable_sources.contains_key(name) && !is_dag_level_call {
                    bindings.insert(name.clone(), expr as &Expr);
                }
            }
            Stmt::Node(node_stmt) => {
                let inner = unwrap_guarded_expr(&node_stmt.expr);
                let is_dag_level_call = match inner {
                    Expr::Call(callee, _) => {
                        // Mirror the Stmt::Let logic: intrinsic collection
                        // functions are evaluable and should be included.
                        !daglang_eval::eval::is_intrinsic_call(callee)
                    }
                    Expr::ServiceCall(_, _) => true,
                    _ => false,
                };
                if !bound_callable_sources.contains_key(&node_stmt.name) && !is_dag_level_call {
                    bindings.insert(node_stmt.name.clone(), &node_stmt.expr as &Expr);
                }
            }
            _ => {}
        }
    }
    bindings
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
            }) => match unwrap_guarded_expr(expr) {
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
        if let gunbc_ir::node::NodeBody::SubDag(sub, _kind) = &node.body {
            collect_output_paths_recursive(&sub.nodes, paths);
        }
    }
}

/// An entrypoint inferred from graph structure.
///
/// A `func` item whose user-facing input ports are not all wired by
/// incoming edges is an entrypoint — its untapped inputs must be
/// supplied by the caller (CLI, REST, Lambda, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferredEntrypoint {
    /// The func name as declared in DSL (e.g., "makegen").
    pub func_name: String,
    /// The module path (e.g., "tools.makegen").
    pub module: String,
    /// The node ID in the lowered DAG (e.g., "tools.makegen::makegen").
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
    !pn.is_internal()
        && !pn.is_tool()
        && !pn.is_resource()
        && !is_output_passthrough_input(port_name)
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

#[cfg(test)]
mod tests;
