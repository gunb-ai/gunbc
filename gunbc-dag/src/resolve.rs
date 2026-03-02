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

use gunbc_resolve::service_ops::{
    GenericFileParseOp, GenericFilePrepareOp, GenericLocalParseOp, GenericLocalPrepareOp,
    GenericRestParseOp, GenericRestPrepareOp, GenericShellParseOp, GenericShellPrepareOp,
    InterfaceStubExecuteOp, InterfaceStubParseOp, InterfaceStubPrepareOp,
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

fn declared_output_ports(outputs: &[Port]) -> Vec<(String, bool)> {
    outputs
        .iter()
        .map(|p| (p.name.0.clone(), p.is_optional()))
        .collect()
}

/// Execute a callable node by forwarding passthrough inputs to declared outputs.
///
/// Enforcement tiers:
///
/// 1. **Partially wired** (at least one `__out:*` input present): missing
///    required outputs are **hard errors** — this indicates a lowerer wiring
///    gap that must be fixed, not masked.
///
/// 2. **Zero wired** (no `__out:*` inputs at all): required outputs fall back
///    to `Value::Skipped`. This is a **C10 gap**: the lowerer cannot yet
///    desugar all return expressions into passthrough edges. Once C10 is
///    complete, this tier collapses into tier 1 and every unwired required
///    output becomes a hard error.
///
/// 3. **Optional** outputs always resolve to `Value::Skipped` when unwired,
///    regardless of tier.
fn execute_with_declared_output_passthrough(
    output_ports: &[(String, bool)],
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let mut outputs = HashMap::new();

    // Partition inputs into regular and passthrough.
    let mut wired_passthroughs = Vec::new();
    for (key, value) in &inputs {
        if let Some(suffix) = key.strip_prefix(PortName::OUTPUT_PASSTHROUGH_PREFIX) {
            wired_passthroughs.push(suffix.to_string());
            continue;
        }
        outputs.insert(key.clone(), value.clone());
    }
    let any_passthrough_wired = !wired_passthroughs.is_empty();

    for (port_name, is_optional) in output_ports {
        let passthrough_key = format!("{}{port_name}", PortName::OUTPUT_PASSTHROUGH_PREFIX);
        if let Some(value) = inputs.get(&passthrough_key) {
            outputs.insert(port_name.clone(), value.clone());
            continue;
        }

        // Output is optional → always Skipped when unwired.
        if *is_optional {
            outputs.insert(port_name.clone(), Value::Skipped);
            continue;
        }

        // C10 gap: zero passthroughs wired → lowerer hasn't desugared return
        // expressions for this callable yet. Fall back to Skipped until C10.
        if !any_passthrough_wired {
            outputs.insert(port_name.clone(), Value::Skipped);
            continue;
        }

        // Fail-closed: at least one passthrough was wired, but this required
        // output is missing. Diagnose which passthroughs ARE present vs. which
        // are expected so the developer can trace the lowerer gap.
        let expected: Vec<String> = output_ports
            .iter()
            .filter(|(_, opt)| !opt)
            .map(|(name, _)| name.clone())
            .collect();
        return Err(ExecError::new(format!(
            "missing required declared output passthrough: `{port_name}` \
             (expected input `{passthrough_key}`). \
             Wired passthroughs: [{}], required outputs: [{}]",
            wired_passthroughs.join(", "),
            expected.join(", "),
        )));
    }
    Ok(outputs)
}

/// Identity callable op for DSL-compiled callables with fn bodies.
///
/// Forwards all inputs to outputs, filling any declared output port that
/// has no matching input with `Value::Skipped`. This is the correct runtime
/// behavior for DSL `fn`/`func` items whose bodies execute as SubDag nodes —
/// the callable node itself is a passthrough that maps SubDag results to outputs.
#[derive(Debug, Clone)]
struct DeclaredOutputCallableOp {
    output_ports: Vec<(String, bool)>,
}

impl Executable for DeclaredOutputCallableOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        execute_with_declared_output_passthrough(&self.output_ports, inputs)
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
    output_ports: Vec<(String, bool)>,
}

impl std::fmt::Debug for PipelineDispatchOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineDispatchOp")
            .field("compat_mode", &"DeclaredOutputCallableOp")
            .field("stage_count", &self.stage_count)
            .field("stage_names", &self.stage_names)
            .field("output_ports", &self.output_ports)
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
        // These outputs are computed by dispatch logic itself, not expected as
        // passthrough inputs from upstream wiring.
        let passthrough_ports: Vec<(String, bool)> = self
            .output_ports
            .iter()
            .filter(|(name, _)| {
                !matches!(
                    name.as_str(),
                    "stages" | "stage_order" | "active_stage" | "next_stage"
                )
            })
            .cloned()
            .collect();
        let mut outputs = execute_with_declared_output_passthrough(&passthrough_ports, inputs)?;
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

