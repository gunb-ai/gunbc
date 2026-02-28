//! Central resolver: `LoweredOp` -> `DynOp` via existing domain ops.
//!
//! Maps each lowered operation from a compiled `.dag` file to its concrete
//! `Executable` implementation, wrapped in `DynOp`. This eliminates the need
//! for per-module union enums (`PragmaGraphOp`, `WorkspaceOp`, etc.).
//!
//! # Architecture
//!
//! Resolution has two layers:
//!
//! 1. **Infrastructure** (cross-module): Typed lowered primitive nodes
//!    (`LoweredOp::Primitive`) map to shared primitive/transport ops.
//!
//! 2. **Domain** (per-module): Module-specific callables (e.g., `tools.pragma`
//!    / `render_clippy_toml`) map to their domain op variants.
//!
//! # Adding a new module
//!
//! To wire a new `.dag` module:
//! - **Passthrough callables** (forward inputs to outputs): no Rust changes
//!   needed — the resolver defaults to passthrough for any callable the
//!   compiler validated.
//! - **Custom callables**: add a match arm in `resolve_domain()` for the
//!   module path and map each callable to its `DynOp`.
//! - Infrastructure nodes (content_upsert, fs_env) are handled automatically.

use std::collections::HashMap;

use daglang_lower::{
    CallableKind, CollectionOpKind, LoweredOp, PrimitiveLiteral,
    PrimitiveOpKind, ServiceCallMetadata, ServiceOperationSpec,
};
use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::node::NodeBody;
use gunbc_ir::resource::{AccessMode, RESOURCE_FILE, RESOURCE_FILE_PREFIX};
use gunbc_ir::transport::{FileRequest, TransportRequest};
use gunbc_ir::types::PortName;
use gunbc_ir::{Cardinality, Dag, Edge, Node, Port, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};

use crate::resolve_service::{
    GenericFileParseOp, GenericFilePrepareOp, GenericLocalParseOp, GenericLocalPrepareOp,
    GenericRestParseOp, GenericRestPrepareOp, GenericShellParseOp, GenericShellPrepareOp,
};

// ============================================================================
// Error type
// ============================================================================

/// Error resolving a `LoweredOp` to a concrete `DynOp`.
#[derive(Debug, Clone)]
pub struct ResolveError {
    pub node_id: String,
    pub reason: String,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "resolve error at `{}`: {}", self.node_id, self.reason)
    }
}

impl std::error::Error for ResolveError {}

fn declared_output_names(outputs: &[Port]) -> Vec<String> {
    outputs.iter().map(|p| p.name.0.clone()).collect()
}

fn execute_with_declared_output_passthrough(
    output_port_names: &[String],
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let mut outputs = HashMap::new();
    for (key, value) in &inputs {
        if key.starts_with(PortName::OUTPUT_PASSTHROUGH_PREFIX) {
            continue;
        }
        outputs.insert(key.clone(), value.clone());
    }
    for port_name in output_port_names {
        let passthrough_key = format!("{}{port_name}", PortName::OUTPUT_PASSTHROUGH_PREFIX);
        if let Some(value) = inputs.get(&passthrough_key) {
            outputs.insert(port_name.clone(), value.clone());
            continue;
        }
        outputs.entry(port_name.clone()).or_insert_with(|| {
            passthrough_fallback_value(port_name, &inputs).unwrap_or(Value::Skipped)
        });
    }
    Ok(outputs)
}

fn passthrough_fallback_value(port_name: &str, inputs: &HashMap<String, Value>) -> Option<Value> {
    let aliases: &[&str] = match port_name {
        "result" => &["input", "value", "content", "document"],
        "return" => &[
            "value",
            "content",
            "document",
            "input",
            "result",
            "directives",
            "sections",
            "lines",
            "text",
            "items",
        ],
        _ => &[],
    };

    let candidate = aliases
        .iter()
        .find_map(|alias| inputs.get(*alias).cloned())?;

    if port_name != "return" {
        return Some(candidate);
    }

    Some(match candidate {
        Value::Str(_) => candidate,
        Value::Int(value) => Value::Str(value.to_string()),
        Value::Bool(value) => Value::Str(value.to_string()),
        Value::Float(value) => Value::Str(value.to_string()),
        Value::Unit => Value::Str(String::new()),
        Value::List(values) | Value::Set(values) => Value::Str(
            values
                .iter()
                .map(passthrough_value_to_text)
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        Value::Map(values) => Value::Str(format!("{values:?}")),
        Value::Json(value) => Value::Str(value.to_string()),
        Value::Bytes(bytes) => Value::Str(format!("{bytes:?}")),
        Value::Secret(secret) => Value::Str(secret.to_string()),
        Value::Request(request) => Value::Str(format!("{request:?}")),
        Value::Response(response) => Value::Str(format!("{response:?}")),
        Value::Skipped => Value::Skipped,
    })
}

fn passthrough_value_to_text(value: &Value) -> String {
    match value {
        Value::Str(value) => value.clone(),
        Value::Int(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::Unit => String::new(),
        Value::Json(value) => value.to_string(),
        Value::Map(value) => format!("{value:?}"),
        Value::Bytes(value) => format!("{value:?}"),
        Value::Secret(value) => value.to_string(),
        Value::Request(value) => format!("{value:?}"),
        Value::Response(value) => format!("{value:?}"),
        Value::List(values) => format!("{values:?}"),
        Value::Set(values) => format!("{values:?}"),
        Value::Skipped => String::new(),
    }
}

/// Identity callable op for DSL-compiled callables with fn bodies.
///
/// Forwards all inputs to outputs, filling any declared output port that
/// has no matching input with `Value::Skipped`. This is the correct runtime
/// behavior for DSL `fn`/`func` items whose bodies execute as SubDag nodes —
/// the callable node itself is a passthrough that maps SubDag results to outputs.
#[derive(Debug, Clone)]
struct DeclaredOutputCallableOp {
    output_port_names: Vec<String>,
}

impl Executable for DeclaredOutputCallableOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        execute_with_declared_output_passthrough(&self.output_port_names, inputs)
    }
}

/// Pipeline dispatch op for resolved `LoweredOp::Pipeline` nodes.
///
/// When a pipeline is invoked as a node in another DAG, this op represents
/// the execution dispatch to the compiled pipeline's stages. The individual
/// stage bodies are lowered elsewhere; this op provides deterministic stage
/// ordering metadata and next-stage progression derived from the lowered
/// stage sequence.
#[derive(Clone)]
struct PipelineDispatchOp {
    _module: String,
    _name: String,
    stage_count: usize,
    stage_names: Vec<String>,
    output_port_names: Vec<String>,
}

impl std::fmt::Debug for PipelineDispatchOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineDispatchOp")
            .field("compat_mode", &"DeclaredOutputCallableOp")
            .field("stage_count", &self.stage_count)
            .field("stage_names", &self.stage_names)
            .field("output_port_names", &self.output_port_names)
            .finish()
    }
}

impl Executable for PipelineDispatchOp {
    /// Execute pipeline dispatch with explicit stage progression contract.
    ///
    /// **Stage progression contract**:
    /// - `stages`: total stage count (always emitted)
    /// - `stage_order`: ordered list of stage names (always emitted)
    /// - `active_stage`: defaults to the first stage (always emitted if stages exist)
    /// - If `current_stage` is provided:
    ///   - Must be a non-empty string matching a known stage name
    ///   - `next_stage` is set to the following stage, or to `current_stage` itself
    ///     if already at the last stage (terminal self-loop)
    /// - If `current_stage` is absent: no `next_stage` is emitted
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut outputs =
            execute_with_declared_output_passthrough(&self.output_port_names, inputs)?;
        outputs.insert("stages".to_string(), Value::Int(self.stage_count as i64));
        outputs.insert(
            "stage_order".to_string(),
            Value::str_list(self.stage_names.clone()),
        );
        if let Some(first) = self.stage_names.first() {
            outputs.insert("active_stage".to_string(), Value::Str(first.clone()));
        }
        if let Some(current_stage) = outputs.get("current_stage").and_then(Value::as_str) {
            // Fail closed on empty current_stage — this is a wiring bug.
            if current_stage.is_empty() {
                return Err(ExecError::new(
                    "pipeline dispatch received empty `current_stage` value — expected a valid stage name",
                ));
            }
            let Some(position) = self
                .stage_names
                .iter()
                .position(|stage| stage == current_stage)
            else {
                return Err(ExecError::new(format!(
                    "pipeline dispatch received unknown `current_stage` value `{current_stage}` \
                     (valid stages: {})",
                    self.stage_names.join(", ")
                )));
            };
            // Terminal stage: next_stage is self (no progression past the end).
            let next_stage = self
                .stage_names
                .get(position + 1)
                .cloned()
                .unwrap_or_else(|| current_stage.to_string());
            outputs.insert("next_stage".to_string(), Value::Str(next_stage));
        }
        Ok(outputs)
    }
}