/// Compute node for lowered expression evaluation.
/// Evaluates a `LoweredFnBody` using `evaluate_fn_body` with inputs from predecessor nodes.
#[derive(Debug, Clone)]
struct ExprComputeOp {
    fn_body: daglang_lower::LoweredFnBody,
    input_ports: Vec<String>,
    referenced_vars: Vec<String>,
    output_port: String,
}

impl Executable for ExprComputeOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut env = HashMap::new();
        for port in &self.input_ports {
            let value = inputs.get(port).cloned().unwrap_or(Value::Skipped);
            // Flatten Map fields into `parent__field` env vars to match the
            // lowerer's `__` convention (e.g., `entry.kind` lowers to `entry__kind`).
            if let Value::Map(fields) = &value {
                for (field_name, field_value) in fields {
                    env.insert(format!("{port}__{field_name}"), field_value.clone());
                }
            } else if let Value::Json(serde_json::Value::Object(map)) = &value {
                for (field_name, field_value) in map {
                    env.insert(format!("{port}__{field_name}"), Value::Json(field_value.clone()));
                }
            }
            env.insert(port.clone(), value);
        }
        // Pre-seed referenced identifiers so that `__`-flattened field access
        // variables resolve to Skipped when the parent input is itself Skipped
        // (the lowerer emits `entry__kind` but when `entry` is Skipped, the
        // Map flattening above has nothing to flatten).
        for ref_name in &self.referenced_vars {
            if !env.contains_key(ref_name) {
                env.insert(ref_name.clone(), Value::Skipped);
            }
        }
        let sibling_fns = HashMap::new();
        let result = daglang_lower::eval::evaluate_fn_body(&self.fn_body, &env, &sibling_fns)
            .map_err(|e| ExecError::new(e.message))?;
        // The fn body returns { result: <value> }, extract and output it.
        if let Some(value) = result.get(&self.output_port) {
            OutputMap::new()
                .value(self.output_port.as_str(), value.clone())
                .ok()
        } else {
            // Fallback: return whatever the fn body produced.
            Ok(result)
        }
    }
}

/// Collect all `Ident` names referenced in a lowered fn body.
///
/// Used to pre-seed the expression environment with `Value::Skipped` for
/// `__`-flattened field access variables (e.g., `entry__kind`) when the
/// parent input is itself Skipped — the Map flattening only fires for
/// actual `Value::Map` inputs.
fn collect_fn_body_idents(body: &daglang_lower::LoweredFnBody) -> Vec<String> {
    let mut idents = Vec::new();
    for stmt in &body.stmts {
        if let daglang_lower::expr::LoweredStmt::Return(fields) = stmt {
            for (_, expr) in fields {
                collect_lowered_expr_idents(expr, &mut idents);
            }
        }
    }
    idents.sort();
    idents.dedup();
    idents
}

fn collect_lowered_expr_idents(expr: &daglang_lower::expr::LoweredExpr, out: &mut Vec<String>) {
    use daglang_lower::expr::LoweredExpr;
    match expr {
        LoweredExpr::Ident(name) => out.push(name.clone()),
        LoweredExpr::BinOp { left, right, .. } => {
            collect_lowered_expr_idents(left, out);
            collect_lowered_expr_idents(right, out);
        }
        LoweredExpr::UnaryOp { expr, .. } => collect_lowered_expr_idents(expr, out),
        LoweredExpr::IfElse {
            cond, then_, else_, ..
        } => {
            collect_lowered_expr_idents(cond, out);
            collect_lowered_expr_idents(then_, out);
            if let Some(e) = else_ {
                collect_lowered_expr_idents(e, out);
            }
        }
        LoweredExpr::StringInterp(parts) => {
            for part in parts {
                if let daglang_lower::expr::LoweredStringPart::Expr(e) = part {
                    collect_lowered_expr_idents(e, out);
                }
            }
        }
        LoweredExpr::Pipe { receiver, call } => {
            collect_lowered_expr_idents(receiver, out);
            collect_lowered_expr_idents(call, out);
        }
        LoweredExpr::Call { args, .. } => {
            for (_, arg) in args {
                collect_lowered_expr_idents(arg, out);
            }
        }
        LoweredExpr::FieldAccess { expr, .. } => collect_lowered_expr_idents(expr, out),
        LoweredExpr::Record { fields, .. } => {
            for (_, field) in fields {
                collect_lowered_expr_idents(field, out);
            }
        }
        LoweredExpr::List(items) => {
            for item in items {
                collect_lowered_expr_idents(item, out);
            }
        }
        LoweredExpr::Lambda { body, .. } => collect_lowered_expr_idents(body, out),
        LoweredExpr::Match { expr, arms } => {
            collect_lowered_expr_idents(expr, out);
            for arm in arms {
                collect_lowered_expr_idents(&arm.body, out);
            }
        }
        LoweredExpr::For {
            iterable, body, ..
        } => {
            collect_lowered_expr_idents(iterable, out);
            collect_lowered_expr_idents(body, out);
        }
        _ => {}
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
            operation_key: node.operation_key.clone(),
            transport_class: node.transport_class,
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
        NodeBody::Opaque(op) => resolve_op(&node_id, op, &node.inputs, &node.outputs),
        NodeBody::SubDag(inner) => Ok(DynOp::new(SubDagDispatchOp {
            dag: resolve_lowered_dag(inner)?,
        })),
    }
}