/// Simple identity callable adapter for DSL entrypoint wrappers.
#[derive(Debug, Clone)]
struct IdentityCallableOp;

impl Executable for IdentityCallableOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        Ok(inputs)
    }
}

/// Source node for callable parameters referenced in service call arguments.
///
/// Receives the parameter value via boundary injection (auto_mock_spec or CLI
/// entrypoint setup), then outputs it on the named port. Falls back to
/// `Value::Skipped` if the value wasn't injected (e.g., inside nested SubDags
/// where boundary propagation doesn't reach).
#[derive(Debug, Clone)]
struct CallParamSourceOp {
    param: String,
    output_port: String,
}

impl Executable for CallParamSourceOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // Value arrives via set_input() from boundary injection.
        let value = inputs
            .get(&self.param)
            .or_else(|| inputs.values().next())
            .cloned()
            .unwrap_or(Value::Skipped);
        Ok(HashMap::from([(self.output_port.clone(), value)]))
    }
}

/// Test-only SubDag execution adapter.
///
/// **Production path**: `resolve_node_body` handles `NodeBody::SubDag` via
/// recursive `resolve_lowered_dag(inner)`, which preserves the SubDag structure
/// in the resolved `Dag<DynOp>`. The execution engine then handles SubDag
/// expansion at runtime.
///
/// **Test path**: `SubDagDispatchOp` flattens and executes the inner DAG
/// immediately, which is convenient for unit tests that need to verify SubDag
/// node behavior without a full engine setup.
///
/// This is intentionally `#[cfg(test)]`-gated because it is NOT the production
/// dispatch path. The production resolver preserves SubDag structure.
#[cfg(test)]
#[derive(Debug, Clone)]
struct SubDagDispatchOp {
    dag: Dag<DynOp>,
}

#[cfg(test)]
impl Executable for SubDagDispatchOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let entrypoints = gunbc_ir::detect_entrypoints(&self.dag);
        let mut input_mocks = gunbc_exec::BoundaryMocks::new();
        for (node_id, port_name, _) in entrypoints.entrypoint_ports {
            if let Some(value) = inputs.get(&port_name.0) {
                input_mocks.set_input(node_id.0, port_name.0, value.clone());
                continue;
            }
            if port_name.0 == PortName::DEPS {
                input_mocks.set_input(node_id.0, port_name.0, Value::List(Vec::new()));
            }
        }
        let execution = gunbc_test::boundary::execute_via_engine_with_inputs(
            &self.dag,
            gunbc_exec::ExecutionMode::Real,
            Some(&input_mocks),
        )?;
        let mut outputs = HashMap::new();
        for entry in execution.entries {
            for (key, value) in entry.outputs {
                if value != Value::Skipped {
                    outputs.insert(key, value);
                }
            }
        }
        Ok(outputs)
    }
}

/// Thin delegate to compiler's collection evaluator.
#[derive(Debug, Clone)]
struct CollectionDelegate {
    kind: CollectionOpKind,
}

impl Executable for CollectionDelegate {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let items = match inputs.get("items") {
            Some(Value::List(values)) => values.clone(),
            Some(Value::Skipped) | None => Vec::new(),
            Some(value) => vec![value.clone()],
        };
        let output = daglang_lower::eval::evaluate_collection(&self.kind, items, &inputs)
            .map_err(|e| ExecError::new(e.message))?;
        OutputMap::new().value("items", output).ok()
    }
}

/// Constant literal source adapter generated by lowering for literal call args.
#[derive(Debug, Clone)]
struct LiteralSourceOp {
    output_port: String,
    value: Value,
}

impl Executable for LiteralSourceOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        OutputMap::new()
            .value(self.output_port.as_str(), self.value.clone())
            .ok()
    }
}

/// Standard resource kinds from `dsl/std/resources.dag`.
///
/// Parsed once at resolution time (fail-fast). Unknown resource names
/// become `ResolveError` — no silent runtime fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceKind {
    Filesystem,
    Network,
    Clock,
    AuthContext,
}

impl ResourceKind {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "Filesystem" => Some(Self::Filesystem),
            "Network" => Some(Self::Network),
            "Clock" => Some(Self::Clock),
            "AuthContext" => Some(Self::AuthContext),
            _ => None,
        }
    }
}

/// Resource lifecycle acquire adapter for `std.resources`.
///
/// Produces a resource handle value appropriate for the resource kind.
/// In production, these will be real handle acquisitions; for now, they
/// produce cross-platform default handles for dry-run/test execution.
#[derive(Debug, Clone)]
struct ResourceAcquireOp {
    resource_kind: ResourceKind,
}

impl Executable for ResourceAcquireOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let handle: Value = match self.resource_kind {
            ResourceKind::Filesystem => {
                filename::FilesystemHandle::cross_platform(filename::Scope::Write).into()
            }
            ResourceKind::Network => Value::Str("network:default".to_string()),
            ResourceKind::Clock => Value::Str("clock:monotonic".to_string()),
            ResourceKind::AuthContext => Value::Str("auth:deferred".to_string()),
        };
        OutputMap::new().value("resource_handle", handle).ok()
    }
}

/// Resource lifecycle release adapter for `std.resources`.
///
/// No-op: releases resource handles (currently a passthrough).
#[derive(Debug, Clone)]
struct ResourceReleaseOp;

impl Executable for ResourceReleaseOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        Ok(inputs)
    }
}

/// Filesystem env adapter for DSL graphs.
///
/// Lowered DAGs currently use different fs output port names (`file:write`
/// and/or `FilesystemHandle`). Emit both for compatibility.
#[derive(Debug, Clone)]
struct DslFsEnvOp;

impl Executable for DslFsEnvOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let fs: Value = filename::FilesystemHandle::cross_platform(filename::Scope::Write).into();
        OutputMap::new()
            .value(FsEnv::WRITE_PORT, fs.clone())
            .value("FilesystemHandle", fs)
            .ok()
    }
}

/// File-read prepare adapter for DSL content-upsert chains.
///
/// Requires a `path` input. Missing `path` is a wiring bug and returns
/// an error so the issue surfaces at execution time rather than silently
/// producing a placeholder request.
#[derive(Debug, Clone)]
struct PrepareFileReadCompatOp;

impl Executable for PrepareFileReadCompatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if matches!(inputs.get("path"), Some(Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let path = inputs.get("path").and_then(Value::as_str).ok_or_else(|| {
            ExecError::new(
                "PrepareFileRead: missing required `path` input — check content-upsert wiring",
            )
        })?;
        OutputMap::new()
            .request("request", TransportRequest::File(FileRequest::read(path)))
            .bool("skip", false)
            .ok()
    }
}

/// File-write prepare adapter for DSL content-upsert chains.
///
/// Requires `path` and content inputs. Content is looked up under `content`,
/// `return`, or `expected_content` because the DSL lowering pipeline uses
/// different port names depending on the call path (callable return values
/// are named `return`, content-upsert compare nodes use `expected_content`,
/// and direct wiring uses `content`).
#[derive(Debug, Clone)]
struct PrepareFileWriteCompatOp;

impl Executable for PrepareFileWriteCompatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let input_keys = {
            let mut keys = inputs.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys.join(", ")
        };
        if matches!(inputs.get("path"), Some(Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let path_value = inputs
            .get("path")
            .or_else(|| inputs.get("target_path"))
            .or_else(|| inputs.get("filepath"));
        if matches!(path_value, Some(Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let path = path_value.and_then(Value::as_str).ok_or_else(|| {
            ExecError::new(
                format!(
                    "PrepareFileWrite: missing required `path` input — check content-upsert wiring (available inputs: {input_keys})"
                ),
            )
        })?;
        let content_value = inputs
            .get("content")
            .or_else(|| inputs.get("return"))
            .or_else(|| inputs.get("expected_content"))
            .or_else(|| inputs.get("makefile_content"));
        if matches!(content_value, Some(Value::Skipped)) {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let content = content_value
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExecError::new(format!(
                    "PrepareFileWrite: missing content input (expected `content`, `return`, or `expected_content`; available inputs: {input_keys})"
                ))
            })?;
        OutputMap::new()
            .request(
                "request",
                TransportRequest::File(FileRequest::write(path, content)),
            )
            .bool("skip", false)
            .ok()
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Resolve a lowered DAG into an executable `Dag<DynOp>`.
///
/// Each `LoweredOp` node is replaced with its concrete domain op wrapped
/// in `DynOp`. Edges and ports are preserved unchanged.
pub fn resolve_lowered_dag(dag: &Dag<LoweredOp>) -> Result<Dag<DynOp>, ResolveError> {
    let mut resolved = Dag::new();
    for node in &dag.nodes {
        let mut resolved_node = Node {
            id: node.id.clone(),
            inputs: node.inputs.clone(),
            outputs: node.outputs.clone(),
            body: resolve_node_body(node)?,
            examples: node.examples.clone(),
            log_detail: node.log_detail,
            kind: node.kind,
        };
        normalize_release_resource_inputs(&mut resolved_node);
        if let Some(mode) = needs_transport_resource(node, &resolved_node) {
            resolved_node
                .inputs
                .push(Port::resource(RESOURCE_FILE, "FilesystemHandle", mode));
        }
        resolved.add_node(resolved_node);
    }
    resolved.edges = dag.edges.clone();
    wire_missing_filesystem_resources(&mut resolved);
    Ok(resolved)
}

fn normalize_release_resource_inputs(node: &mut Node<DynOp>) {
    if !node.id.0.starts_with("release_resource_") {
        return;
    }
    for input in &mut node.inputs {
        if input.name.0 == "resource_handle" {
            input.cardinality = Cardinality::ZERO_OR_MORE;
        }
    }
}

// ============================================================================
// Node resolution
// ============================================================================

#[cfg(test)]
fn resolve_node(node: &Node<LoweredOp>) -> Result<DynOp, ResolveError> {
    let node_id = node.id.0.clone();
    match &node.body {
        NodeBody::Opaque(op) => resolve_op(&node_id, op, &node.outputs),
        NodeBody::SubDag(inner) => Ok(DynOp::new(SubDagDispatchOp {
            dag: resolve_lowered_dag(inner)?,
        })),
    }
}

fn resolve_node_body(node: &Node<LoweredOp>) -> Result<NodeBody<DynOp>, ResolveError> {
    match &node.body {
        NodeBody::Opaque(op) => Ok(NodeBody::Opaque(resolve_op(&node.id.0, op, &node.outputs)?)),
        NodeBody::SubDag(inner) => Ok(NodeBody::SubDag(resolve_lowered_dag(inner)?)),
    }
}

fn resolve_op(node_id: &str, op: &LoweredOp, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match op {
        LoweredOp::Collection { kind, .. } => resolve_collection(kind),
        LoweredOp::Pipeline {
            module,
            name,
            stages,
            stage_names,
        } => Ok(DynOp::new(PipelineDispatchOp {
            _module: module.clone(),
            _name: name.clone(),
            stage_count: *stages,
            stage_names: stage_names.clone(),
            output_port_names: declared_output_names(outputs),
        })),
        LoweredOp::Primitive { kind, .. } => resolve_primitive(kind, outputs),
        LoweredOp::Callable {
            module,
            name,
            kind,
            service_metadata,
            ..
        } => resolve_domain(
            node_id,
            module,
            name,
            *kind,
            outputs,
            service_metadata.as_deref(),
        ),
        LoweredOp::Pattern(pattern_op) => Ok(DynOp::new(pattern_op.clone())),
        LoweredOp::UnsupportedPattern { name } => Err(ResolveError {
            node_id: node_id.to_string(),
            reason: format!(
                "unsupported pattern `{name}` — not yet implemented in daglang lowering"
            ),
        }),
        LoweredOp::ExternCall { symbol } => resolve_extern_call(node_id, symbol),
    }
}

// ============================================================================
// Infrastructure resolution (cross-module patterns)
// ============================================================================

/// Resolve typed lowered primitive nodes shared across all modules.
fn resolve_primitive(kind: &PrimitiveOpKind, outputs: &[Port]) -> Result<DynOp, ResolveError> {
    match kind {
        PrimitiveOpKind::FsEnv => Ok(DynOp::new(DslFsEnvOp)),
        PrimitiveOpKind::CallParamSource { param, .. } => {
            let output_port = outputs
                .first()
                .map(|port| port.name.0.clone())
                .unwrap_or_else(|| param.clone());
            Ok(DynOp::new(CallParamSourceOp {
                param: param.clone(),
                output_port,
            }))
        }
        PrimitiveOpKind::CallLiteralSource { literal } => {
            let output_port = outputs
                .first()
                .map(|port| port.name.0.clone())
                .unwrap_or_else(|| "value".to_string());
            let value = match literal {
                PrimitiveLiteral::String(value) => Value::Str(value.clone()),
                PrimitiveLiteral::Int(value) => Value::Int(*value),
                PrimitiveLiteral::Bool(value) => Value::Bool(*value),
                PrimitiveLiteral::Json(value) => Value::from(value.clone()),
                PrimitiveLiteral::Unit => Value::Unit,
            };
            Ok(DynOp::new(LiteralSourceOp { output_port, value }))
        }
        PrimitiveOpKind::IoPrepareFileRead => Ok(DynOp::new(PrepareFileReadCompatOp)),
        PrimitiveOpKind::IoExecuteFileRead => Ok(DynOp::new(TransportOps::Execute)),
        PrimitiveOpKind::CompareEquality => Ok(DynOp::new(BlobOps::CompareContent)),
        PrimitiveOpKind::IoPrepareFileWrite => Ok(DynOp::new(PrepareFileWriteCompatOp)),
        PrimitiveOpKind::IoExecuteFileWrite => Ok(DynOp::new(TransportOps::Execute)),
        // FC-7: Output path annotation nodes are metadata-only, resolve as identity.
        PrimitiveOpKind::ContentUpsertOutputPath { .. } => Ok(DynOp::new(IdentityCallableOp)),
    }
}

// ============================================================================
// Domain resolution (per-module callables)
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn resolve_domain(
    node_id: &str,
    module: &str,
    name: &str,
    kind: CallableKind,
    outputs: &[Port],
    service_metadata: Option<&ServiceCallMetadata>,
) -> Result<DynOp, ResolveError> {
    // 1. Modules with custom resolvers — return Some for known callables,
    //    None for unknown (which falls through to passthrough).
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    let custom = match module {
        "tools.infra" => resolve_tools_infra(name),
        _ => None,
    };
    if let Some(op) = custom {
        return Ok(op);
    }
    // 2. Service/workspace modules use generic transport dispatch.
    if module.starts_with("services.") || module.starts_with("workspace.") {
        return resolve_service_transport(node_id, module, name, outputs, service_metadata);
    }
    // 3. Service transport nodes from non-service modules (e.g., loop body
    //    transport nodes which inherit the tool module name, not the service module).
    //    Only route when the metadata has a concrete operation spec; nodes without
    //    specs (e.g., not-yet-implemented service operations) fall through to the
    //    passthrough default.
    if let Some(transport_role) = TransportRole::from_name(name) {
        let has_spec = service_metadata.as_ref().is_some_and(|m| m.spec.is_some());
        if has_spec || transport_role == TransportRole::Execute {
            return resolve_service_transport(node_id, module, name, outputs, service_metadata);
        }
    }
    // 4. Extern impl lookup — DSL `extern func` items resolved to Rust ops.
    //
    // Shadow bridge detection: if an extern impl exists for a Fn/Func callable,
    // the Rust impl silently overrides whatever DSL body the callable has.
    // This is a documented workaround for a lowerer limitation (NF-7: same-module
    // extern func calls don't wire output ports correctly). Once NF-7 is resolved,
    // these callables should be converted to `extern func` declarations in DSL
    // and this shadow bridge path can be removed.
    let _ = &kind; // used in debug_assertions block below; suppress release warning
    if let Some(op) = crate::extern_impls::lookup_extern_impl(module, name) {
        #[cfg(debug_assertions)]
        if matches!(kind, CallableKind::Fn | CallableKind::Func) {
            eprintln!(
                "resolve: shadow bridge {module}::{name} (kind={kind:?}) — \
                 Rust extern impl overrides DSL callable body"
            );
        }
        return Ok(op);
    }
    // 5. Default: identity callable for compiler-validated callables.
    //
    // All LoweredOp::Callable nodes are produced by the DSL compiler (the
    // lowerer only emits Callable for items in the typed project). The
    // callable's logic is wired as separate nodes/edges in the DAG; this
    // wrapper node maps SubDag results to output ports via passthrough.
    //
    // ExternCall nodes (from `extern func` declarations) use
    // resolve_extern_call() which is fail-closed — no passthrough fallback.
    Ok(DynOp::new(DeclaredOutputCallableOp {
        output_port_names: declared_output_names(outputs),
    }))
}

fn resolve_std_resources(name: &str) -> Result<DynOp, ResolveError> {
    // Resource lifecycle acquire/release nodes from the DSL resource system.
    // Names follow the pattern: `resource_lifecycle::acquire::ResourceName`
    // or `resource_lifecycle::release::ResourceName`.
    if let Some(resource_name) = name.strip_prefix("resource_lifecycle::acquire::") {
        let kind = ResourceKind::parse(resource_name).ok_or_else(|| ResolveError {
            node_id: format!("resource_lifecycle::acquire::{resource_name}"),
            reason: format!(
                "unknown resource kind `{resource_name}` — \
                 expected one of: Filesystem, Network, Clock, AuthContext"
            ),
        })?;
        return Ok(DynOp::new(ResourceAcquireOp {
            resource_kind: kind,
        }));
    }
    if name.starts_with("resource_lifecycle::release::") {
        return Ok(DynOp::new(ResourceReleaseOp));
    }
    // Other std.resources callables pass through as identity.
    Ok(DynOp::new(IdentityCallableOp))
}

fn resolve_tools_infra(name: &str) -> Option<DynOp> {
    match name {
        "infra" => Some(DynOp::new(InfraDispatchOp)),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct InfraDispatchOp;

impl Executable for InfraDispatchOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let environment = inputs
            .get("environment")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExecError::new("missing required 'environment' input for tools.infra::infra")
            })?
            .to_string();
        let runtime = inputs
            .get("runtime")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExecError::new("missing required 'runtime' input for tools.infra::infra")
            })?
            .to_string();
        let spec_targets = inputs
            .get("spec_targets")
            .and_then(Value::as_str_list)
            .ok_or_else(|| {
                ExecError::new("missing required 'spec_targets' input for tools.infra::infra")
            })?;
        let target = inputs
            .get("target")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let skip = inputs
            .get("skip")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let execute = inputs
            .get("execute")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let targeted = if target.is_empty() {
            spec_targets.clone()
        } else {
            spec_targets
                .iter()
                .filter(|item| target.iter().any(|selected| selected == *item))
                .cloned()
                .collect::<Vec<_>>()
        };
        let planned_targets = targeted
            .into_iter()
            .filter(|item| !skip.iter().any(|excluded| excluded == item))
            .collect::<Vec<_>>();
        let target_count = planned_targets.len() as i64;
        let mode = if execute { "apply" } else { "plan" };
        let applied_count = if execute { target_count } else { 0 };
        let report = format!(
            "infra {mode} (env={environment}, runtime={runtime}): {target_count} target(s)"
        );

        OutputMap::new()
            .str("environment", environment)
            .str("runtime", runtime)
            .str("mode", mode)
            .str_list("planned_targets", planned_targets)
            .int("target_count", target_count)
            .int("applied_count", applied_count)
            .str("report", report)
            .ok()
    }
}