fn resolve_node_body(node: &Node<LoweredOp>) -> Result<NodeBody<DynOp>, ResolveError> {
    match &node.body {
        NodeBody::Opaque(op) => Ok(NodeBody::Opaque(resolve_op(
            &node.id.0,
            op,
            &node.inputs,
            &node.outputs,
        )?)),
        NodeBody::SubDag(inner) => Ok(NodeBody::SubDag(resolve_lowered_dag(inner)?)),
    }
}

fn resolve_op(
    node_id: &str,
    op: &LoweredOp,
    inputs: &[Port],
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
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
            output_ports: declared_output_ports(outputs),
        })),
        LoweredOp::Primitive { kind, .. } => resolve_primitive(kind, inputs, outputs),
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
fn resolve_primitive(
    kind: &PrimitiveOpKind,
    inputs: &[Port],
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
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
        PrimitiveOpKind::ExprCompute { fn_body } => {
            let input_ports: Vec<String> = inputs.iter().map(|p| p.name.0.clone()).collect();
            let referenced_vars = collect_fn_body_idents(fn_body);
            let output_port = outputs
                .first()
                .map(|port| port.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(ExprComputeOp {
                fn_body: *fn_body.clone(),
                input_ports,
                referenced_vars,
                output_port,
            }))
        }
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
    _kind: CallableKind,
    outputs: &[Port],
    service_metadata: Option<&ServiceCallMetadata>,
) -> Result<DynOp, ResolveError> {
    // 1. Modules with custom resolvers — return Some for known callables,
    //    None for unknown (which falls through to passthrough).
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    // 2. App-specific callables resolved via extern_ops (single dispatch table).
    if let Some(op) = crate::extern_ops::resolve_extern_symbol(module, name) {
        return Ok(op);
    }
    // 3. Service/workspace modules use generic transport dispatch.
    if module.starts_with("services.") || module.starts_with("workspace.") {
        return resolve_service_transport(node_id, module, name, outputs, service_metadata);
    }
    // 4. Service transport nodes from non-service modules (e.g., loop body
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
        output_ports: declared_output_ports(outputs),
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

    if let Some(op) = crate::extern_ops::resolve_extern_symbol(module, name) {
        return Ok(op);
    }
    if module == "std.resources" {
        return resolve_std_resources(name);
    }

    Err(ResolveError {
        node_id: node_id.to_string(),
        reason: format!(
            "extern symbol `{symbol}` could not be resolved — \
             no extern op or std.resources handler found"
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
                return Ok(DynOp::new(InterfaceStubExecuteOp {
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
                        spec: (**rest_spec).clone(),
                    }));
                }
                (ServiceOperationSpec::Rest(rest_spec), Some(TransportRole::Parse)) => {
                    return Ok(DynOp::new(GenericRestParseOp {
                        spec: (**rest_spec).clone(),
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
                    return Ok(DynOp::new(InterfaceStubPrepareOp {
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
                    return Ok(DynOp::new(InterfaceStubParseOp {
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
                    is_optional: false,
                }],
                output_parsing: ShellOutputParsing::ExitCodeBool,
                env: vec![],
                exit_mapping: vec![],
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
                        is_optional: false,
                    },
                    OutputFieldSpec {
                        name: "stdout".to_string(),
                        type_id: "String".to_string(),
                        json_path: "stdout".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                        is_optional: false,
                    },
                    OutputFieldSpec {
                        name: "stderr".to_string(),
                        type_id: "String".to_string(),
                        json_path: "stderr".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                        is_optional: false,
                    },
                ],
                output_parsing: ShellOutputParsing::SuccessStdoutStderr,
                env: vec![],
                exit_mapping: vec![],
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
            spec: Some(ServiceOperationSpec::Rest(Box::new(RestOperationSpec {
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
                        is_optional: false,
                    },
                    OutputFieldSpec {
                        name: "expires_in".to_string(),
                        type_id: "Int".to_string(),
                        json_path: "expires_in".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                        is_optional: false,
                    },
                ],
                body_template: None,
                headers: vec![],
                auth_scheme: None,
                auth_input: None,
                middleware: None,
                response_mapping: vec![],
            }))),
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
            spec: Some(ServiceOperationSpec::Rest(Box::new(RestOperationSpec {
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
                        is_optional: false,
                    },
                    OutputFieldSpec {
                        name: "name".to_string(),
                        type_id: "String".to_string(),
                        json_path: "name".to_string(),
                        is_secret: false,
                        is_raw_body: false,
                        is_optional: false,
                    },
                ],
                body_template: None,
                headers: vec![],
                auth_scheme: None,
                auth_input: None,
                middleware: None,
                response_mapping: vec![],
            }))),
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

    // ============================================================================
    // Passthrough enforcement (C19)
    // ============================================================================

    #[test]
    fn passthrough_partially_wired_missing_required_is_error() {
        // When at least one __out:* input is wired but a required output is
        // missing, execution must fail with a diagnostic error.
        let output_ports = vec![
            ("result".to_string(), false),
            ("status".to_string(), false),
        ];
        let mut inputs = HashMap::new();
        // Wire only one passthrough — "result" — leave "status" unwired.
        inputs.insert(
            format!("{}result", PortName::OUTPUT_PASSTHROUGH_PREFIX),
            Value::Str("ok".into()),
        );
        let err = execute_with_declared_output_passthrough(&output_ports, inputs)
            .expect_err("should fail when required passthrough is missing");
        let msg = err.to_string();
        assert!(
            msg.contains("status"),
            "error should name the missing output: {msg}"
        );
        assert!(
            msg.contains("Wired passthroughs:"),
            "error should list wired passthroughs: {msg}"
        );
        assert!(
            msg.contains("result"),
            "error should list the wired passthrough name: {msg}"
        );
    }

    #[test]
    fn passthrough_zero_wired_returns_skipped_for_required() {
        // C10 gap: when zero __out:* inputs exist, required outputs fall back
        // to Value::Skipped (not an error). This test documents the gap and
        // will break once C10 is complete and this fallback is removed.
        let output_ports = vec![("result".to_string(), false)];
        let inputs = HashMap::new();
        let outputs = execute_with_declared_output_passthrough(&output_ports, inputs)
            .expect("zero-wired should not error (C10 gap)");
        assert_eq!(
            outputs.get("result"),
            Some(&Value::Skipped),
            "required output should be Skipped when zero passthroughs wired"
        );
    }

    #[test]
    fn passthrough_optional_output_always_skipped_when_unwired() {
        // Optional outputs should be Skipped regardless of wiring state.
        let output_ports = vec![
            ("result".to_string(), false),
            ("details".to_string(), true), // optional
        ];
        let mut inputs = HashMap::new();
        inputs.insert(
            format!("{}result", PortName::OUTPUT_PASSTHROUGH_PREFIX),
            Value::Str("ok".into()),
        );
        // Don't wire __out:details — it's optional.
        let outputs = execute_with_declared_output_passthrough(&output_ports, inputs)
            .expect("optional unwired output should not error");
        assert_eq!(
            outputs.get("result").and_then(Value::as_str),
            Some("ok"),
            "wired output should be forwarded"
        );
        assert_eq!(
            outputs.get("details"),
            Some(&Value::Skipped),
            "optional unwired output should be Skipped"
        );
    }

    #[test]
    fn passthrough_all_wired_succeeds() {
        // Happy path: all required outputs have passthrough inputs wired.
        let output_ports = vec![
            ("result".to_string(), false),
            ("status".to_string(), false),
        ];
        let mut inputs = HashMap::new();
        inputs.insert(
            format!("{}result", PortName::OUTPUT_PASSTHROUGH_PREFIX),
            Value::Str("data".into()),
        );
        inputs.insert(
            format!("{}status", PortName::OUTPUT_PASSTHROUGH_PREFIX),
            Value::Str("ok".into()),
        );
        // Also include a non-passthrough input (should be forwarded).
        inputs.insert("extra".to_string(), Value::Int(42));
        let outputs = execute_with_declared_output_passthrough(&output_ports, inputs)
            .expect("all-wired should succeed");
        assert_eq!(
            outputs.get("result").and_then(Value::as_str),
            Some("data")
        );
        assert_eq!(
            outputs.get("status").and_then(Value::as_str),
            Some("ok")
        );
        assert_eq!(
            outputs.get("extra"),
            Some(&Value::Int(42)),
            "non-passthrough inputs should be forwarded"
        );
    }

}