// ============================================================================
// Extern symbol resolution
// ============================================================================

/// Resolve an `ExternCall` node at compile time.
///
/// Parses the symbol into (module, name) and dispatches to the same
/// resolvers used by Callable nodes. Unlike Callable, unresolvable
/// extern symbols are hard errors — no passthrough fallback.
fn resolve_extern_call(node_id: &str, symbol: &str) -> Result<DynOp, ResolveError> {
    use gunbc_ir::ProgramSymbolId;

    let sym = ProgramSymbolId::new(symbol);
    let module = sym.module().unwrap_or("");
    let name = sym.name().unwrap_or("");

    if let Some(op) = crate::extern_impls::lookup_extern_impl(module, name) {
        return Ok(op);
    }
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    if let Some(op) = resolve_tools_infra(name) {
        if module == "tools.infra" {
            return Ok(op);
        }
    }

    Err(ResolveError {
        node_id: node_id.to_string(),
        reason: format!(
            "extern symbol `{symbol}` could not be resolved — \
             no extern impl, std.resources, or tools.infra handler found"
        ),
    })
}

// ============================================================================
// Service transport resolution
// ============================================================================

/// Structural classification of a service transport node's role.
///
/// Parsed once from the `service_transport::{role}::` name prefix, then
/// dispatched on. Eliminates repeated `starts_with()` string checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportRole {
    Prepare,
    Execute,
    Parse,
}

impl TransportRole {
    /// Parse the transport role from a node name.
    ///
    /// Returns `None` for names that don't start with `service_transport::`.
    fn from_name(name: &str) -> Option<Self> {
        if name.starts_with("service_transport::execute::") {
            Some(Self::Execute)
        } else if name.starts_with("service_transport::prepare::") {
            Some(Self::Prepare)
        } else if name.starts_with("service_transport::parse::") {
            Some(Self::Parse)
        } else {
            None
        }
    }
}

fn resolve_service_transport(
    node_id: &str,
    module: &str,
    name: &str,
    _outputs: &[Port],
    service_metadata: Option<&ServiceCallMetadata>,
) -> Result<DynOp, ResolveError> {
    let role = TransportRole::from_name(name);

    // Execute nodes: for InterfaceStub specs, use the stub execute op
    // (errors in Real mode, auto-mocked in DryRun). All others use the
    // standard transport executor.
    if role == Some(TransportRole::Execute) {
        if let Some(metadata) = service_metadata {
            if let Some(ServiceOperationSpec::InterfaceStub {
                interface,
                capability,
            }) = &metadata.spec
            {
                return Ok(DynOp::new(crate::resolve_service::InterfaceStubExecuteOp {
                    interface: interface.clone(),
                    capability: capability.clone(),
                }));
            }
        }
        return Ok(DynOp::new(TransportOps::Execute));
    }

    // Generic dispatch: use the spec from service_metadata to select interpreter.
    if let Some(metadata) = service_metadata {
        if let Some(spec) = &metadata.spec {
            match (spec, role) {
                (ServiceOperationSpec::Rest(rest_spec), Some(TransportRole::Prepare)) => {
                    return Ok(DynOp::new(GenericRestPrepareOp {
                        spec: rest_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::Rest(rest_spec), Some(TransportRole::Parse)) => {
                    return Ok(DynOp::new(GenericRestParseOp {
                        spec: rest_spec.clone(),
                        service_name: metadata.service.clone(),
                        operation_name: metadata.operation.clone(),
                        auth_scheme: rest_spec.auth_scheme.clone().unwrap_or_default(),
                        permissions: metadata.permissions.clone(),
                    }));
                }
                (ServiceOperationSpec::Shell(shell_spec), Some(TransportRole::Prepare)) => {
                    return Ok(DynOp::new(GenericShellPrepareOp {
                        spec: shell_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::Shell(shell_spec), Some(TransportRole::Parse)) => {
                    return Ok(DynOp::new(GenericShellParseOp {
                        spec: shell_spec.clone(),
                        service_name: metadata.service.clone(),
                        operation_name: metadata.operation.clone(),
                    }));
                }
                (ServiceOperationSpec::File(file_spec), Some(TransportRole::Prepare)) => {
                    return Ok(DynOp::new(GenericFilePrepareOp {
                        spec: file_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::File(file_spec), Some(TransportRole::Parse)) => {
                    return Ok(DynOp::new(GenericFileParseOp {
                        spec: file_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::Local(local_spec), Some(TransportRole::Prepare)) => {
                    return Ok(DynOp::new(GenericLocalPrepareOp {
                        spec: local_spec.clone(),
                    }));
                }
                (ServiceOperationSpec::Local(local_spec), Some(TransportRole::Parse)) => {
                    return Ok(DynOp::new(GenericLocalParseOp {
                        spec: local_spec.clone(),
                    }));
                }
                // IS-6: InterfaceStub ops for interface capabilities without profile.
                (
                    ServiceOperationSpec::InterfaceStub {
                        interface,
                        capability,
                    },
                    Some(TransportRole::Prepare),
                ) => {
                    return Ok(DynOp::new(crate::resolve_service::InterfaceStubPrepareOp {
                        interface: interface.clone(),
                        capability: capability.clone(),
                    }));
                }
                (
                    ServiceOperationSpec::InterfaceStub {
                        interface,
                        capability,
                    },
                    Some(TransportRole::Parse),
                ) => {
                    return Ok(DynOp::new(crate::resolve_service::InterfaceStubParseOp {
                        interface: interface.clone(),
                        capability: capability.clone(),
                    }));
                }
                _ => {}
            }
        }
        // Fail-closed: metadata present but no matching operation spec.
        // This indicates a resolver gap (missing service operation handler)
        // rather than a valid passthrough scenario.
        return Err(ResolveError {
            node_id: node_id.to_string(),
            reason: format!(
                "service transport node has metadata but no matching operation spec: module={module} name={name}"
            ),
        });
    }

    Err(unknown_callable(node_id, module, name))
}

// ============================================================================
// Collection resolution
// ============================================================================

fn resolve_collection(kind: &CollectionOpKind) -> Result<DynOp, ResolveError> {
    Ok(DynOp::new(CollectionDelegate { kind: *kind }))
}

// ============================================================================
// Helpers
// ============================================================================

fn unknown_callable(node_id: &str, module: &str, name: &str) -> ResolveError {
    ResolveError {
        node_id: node_id.to_string(),
        reason: format!("unknown callable `{module}.{name}`"),
    }
}

/// Check if a transport execute node needs a filesystem resource input added.
///
/// Returns `Some(AccessMode)` if the node is a transport execute node
/// (content_upsert or service_transport) that doesn't already have a
/// filesystem resource input. The resource system requires all transport
/// execute nodes to declare their resource access.
fn needs_transport_resource(
    lowered: &Node<LoweredOp>,
    resolved: &Node<DynOp>,
) -> Option<AccessMode> {
    let mode = match &lowered.body {
        NodeBody::Opaque(LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileWrite,
            ..
        }) => AccessMode::Write,
        NodeBody::Opaque(LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileRead,
            ..
        }) => AccessMode::Read,
        _ if lowered.kind == gunbc_ir::NodeKind::TransportExecute => {
            // Service transport execute nodes need filesystem access.
            AccessMode::Read
        }
        _ => return None,
    };

    // Only add if not already present.
    let already_has = resolved.inputs.iter().any(|port| {
        port.type_id.0 == "FilesystemHandle"
            && (port.name.0 == RESOURCE_FILE || port.name.0.starts_with(RESOURCE_FILE_PREFIX))
    });
    if already_has {
        None
    } else {
        Some(mode)
    }
}

fn wire_missing_filesystem_resources(dag: &mut Dag<DynOp>) {
    let mut pending = Vec::new();
    for node in &dag.nodes {
        for port in &node.inputs {
            let is_filesystem_resource_port = port.type_id.0 == "FilesystemHandle"
                && (port.name.0 == RESOURCE_FILE || port.name.0.starts_with(RESOURCE_FILE_PREFIX));
            if !is_filesystem_resource_port {
                continue;
            }
            let connected = dag
                .edges
                .iter()
                .any(|edge| edge.to_node == node.id && edge.to_port == port.name);
            if !connected {
                pending.push((node.id.0.clone(), port.name.0.clone()));
            }
        }
    }
    if pending.is_empty() {
        return;
    }

    let fs_node_id = "fs_env".to_string();
    let fs_output_port = if let Some(existing) = dag.get_node(&fs_node_id.clone().into()) {
        existing
            .outputs
            .iter()
            .find(|port| port.type_id.0 == "FilesystemHandle")
            .map(|port| port.name.0.clone())
            .unwrap_or_else(|| "FilesystemHandle".to_string())
    } else {
        dag.add_node(
            Node::opaque(
                fs_node_id.as_str(),
                vec![],
                vec![Port::new("FilesystemHandle", "FilesystemHandle")],
                DynOp::new(DslFsEnvOp),
            )
            .with_kind(gunbc_ir::NodeKind::ResourceEnvironment),
        );
        "FilesystemHandle".to_string()
    };

    for (node_id, port_name) in pending {
        let already_connected = dag.edges.iter().any(|edge| {
            edge.from_node.0 == fs_node_id
                && edge.from_port.0 == fs_output_port
                && edge.to_node.0 == node_id
                && edge.to_port.0 == port_name
        });
        if !already_connected {
            dag.add_edge(Edge::new(
                fs_node_id.clone(),
                fs_output_port.clone(),
                node_id,
                port_name,
            ));
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{CallableKind, ObligationCategory, PrimitiveLiteral, PrimitiveOpKind};
    use gunbc_ir::{Node, Port};

    fn callable_node(
        id: &str,
        module: &str,
        name: &str,
        obligation: ObligationCategory,
    ) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Callable {
                module: module.to_string(),
                kind: CallableKind::Fn,
                name: name.to_string(),
                obligation,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
    }

    fn collection_node(id: &str, kind: CollectionOpKind) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![Port::new("items", "String")],
            vec![Port::new("items", "String")],
            LoweredOp::Collection {
                module: "test".to_string(),
                callable: "test_fn".to_string(),
                kind,
            },
        )
    }

    fn primitive_node(
        id: &str,
        module: &str,
        name: &str,
        kind: PrimitiveOpKind,
    ) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Primitive {
                module: module.to_string(),
                name: name.to_string(),
                kind,
            },
        )
    }

    #[test]
    fn resolve_pragma_render_ops_emit_content() {
        let cases = [
            "render_clippy_toml",
            "render_disallowed_methods_allowlist",
            "render_pragma_lint_policy",
        ];
        for name in cases {
            let node = callable_node(name, "tools.pragma", name, ObligationCategory::None);
            let result = resolve_node(&node).expect(name);
            let outputs = result.execute(HashMap::new()).expect(name);
            assert!(
                outputs
                    .get("return")
                    .and_then(Value::as_str)
                    .map(|content| !content.is_empty())
                    .unwrap_or(false),
                "resolver should execute pragma renderer `{name}` and emit non-empty return"
            );
        }
    }

    #[test]
    fn resolve_bootstrap_render_ops_emit_content() {
        let cases = ["render_bootstrap_makefile", "render_bootstrap_gitignore"];
        for name in cases {
            let node = callable_node(name, "tools.bootstrap", name, ObligationCategory::None);
            let result = resolve_node(&node).expect(name);
            let outputs = result.execute(HashMap::new()).expect(name);
            assert!(
                outputs
                    .get("return")
                    .and_then(Value::as_str)
                    .map(|content| !content.is_empty())
                    .unwrap_or(false),
                "resolver should execute bootstrap renderer `{name}` and emit non-empty return"
            );
        }
    }

    #[test]
    fn resolve_tools_infra_entrypoint_emits_plan_summary() {
        let node = callable_node("infra", "tools.infra", "infra", ObligationCategory::None);
        let result = resolve_node(&node).expect("tools.infra::infra");
        let mut inputs = HashMap::new();
        inputs.insert("environment".to_string(), Value::Str("dev".to_string()));
        inputs.insert("runtime".to_string(), Value::Str("local".to_string()));
        inputs.insert(
            "spec_targets".to_string(),
            Value::str_list(vec!["secret:a".to_string(), "secret:b".to_string()]),
        );
        inputs.insert(
            "target".to_string(),
            Value::str_list(vec!["secret:b".to_string()]),
        );
        inputs.insert("skip".to_string(), Value::str_list(Vec::<String>::new()));
        inputs.insert("execute".to_string(), Value::Bool(false));
        let outputs = result.execute(inputs).expect("infra op should execute");
        assert_eq!(
            outputs.get("planned_targets"),
            Some(&Value::str_list(vec!["secret:b".to_string()]))
        );
        assert_eq!(outputs.get("target_count"), Some(&Value::Int(1)));
        assert_eq!(outputs.get("applied_count"), Some(&Value::Int(0)));
    }

    #[test]
    fn resolve_pipeline_dispatch_reports_stage_count() {
        let node = Node::opaque(
            "pipeline_sdlc",
            vec![],
            vec![Port::new("stages", "Int")],
            LoweredOp::Pipeline {
                module: "pipelines.sdlc".to_string(),
                name: "sdlc".to_string(),
                stages: 8,
                stage_names: vec![
                    "fetch".to_string(),
                    "claim_design".to_string(),
                    "design".to_string(),
                    "design_review".to_string(),
                    "record_design_outcome".to_string(),
                    "accept_design".to_string(),
                    "implementation".to_string(),
                    "close".to_string(),
                ],
            },
        );
        let op = resolve_node(&node).expect("pipeline node should resolve");
        let mut inputs = HashMap::new();
        inputs.insert(
            "current_stage".to_string(),
            Value::Str("design_review".to_string()),
        );
        let outputs = op
            .execute(inputs)
            .expect("pipeline dispatch should execute");
        assert_eq!(outputs.get("stages"), Some(&Value::Int(8)));
        assert_eq!(
            outputs.get("active_stage"),
            Some(&Value::Str("fetch".to_string()))
        );
        assert_eq!(
            outputs.get("next_stage"),
            Some(&Value::Str("record_design_outcome".to_string()))
        );
        assert_eq!(
            outputs.get("stage_order"),
            Some(&Value::str_list(vec![
                "fetch".to_string(),
                "claim_design".to_string(),
                "design".to_string(),
                "design_review".to_string(),
                "record_design_outcome".to_string(),
                "accept_design".to_string(),
                "implementation".to_string(),
                "close".to_string(),
            ]))
        );
    }

    #[test]
    fn resolve_pipeline_dispatch_fails_closed_for_unknown_stage() {
        let node = Node::opaque(
            "pipeline_sdlc",
            vec![],
            vec![Port::new("stages", "Int")],
            LoweredOp::Pipeline {
                module: "pipelines.sdlc".to_string(),
                name: "sdlc".to_string(),
                stages: 2,
                stage_names: vec!["fetch".to_string(), "design".to_string()],
            },
        );
        let op = resolve_node(&node).expect("pipeline node should resolve");
        let mut inputs = HashMap::new();
        inputs.insert(
            "current_stage".to_string(),
            Value::Str("unknown-stage".to_string()),
        );
        let error = op
            .execute(inputs)
            .expect_err("unknown stage should fail closed");
        assert!(
            error
                .to_string()
                .contains("pipeline dispatch received unknown `current_stage`"),
            "unexpected error: {error}"
        );
        // FC-6: error message should include valid stage names for diagnostics
        assert!(
            error.to_string().contains("fetch") && error.to_string().contains("design"),
            "error should list valid stages: {error}"
        );
    }

    #[test]
    fn resolve_pipeline_dispatch_fails_closed_for_empty_stage() {
        let node = Node::opaque(
            "pipeline_sdlc",
            vec![],
            vec![Port::new("stages", "Int")],
            LoweredOp::Pipeline {
                module: "pipelines.sdlc".to_string(),
                name: "sdlc".to_string(),
                stages: 2,
                stage_names: vec!["fetch".to_string(), "design".to_string()],
            },
        );
        let op = resolve_node(&node).expect("pipeline node should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("current_stage".to_string(), Value::Str(String::new()));
        let error = op
            .execute(inputs)
            .expect_err("empty current_stage should fail closed");
        assert!(
            error.to_string().contains("empty `current_stage`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn resolve_pipeline_dispatch_last_stage_loops_to_self() {
        let node = Node::opaque(
            "pipeline_sdlc",
            vec![],
            vec![Port::new("stages", "Int")],
            LoweredOp::Pipeline {
                module: "pipelines.sdlc".to_string(),
                name: "sdlc".to_string(),
                stages: 2,
                stage_names: vec!["fetch".to_string(), "design".to_string()],
            },
        );
        let op = resolve_node(&node).expect("pipeline node should resolve");
        let mut inputs = HashMap::new();
        inputs.insert(
            "current_stage".to_string(),
            Value::Str("design".to_string()),
        );
        let outputs = op.execute(inputs).expect("last stage should not error");
        assert_eq!(
            outputs.get("next_stage"),
            Some(&Value::Str("design".to_string())),
            "last stage should loop to self as terminal"
        );
    }

    #[test]
    fn resolve_subdag_node_executes_inner_graph() {
        let mut inner = Dag::new();
        inner.add_node(Node::opaque(
            "inner_literal",
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "literal".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::String("ok".to_string()),
                },
            },
        ));
        let node = Node::subdag("wrapper", inner);
        let op = resolve_node(&node).expect("subdag node should resolve");
        let outputs = op
            .execute(HashMap::new())
            .expect("resolved subdag should execute inner graph");
        assert_eq!(outputs.get("out"), Some(&Value::Str("ok".to_string())));
    }

    /// Build a service transport node with metadata and spec for generic dispatch.
    fn service_callable_node(
        id: &str,
        module: &str,
        name: &str,
        obligation: ObligationCategory,
        metadata: ServiceCallMetadata,
    ) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Callable {
                module: module.to_string(),
                kind: CallableKind::Fn,
                name: name.to_string(),
                obligation,
                service_metadata: Some(Box::new(metadata)),
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
    }

    fn codegen_check_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "shell.Codegen".to_string(),
            operation: "Check".to_string(),
            transport: ServiceTransportClass::ShellLocal,
            idempotent: false,
            readonly: true,
            permissions: vec![],
            spec: Some(ServiceOperationSpec::Shell(ShellOperationSpec {
                argv_template: vec![
                    ArgvSegment::Literal("test".to_string()),
                    ArgvSegment::Literal("-f".to_string()),
                    ArgvSegment::Literal("target/codegen/.stamp".to_string()),
                ],
                input_fields: vec![],
                output_fields: vec![OutputFieldSpec {
                    name: "needed".to_string(),
                    type_id: "Bool".to_string(),
                    json_path: "needed".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                }],
                output_parsing: ShellOutputParsing::ExitCodeBool,
                env: vec![],
            })),

        }
    }

    fn codegen_run_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "shell.Codegen".to_string(),
            operation: "Run".to_string(),
            transport: ServiceTransportClass::ShellLocal,
            idempotent: false,
            readonly: false,
            permissions: vec![],
            spec: Some(ServiceOperationSpec::Shell(ShellOperationSpec {
                argv_template: vec![
                    ArgvSegment::Literal("cargo".to_string()),
                    ArgvSegment::Literal("run".to_string()),
                    ArgvSegment::Literal("-p".to_string()),
                    ArgvSegment::Literal("gunbc-dag".to_string()),
                    ArgvSegment::Literal("--bin".to_string()),
                    ArgvSegment::Literal("gunbc-codegen".to_string()),
                    ArgvSegment::Literal("--".to_string()),
                    ArgvSegment::Literal("codegen".to_string()),
                ],
                input_fields: vec![],
                output_fields: vec![
                    OutputFieldSpec {
                        name: "success".to_string(),
                        type_id: "Bool".to_string(),
                        json_path: "success".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                    },
                    OutputFieldSpec {
                        name: "stdout".to_string(),
                        type_id: "String".to_string(),
                        json_path: "stdout".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                    },
                    OutputFieldSpec {
                        name: "stderr".to_string(),
                        type_id: "String".to_string(),
                        json_path: "stderr".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                    },
                ],
                output_parsing: ShellOutputParsing::SuccessStdoutStderr,
                env: vec![],
            })),

        }
    }

    #[test]
    fn resolve_services_shell_codegen_transport_ops() {
        // Prepare nodes use generic shell prepare/parse with spec from metadata.
        let cases = [
            (
                "service_transport::prepare::shell.Codegen::Check",
                codegen_check_metadata(),
                "GenericShellPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Check",
                codegen_check_metadata(),
                "GenericShellParseOp",
            ),
            (
                "service_transport::prepare::shell.Codegen::Run",
                codegen_run_metadata(),
                "GenericShellPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Run",
                codegen_run_metadata(),
                "GenericShellParseOp",
            ),
        ];

        for (name, metadata, expected_debug) in cases {
            let node = service_callable_node(
                name,
                "services.shell",
                name,
                ObligationCategory::None,
                metadata,
            );
            let result = resolve_node(&node).expect(name);
            assert!(
                format!("{:?}", result).contains(expected_debug),
                "expected {expected_debug} for {name}, got {:?}",
                result
            );
        }
    }

    #[test]
    fn resolve_tools_codegen_entrypoint_uses_passthrough() {
        let node = callable_node(
            "codegen",
            "tools.codegen",
            "codegen",
            ObligationCategory::None,
        );
        let result =
            resolve_node(&node).expect("tools.codegen::codegen should resolve via passthrough");
        let debug = format!("{result:?}");
        assert!(
            debug.contains("DeclaredOutputCallableOp"),
            "should use DeclaredOutputCallableOp: {debug}"
        );
    }

    #[test]
    fn resolve_fs_env() {
        let node = primitive_node("fs_env", "tools.makegen", "fs_env", PrimitiveOpKind::FsEnv);
        let result = resolve_node(&node).expect("fs_env");
        assert!(format!("{:?}", result).contains("FsEnv"));
    }

    #[test]
    fn resolve_content_upsert_prepare_read() {
        let node = primitive_node(
            "prepare_read_clippy",
            "tools.pragma",
            "content_upsert::prepare_read_clippy",
            PrimitiveOpKind::IoPrepareFileRead,
        );
        let result = resolve_node(&node).expect("prepare_read");
        assert!(format!("{:?}", result).contains("PrepareFileRead"));
    }

    #[test]
    fn resolve_content_upsert_execute_read() {
        let node = primitive_node(
            "execute_read_clippy",
            "tools.pragma",
            "content_upsert::execute_read_clippy",
            PrimitiveOpKind::IoExecuteFileRead,
        );
        let result = resolve_node(&node).expect("execute_read");
        assert!(format!("{:?}", result).contains("Execute"));
    }

    #[test]
    fn resolve_content_upsert_compare() {
        let node = primitive_node(
            "compare_clippy_content",
            "tools.pragma",
            "content_upsert::compare_clippy_content",
            PrimitiveOpKind::CompareEquality,
        );
        let result = resolve_node(&node).expect("compare");
        assert!(format!("{:?}", result).contains("CompareContent"));
    }

    #[test]
    fn resolve_content_upsert_prepare_write() {
        let node = primitive_node(
            "prepare_write_clippy",
            "tools.pragma",
            "content_upsert::prepare_write_clippy",
            PrimitiveOpKind::IoPrepareFileWrite,
        );
        let result = resolve_node(&node).expect("prepare_write");
        assert!(format!("{:?}", result).contains("PrepareFileWrite"));
    }

    #[test]
    fn resolve_content_upsert_execute_transport() {
        let node = primitive_node(
            "execute_clippy_transport",
            "tools.pragma",
            "content_upsert::execute_clippy_transport",
            PrimitiveOpKind::IoExecuteFileWrite,
        );
        let result = resolve_node(&node).expect("execute_transport");
        assert!(format!("{:?}", result).contains("Execute"));
    }

    #[test]
    fn resolve_literal_source_op_emits_constant_output() {
        let node = Node::opaque(
            "literal_path",
            vec![],
            vec![Port::new("path", "String")],
            LoweredOp::Primitive {
                module: "tools.bootstrap".to_string(),
                name: "call_literal_source::strhex:637261746573".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::String("crates".to_string()),
                },
            },
        );
        let result = resolve_node(&node).expect("literal source should resolve");
        let outputs = result
            .execute(HashMap::new())
            .expect("literal source executes");
        assert_eq!(
            outputs.get("path").and_then(Value::as_str),
            Some("crates"),
            "literal source should decode and emit the string constant"
        );
    }

    #[test]
    fn resolve_param_source_op_passthroughs_input() {
        let node = Node::opaque(
            "param_source_path",
            vec![Port::new("path", "String")],
            vec![Port::new("path", "String")],
            LoweredOp::Primitive {
                module: "tools.makegen".to_string(),
                name: "call_param_source::makegen::path".to_string(),
                kind: PrimitiveOpKind::CallParamSource {
                    callable: "makegen".to_string(),
                    param: "path".to_string(),
                },
            },
        );
        let result = resolve_node(&node).expect("param source should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("tmp/out.mk".to_string()));
        let outputs = result.execute(inputs).expect("param source should execute");
        assert_eq!(
            outputs.get("path").and_then(Value::as_str),
            Some("tmp/out.mk"),
            "param source should pass through input port value"
        );
    }

    fn sts_exchange_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "gcp.STS".to_string(),
            operation: "Exchange".to_string(),
            transport: ServiceTransportClass::RestNetwork,
            idempotent: true,
            readonly: false,
            permissions: vec![],
            spec: Some(ServiceOperationSpec::Rest(RestOperationSpec {
                endpoint: "https://sts.googleapis.com".to_string(),
                method: "POST".to_string(),
                path_template: "/v1/token".to_string(),
                input_fields: vec![
                    FieldSpec {
                        name: "subject_token".to_string(),
                        type_id: "Secret".to_string(),
                        default: None,
                        is_secret: true,
                        is_path_param: false,
                    },
                    FieldSpec {
                        name: "audience".to_string(),
                        type_id: "NonEmptyStr".to_string(),
                        default: None,
                        is_secret: false,
                        is_path_param: false,
                    },
                ],
                output_fields: vec![
                    OutputFieldSpec {
                        name: "access_token".to_string(),
                        type_id: "Secret".to_string(),
                        json_path: "access_token".to_string(),
                        is_secret: true,
                        is_raw_body: false,
                    },
                    OutputFieldSpec {
                        name: "expires_in".to_string(),
                        type_id: "Int".to_string(),
                        json_path: "expires_in".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                    },
                ],
                body_template: None,
                headers: vec![],
                auth_scheme: None,
                auth_input: None,

            })),

        }
    }

    fn secret_manager_metadata() -> ServiceCallMetadata {
        use daglang_lower::*;
        ServiceCallMetadata {
            service: "gcp.SecretManager".to_string(),
            operation: "AccessVersion".to_string(),
            transport: ServiceTransportClass::RestNetwork,
            idempotent: true,
            readonly: true,
            permissions: vec!["secretmanager.versions.access".to_string()],
            spec: Some(ServiceOperationSpec::Rest(RestOperationSpec {
                endpoint: "https://secretmanager.googleapis.com".to_string(),
                method: "GET".to_string(),
                path_template: "/v1/projects/{project}/secrets/{secret}/versions/{version}:access"
                    .to_string(),
                input_fields: vec![
                    FieldSpec {
                        name: "project".to_string(),
                        type_id: "ProjectId".to_string(),
                        default: None,
                        is_secret: false,
                        is_path_param: true,
                    },
                    FieldSpec {
                        name: "secret".to_string(),
                        type_id: "NonEmptyStr".to_string(),
                        default: None,
                        is_secret: false,
                        is_path_param: true,
                    },
                    FieldSpec {
                        name: "version".to_string(),
                        type_id: "NonEmptyStr".to_string(),
                        default: Some("latest".to_string()),
                        is_secret: false,
                        is_path_param: true,
                    },
                ],
                output_fields: vec![
                    OutputFieldSpec {
                        name: "payload".to_string(),
                        type_id: "Bytes".to_string(),
                        json_path: "payload".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                    },
                    OutputFieldSpec {
                        name: "name".to_string(),
                        type_id: "String".to_string(),
                        json_path: "name".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                    },
                ],
                body_template: None,
                headers: vec![],
                auth_scheme: None,
                auth_input: None,

            })),

        }
    }

    #[test]
    fn resolve_services_gcp_transport_ops() {
        let cases = [
            (
                "services.gcp.sts",
                "service_transport::prepare::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericRestPrepareOp",
            ),
            (
                "services.gcp.sts",
                "service_transport::parse::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericRestParseOp",
            ),
            (
                "services.gcp.secret_manager",
                "service_transport::prepare::gcp.SecretManager::AccessVersion",
                secret_manager_metadata(),
                "GenericRestPrepareOp",
            ),
            (
                "services.gcp.secret_manager",
                "service_transport::parse::gcp.SecretManager::AccessVersion",
                secret_manager_metadata(),
                "GenericRestParseOp",
            ),
        ];
        for (module, name, metadata, expected_debug) in cases {
            let node =
                service_callable_node(name, module, name, ObligationCategory::None, metadata);
            let result = resolve_node(&node).expect(name);
            assert!(
                format!("{:?}", result).contains(expected_debug),
                "expected {expected_debug} for {name}, got {:?}",
                result
            );
        }
    }

    #[test]
    fn resolve_collection_map() {
        let node = collection_node("map_items", CollectionOpKind::Map);
        let result = resolve_node(&node).expect("map");
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ]),
        );
        let outputs = result
            .execute(inputs)
            .expect("collection map should execute");
        assert_eq!(
            outputs.get("items"),
            Some(&Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string())
            ]))
        );
    }

    #[test]
    fn resolve_collection_len() {
        let node = collection_node("len_items", CollectionOpKind::Len);
        let result = resolve_node(&node).expect("len");
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        );
        let outputs = result
            .execute(inputs)
            .expect("collection len should execute");
        assert_eq!(outputs.get("items"), Some(&Value::Int(3)));
    }

    #[test]
    fn resolve_collection_contains() {
        let node = collection_node("contains_items", CollectionOpKind::Contains);
        let result = resolve_node(&node).expect("contains");
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ]),
        );
        inputs.insert("needle".to_string(), Value::Str("b".to_string()));
        let outputs = result
            .execute(inputs)
            .expect("collection contains should execute");
        assert_eq!(outputs.get("items"), Some(&Value::Bool(true)));
    }

    #[test]
    fn resolve_unknown_module_uses_passthrough() {
        let node = callable_node(
            "unknown_op",
            "tools.unknown",
            "do_something",
            ObligationCategory::None,
        );
        let result = resolve_node(&node).expect("unknown modules should resolve via passthrough");
        let debug = format!("{result:?}");
        assert!(
            debug.contains("DeclaredOutputCallableOp"),
            "should use DeclaredOutputCallableOp: {debug}"
        );
    }

    #[test]
    fn resolve_unknown_callable_in_custom_module_uses_passthrough() {
        let node = callable_node(
            "bad_op",
            "tools.pragma",
            "nonexistent_op",
            ObligationCategory::None,
        );
        let result = resolve_node(&node).expect("unknown callable should resolve via passthrough");
        let debug = format!("{result:?}");
        assert!(
            debug.contains("DeclaredOutputCallableOp"),
            "should use DeclaredOutputCallableOp: {debug}"
        );
    }

    #[test]
    fn resolve_infra_callable_maps_to_infra_dispatch_op() {
        let node = callable_node("infra", "tools.infra", "infra", ObligationCategory::None);
        let result = resolve_node(&node).expect("infra");
        let mut inputs = HashMap::new();
        inputs.insert("environment".to_string(), Value::Str("dev".to_string()));
        inputs.insert("runtime".to_string(), Value::Str("local".to_string()));
        inputs.insert(
            "spec_targets".to_string(),
            Value::str_list(vec!["secret:github-token".to_string()]),
        );
        inputs.insert("target".to_string(), Value::str_list(Vec::<String>::new()));
        inputs.insert("skip".to_string(), Value::str_list(Vec::<String>::new()));
        inputs.insert("execute".to_string(), Value::Bool(false));
        let outputs = result.execute(inputs).expect("infra op should execute");
        assert_eq!(outputs.get("mode"), Some(&Value::Str("plan".to_string())));
        assert_eq!(outputs.get("target_count"), Some(&Value::Int(1)));
    }

    #[test]
    fn resolve_unknown_service_transport_prepare_fails() {
        let node = callable_node(
            "bad_service_prepare",
            "services.gcp.sts",
            "service_transport::prepare::gcp.STS::Refresh",
            ObligationCategory::ServiceTransportPrepare,
        );
        let err = resolve_node(&node).unwrap_err();
        assert!(err.reason.contains("unknown callable"));
    }

    #[test]
    fn resolve_full_dag_preserves_edges() {
        let mut dag = Dag::new();
        dag.add_node(callable_node(
            "render",
            "tools.pragma",
            "render_clippy_toml",
            ObligationCategory::None,
        ));
        dag.add_node(primitive_node(
            "prepare_read",
            "tools.pragma",
            "content_upsert::prepare_read_clippy",
            PrimitiveOpKind::IoPrepareFileRead,
        ));
        dag.edges.push(gunbc_ir::Edge {
            from_node: "render".into(),
            from_port: "content".into(),
            to_node: "prepare_read".into(),
            to_port: "content".into(),
            index: 0,
            kind: gunbc_ir::EdgeKind::DataFlow,
        });

        let resolved = resolve_lowered_dag(&dag).expect("resolve dag");
        assert_eq!(resolved.nodes.len(), 2);
        assert_eq!(resolved.edges.len(), 1);
        assert_eq!(resolved.edges[0].from_node.0, "render");
        assert_eq!(resolved.edges[0].to_node.0, "prepare_read");
    }

    #[test]
    fn needs_transport_resource_respects_existing_res_file_port() {
        let lowered = primitive_node(
            "execute_read_makegen",
            "tools.makegen",
            "content_upsert::execute_read_makegen",
            PrimitiveOpKind::IoExecuteFileRead,
        );
        let resolved = Node::opaque(
            "execute_read_makegen",
            vec![Port::resource("file", "FilesystemHandle", AccessMode::Read)],
            vec![Port::new("response", "TransportResponse")],
            DynOp::new(DslFsEnvOp),
        );

        let mode = needs_transport_resource(&lowered, &resolved);
        assert!(
            mode.is_none(),
            "existing `res:file` input should prevent duplicate resource port injection"
        );
    }

    #[test]
    fn wire_missing_filesystem_resources_wires_res_file_port() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "fs_env",
            vec![],
            vec![Port::new("FilesystemHandle", "FilesystemHandle")],
            DynOp::new(DslFsEnvOp),
        ));
        dag.add_node(Node::opaque(
            "execute_read_makegen",
            vec![Port::resource("file", "FilesystemHandle", AccessMode::Read)],
            vec![Port::new("response", "TransportResponse")],
            DynOp::new(DslFsEnvOp),
        ));

        wire_missing_filesystem_resources(&mut dag);

        let has_edge = dag.edges.iter().any(|edge| {
            edge.from_node.0 == "fs_env"
                && edge.from_port.0 == "FilesystemHandle"
                && edge.to_node.0 == "execute_read_makegen"
                && edge.to_port.0 == "res:file"
        });
        assert!(
            has_edge,
            "filesystem resource edge should be auto-wired for `res:file` inputs"
        );
    }

    #[test]
    fn prepare_file_write_propagates_skipped_content_to_request() {
        let mut inputs = HashMap::new();
        inputs.insert("path".to_string(), Value::Str("foo.txt".to_string()));
        inputs.insert("content".to_string(), Value::Skipped);

        let outputs = PrepareFileWriteCompatOp
            .execute(inputs)
            .expect("skipped content should not error");
        assert_eq!(
            outputs.get("request"),
            Some(&Value::Skipped),
            "prepare_write should propagate skipped content via skipped request"
        );
    }

    #[test]
    fn normalize_release_resource_inputs_uses_list_cardinality() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "release_resource_std_resources_Filesystem",
            vec![Port::new("resource_handle", "ResourceHandle")],
            vec![Port::new("released", "Bool")],
            LoweredOp::Callable {
                module: "std.resources".to_string(),
                kind: CallableKind::Pattern,
                name: "resource_lifecycle::release::Filesystem".to_string(),
                obligation: ObligationCategory::ResourceRelease,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));

        let resolved = resolve_lowered_dag(&dag).expect("release node should resolve");
        let release_node = resolved
            .get_node(&"release_resource_std_resources_Filesystem".into())
            .expect("release node should exist");
        let handle_port = release_node
            .inputs
            .iter()
            .find(|port| port.name.0 == "resource_handle")
            .expect("release node should keep resource_handle input");
        assert_eq!(
            handle_port.cardinality,
            Cardinality::ZERO_OR_MORE,
            "release resource_handle input should accept fan-in without scalar conflicts"
        );
    }

    /// FC-5 guardrail: verify that `resolve_lowered_dag` preserves SubDag
    /// structure in the resolved output (production path), rather than flattening
    /// it via `SubDagDispatchOp` (test-only path).
    #[test]
    fn resolve_lowered_dag_preserves_subdag_structure() {
        let mut inner = Dag::new();
        inner.add_node(Node::opaque(
            "inner_literal",
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "literal".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::String("ok".to_string()),
                },
            },
        ));
        let mut dag = Dag::new();
        dag.add_node(Node::subdag("wrapper", inner));

        let resolved = resolve_lowered_dag(&dag).expect("resolve dag with SubDag");
        let wrapper = resolved
            .get_node(&"wrapper".into())
            .expect("wrapper node should exist");
        assert!(
            matches!(wrapper.body, NodeBody::SubDag(_)),
            "production resolver should preserve SubDag structure, not flatten it"
        );
    }

    #[test]
    fn resolve_extern_call_returns_error_for_unknown_symbol() {
        let node = Node::opaque(
            "extern_fetch",
            vec![],
            vec![],
            LoweredOp::ExternCall {
                symbol: "fetch_data".to_string(),
            },
        );
        let err = resolve_node(&node).expect_err("unknown extern call should return error");
        assert!(
            err.reason.contains("extern symbol `fetch_data`"),
            "error should name the extern symbol: {}",
            err.reason
        );
        assert!(
            err.reason.contains("could not be resolved"),
            "error should indicate resolution failure: {}",
            err.reason
        );
    }

    #[test]
    fn resolve_extern_call_succeeds_for_known_extern_impl() {
        let node = Node::opaque(
            "extern_render",
            vec![],
            vec![],
            LoweredOp::ExternCall {
                symbol: "std.markdown::render_tree".to_string(),
            },
        );
        let result = resolve_node(&node);
        assert!(
            result.is_ok(),
            "known extern call should resolve successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn resolve_unsupported_pattern_returns_error_not_panic() {
        let node = Node::opaque(
            "unsupported_retry",
            vec![],
            vec![],
            LoweredOp::UnsupportedPattern {
                name: "RetryController".to_string(),
            },
        );
        let err = resolve_node(&node).expect_err("unsupported pattern should return error");
        assert!(
            err.reason.contains("unsupported pattern `RetryController`"),
            "error should name the unsupported pattern: {}",
            err.reason
        );
    }

    /// Shadow bridge: a Callable (DSL fn body) that has a Rust extern impl.
    /// The extern impl wins at resolution (Step 4 > Step 5). This test
    /// documents the behavior. When NF-7 lands (same-module extern func
    /// wiring), these callables should become ExternCall nodes and this
    /// shadow bridge pattern should be eliminated.
    #[test]
    fn resolve_shadow_bridge_callable_uses_extern_impl_not_passthrough() {
        // tools.pragma::render_clippy_toml has a Rust extern impl.
        // When it appears as a Callable (fn body), the extern impl wins.
        let node = callable_node(
            "render_clippy_toml",
            "tools.pragma",
            "render_clippy_toml",
            ObligationCategory::PureRender,
        );
        let result =
            resolve_node(&node).expect("shadow bridge callable should resolve to extern impl");
        let debug = format!("{result:?}");
        assert!(
            debug.contains("RenderClippyTomlOp"),
            "should resolve to RenderClippyTomlOp (extern impl), not DeclaredOutputCallableOp: {debug}"
        );
    }
}
