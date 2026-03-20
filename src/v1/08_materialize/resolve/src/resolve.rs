//! Central resolver: `LoweredOp` -> `DynOp` via existing domain ops.
//!
//! Maps each lowered operation from a compiled `.dag` file to its concrete
//! `Executable` implementation, wrapped in `DynOp`. This eliminates the need
//! for legacy per-module union enums in app crates.
//!
//! # Architecture
//!
//! Resolution has two layers:
//!
//! 1. **Infrastructure** (cross-module): Typed lowered primitive nodes
//!    (`LoweredOp::Primitive`) map to shared primitive/transport ops.
//!
//! 2. **Domain** (per-module): Module-specific callables (e.g., `gunbc.tools.gist`
//!    / `build_snapshot_content`) map to their domain op variants.
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

use crate::fs_env::FsEnv;
use daglang_lower::{
    CallableKind, CollectionOpKind, LoweredOp, PrimitiveLiteral, PrimitiveOpKind,
    ServiceCallMetadata, ServiceOperationSpec,
};
use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::filename;
use gunbc_ir::node::NodeBody;
use gunbc_ir::patterns::PatternOp;
use gunbc_ir::resource::{
    is_filesystem_resource_port, AccessMode, FILESYSTEM_HANDLE_TYPE, RESOURCE_FILE,
};
use gunbc_ir::transport::{FileRequest, TransportRequest};
use gunbc_ir::types::PortName;
use gunbc_ir::{Cardinality, Dag, Edge, Node, NodeKind, Port, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;

use crate::service_ops::{
    FilesystemExecuteOp, GenericParseOp, GenericPrepareOp, InterfaceStubExecuteOp,
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

fn default_output_port(outputs: &[Port]) -> String {
    outputs
        .first()
        .map(|p| p.name.0.clone())
        .unwrap_or_else(|| "result".to_string())
}

fn declared_output_ports(outputs: &[Port]) -> Vec<(String, bool)> {
    outputs
        .iter()
        .map(|p| (p.name.0.clone(), p.is_optional()))
        .collect()
}

fn require_input_port<'a>(
    inputs: &'a HashMap<String, Value>,
    port: &str,
    op_name: &str,
) -> Result<&'a Value, ExecError> {
    inputs.get(port).ok_or_else(|| {
        let mut available: Vec<&str> = inputs.keys().map(String::as_str).collect();
        available.sort_unstable();
        ExecError::new(format!(
            "{op_name}: missing required input port `{port}`; available inputs: [{}]",
            available.join(", ")
        ))
    })
}

/// Execute a callable node by forwarding passthrough inputs to declared outputs.
///
/// Enforcement tiers:
///
/// 1. **Required outputs** must have a corresponding `__out:*` input wired.
///    Missing required passthrough outputs are always hard errors.
///
/// 2. **Optional** outputs always resolve to `Value::Skipped` when unwired.
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

        // Fail-closed: required passthrough output is missing.
        // Diagnose which passthroughs ARE present vs. which are expected so
        // the developer can trace lowerer wiring gaps.
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

/// Unified callable op for DSL `fn`/`func` items.
///
/// When `fn_body` is `Some`, evaluates the body using the expression evaluator,
/// falling back to passthrough for missing outputs. When `fn_body` is `None`,
/// acts as pure passthrough (maps `__out:` inputs to declared outputs).
#[derive(Clone)]
struct CallableOp {
    fn_body: Option<daglang_lower::LoweredFnBody>,
    output_ports: Vec<(String, bool)>,
    sibling_fns: HashMap<String, daglang_lower::LoweredFnBody>,
    data_values: HashMap<String, Value>,
}

impl std::fmt::Debug for CallableOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallableOp")
            .field("has_fn_body", &self.fn_body.is_some())
            .field("output_ports", &self.output_ports)
            .finish()
    }
}

impl Executable for CallableOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let Some(fn_body) = &self.fn_body else {
            return execute_with_declared_output_passthrough(&self.output_ports, inputs);
        };

        // Build the fn body evaluation environment from non-passthrough inputs.
        let mut eval_inputs = HashMap::new();
        for (key, value) in &inputs {
            if !key.starts_with(PortName::OUTPUT_PASSTHROUGH_PREFIX)
                && key != "__deps"
                && key != "_freshness"
            {
                eval_inputs.insert(key.clone(), value.clone());
            }
        }

        match daglang_lower::eval::evaluate_fn_body_with_data(
            fn_body,
            &eval_inputs,
            &self.sibling_fns,
            &self.data_values,
        ) {
            Ok(body_results) => {
                let mut outputs = HashMap::new();
                for (port_name, is_optional) in &self.output_ports {
                    // 1. Check passthrough (explicit DAG wiring takes priority)
                    let passthrough_key =
                        format!("{}{port_name}", PortName::OUTPUT_PASSTHROUGH_PREFIX);
                    if let Some(value) = inputs.get(&passthrough_key) {
                        if !matches!(value, Value::Skipped) {
                            outputs.insert(port_name.clone(), value.clone());
                            continue;
                        }
                    }
                    // 2. Check fn body evaluation result
                    if let Some(value) = body_results.get(port_name) {
                        outputs.insert(port_name.clone(), value.clone());
                        continue;
                    }
                    // 3. Single-output "return" port: reassemble record fields
                    if port_name == "return"
                        && self.output_ports.len() == 1
                        && !body_results.is_empty()
                    {
                        let map: std::collections::BTreeMap<String, Value> = body_results
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        outputs.insert(port_name.clone(), Value::Map(map));
                        continue;
                    }
                    // 4. Required output missing → error; optional → Skipped
                    if *is_optional {
                        outputs.insert(port_name.clone(), Value::Skipped);
                    } else {
                        return Err(ExecError::new(format!(
                            "FnBody evaluation succeeded but required output `{port_name}` was not produced"
                        )));
                    }
                }
                Ok(outputs)
            }
            Err(eval_err) => {
                // Helper fn items called via evaluate_fn_body as sibling fns
                // may fail when the DAG executor runs them standalone with no
                // real inputs. Suppress errors only when all inputs are Skipped.
                let has_real_inputs = eval_inputs.values().any(|v| !matches!(v, Value::Skipped));
                if !has_real_inputs {
                    let mut outputs = HashMap::new();
                    for (port_name, _) in &self.output_ports {
                        outputs.insert(port_name.clone(), Value::Skipped);
                    }
                    return Ok(outputs);
                }
                // TEMPORARY DEBT (SUSTAINABILITY S3): fn body evaluation is a
                // parallel implementation of DAG execution. The evaluator can't
                // handle all expression forms, so errors are caught here. This
                // masks real regressions — the evaluator can never visibly fail.
                //
                // Exit plan: either (a) make the evaluator complete, (b) remove
                // fn body evaluation from DryRun entirely, or (c) declare eval
                // capability at resolve time so unsupported forms are compile-time
                // opaque, not runtime catch-all. See SUSTAINABILITY.md S3.
                //
                // Until resolved, preserve the error message for diagnostics
                // even though execution continues with passthrough.
                let _ = eval_err; // observable in debug builds via breakpoint
                execute_with_declared_output_passthrough(&self.output_ports, inputs)
            }
        }
    }
}

/// Passthrough for std.resources callables that are neither acquire nor release.
///
/// Resource lifecycle nodes like `probe` or `check` that don't need
/// specialized execution logic use this identity adapter.
#[derive(Debug, Clone)]
struct ResourcePassthroughOp;

impl Executable for ResourcePassthroughOp {
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
        // No fallback to arbitrary inputs — if the named param isn't found,
        // use Value::Skipped so downstream nodes can detect the gap.
        let value = inputs.get(&self.param).cloned().unwrap_or(Value::Skipped);
        Ok(HashMap::from([(self.output_port.clone(), value)]))
    }
}

/// Test-only SubDag execution adapter.
///
/// **Production path**: `resolve_node_body` handles `NodeBody::SubDag` via
/// recursive `resolve_lowered_dag_with(inner, resolver)`, which preserves the SubDag structure
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
                input_mocks.set_input(
                    node_id.0,
                    port_name.0,
                    Value::List(std::sync::Arc::new(Vec::new())),
                );
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

/// C24: Consolidated pure-value primitive op.
///
/// Replaces 10 separate structs (GetFieldOp, StringInterpolateOp, BinaryOpOp,
/// UnaryOpOp, ConditionalOp, MatchDispatchOp, RecordConstructOp, NullCoalesceOp,
/// VariantConstructOp, ListConstructOp) with a single struct that dispatches
/// via `execute_pure_primitive()`.
#[derive(Clone)]
struct PurePrimitiveOp {
    kind: PrimitiveOpKind,
    output_port: String,
    /// Only used for Conditional variant — whether an `else` branch exists.
    has_else: bool,
    /// Validated input port name from node schema (GetField only).
    /// Derived at resolve time from the node's declared inputs, not from the IR.
    input_port: Option<String>,
}

impl std::fmt::Debug for PurePrimitiveOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PurePrimitiveOp")
            .field("kind", &std::mem::discriminant(&self.kind))
            .field("output_port", &self.output_port)
            .finish()
    }
}

impl Executable for PurePrimitiveOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let result = match &self.kind {
            PrimitiveOpKind::GetField { field } => {
                let input_port = self.input_port.as_deref().ok_or_else(|| {
                    ExecError::new(
                        "GetField: validated input port missing from executable (compiler bug)",
                    )
                })?;
                let value = require_input_port(&inputs, input_port, "GetField")?;
                daglang_lower::eval::eval_get_field(value, field)
                    .map_err(|e| ExecError::new(format!("on port `{input_port}`: {e}")))?
            }
            PrimitiveOpKind::StringInterpolate { parts, input_ports } => {
                let values: Vec<Value> = input_ports
                    .iter()
                    .map(|port| require_input_port(&inputs, port, "StringInterpolate").cloned())
                    .collect::<Result<_, _>>()?;
                daglang_lower::eval::eval_string_interpolate(parts, &values)
                    .map_err(|e| ExecError::new(e.to_string()))?
            }
            PrimitiveOpKind::BinaryOp { op: bin_op } => {
                let left = require_input_port(&inputs, "left", "BinaryOp")?.clone();
                let right = require_input_port(&inputs, "right", "BinaryOp")?.clone();
                // Short-circuit semantics: the DAG executor materializes both
                // sides before calling execute(), so we handle short-circuit here
                // rather than in eval_binop (which assumes both sides are ready).
                match bin_op {
                    daglang_lower::expr::LoweredBinOp::And => {
                        if matches!(left, Value::Skipped) {
                            Value::Skipped
                        } else if !daglang_lower::eval::value_truthy(&left) {
                            Value::Bool(false)
                        } else if matches!(right, Value::Skipped) {
                            Value::Skipped
                        } else {
                            Value::Bool(daglang_lower::eval::value_truthy(&right))
                        }
                    }
                    daglang_lower::expr::LoweredBinOp::Or => {
                        if matches!(left, Value::Skipped) {
                            Value::Skipped
                        } else if daglang_lower::eval::value_truthy(&left) {
                            Value::Bool(true)
                        } else if matches!(right, Value::Skipped) {
                            Value::Skipped
                        } else {
                            Value::Bool(daglang_lower::eval::value_truthy(&right))
                        }
                    }
                    _ => daglang_lower::eval::eval_binop(&left, *bin_op, &right)
                        .map_err(|e| ExecError::new(format!("BinaryOp {:?}: {e}", bin_op)))?,
                }
            }
            PrimitiveOpKind::UnaryOp { op: unary_op } => {
                let val = require_input_port(&inputs, "operand", "UnaryOp")?;
                daglang_lower::eval::eval_unary_op(*unary_op, val)
                    .map_err(|e| ExecError::new(e.to_string()))?
            }
            PrimitiveOpKind::Conditional => {
                let condition = require_input_port(&inputs, "condition", "Conditional")?;
                let skipped = Value::Skipped;
                let then_val = inputs.get("then").unwrap_or(&skipped);
                let else_val = if self.has_else {
                    Some(inputs.get("else").unwrap_or(&skipped) as &Value)
                } else {
                    None
                };
                daglang_lower::eval::eval_conditional(condition, then_val, else_val)
            }
            PrimitiveOpKind::MatchDispatch { arms, sibling_fns } => {
                let scrutinee = require_input_port(&inputs, "scrutinee", "MatchDispatch")?.clone();
                if matches!(scrutinee, Value::Skipped) {
                    Value::Skipped
                } else {
                    let env: HashMap<String, Value> = inputs
                        .iter()
                        .filter(|(k, _)| k.as_str() != "scrutinee")
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    let sibling_fns_map: HashMap<String, daglang_lower::LoweredFnBody> =
                        sibling_fns
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                    daglang_lower::eval::eval_match(&scrutinee, arms, &env, &sibling_fns_map)
                        .map_err(|e| ExecError::new(format!("MatchDispatch: {e}")))?
                }
            }
            PrimitiveOpKind::RecordConstruct { fields } => {
                let field_values: Vec<(String, Value)> = fields
                    .iter()
                    .map(|field| {
                        let value = require_input_port(&inputs, field, "RecordConstruct")?.clone();
                        Ok((field.clone(), value))
                    })
                    .collect::<Result<_, ExecError>>()?;
                daglang_lower::eval::eval_record_construct(&field_values)
                    .map_err(|e| ExecError::new(e.to_string()))?
            }
            PrimitiveOpKind::NullCoalesce => {
                let value = require_input_port(&inputs, "value", "NullCoalesce")?;
                let default_value = require_input_port(&inputs, "default", "NullCoalesce")?;
                daglang_lower::eval::eval_null_coalesce(value, default_value)
            }
            PrimitiveOpKind::VariantConstruct { tag, fields } => {
                let field_values: Vec<(String, Value)> = fields
                    .iter()
                    .map(|field| {
                        let value = require_input_port(&inputs, field, "VariantConstruct")?.clone();
                        Ok((field.clone(), value))
                    })
                    .collect::<Result<_, ExecError>>()?;
                daglang_lower::eval::eval_variant_construct(tag, &field_values)
                    .map_err(|e| ExecError::new(e.to_string()))?
            }
            PrimitiveOpKind::ListConstruct { count } => {
                let elements: Vec<Value> = (0..*count)
                    .map(|i| {
                        let port = format!("elem_{i}");
                        require_input_port(&inputs, &port, "ListConstruct").cloned()
                    })
                    .collect::<Result<_, _>>()?;
                daglang_lower::eval::eval_list_construct(elements)
                    .map_err(|e| ExecError::new(e.to_string()))?
            }
            // Non-pure-value variants should never reach PurePrimitiveOp
            _ => unreachable!(
                "PurePrimitiveOp received non-pure-value PrimitiveOpKind: {:?}",
                std::mem::discriminant(&self.kind)
            ),
        };
        OutputMap::new().value(&self.output_port, result).ok()
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
            .value(FILESYSTEM_HANDLE_TYPE, fs)
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
/// Requires canonical `path` and `content` inputs. The lowering pipeline
/// always stamps prepare-write nodes with exactly these two port names
/// (see `expand_single_content_upsert` and `add_content_upsert_chain`),
/// so no alias guessing is needed.
#[derive(Debug, Clone)]
struct PrepareFileWriteCompatOp;

impl Executable for PrepareFileWriteCompatOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        if matches!(inputs.get("path"), Some(Value::Skipped))
            || matches!(inputs.get("content"), Some(Value::Skipped))
        {
            return OutputMap::new()
                .value("request", Value::Skipped)
                .bool("skip", true)
                .ok();
        }
        let path = inputs.get("path").and_then(Value::as_str).ok_or_else(|| {
            let input_keys = {
                let mut keys = inputs.keys().cloned().collect::<Vec<_>>();
                keys.sort();
                keys.join(", ")
            };
            ExecError::new(format!(
                "PrepareFileWrite: missing required `path` input — check content-upsert wiring (available inputs: {input_keys})"
            ))
        })?;
        let content = inputs.get("content").and_then(Value::as_str).ok_or_else(|| {
            let input_keys = {
                let mut keys = inputs.keys().cloned().collect::<Vec<_>>();
                keys.sort();
                keys.join(", ")
            };
            ExecError::new(format!(
                "PrepareFileWrite: missing required `content` input — check content-upsert wiring (available inputs: {input_keys})"
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
///
/// Data declaration values are extracted from embedded `__data_decl::` nodes
/// in the DAG itself — no external sidecar needed.
pub fn resolve_lowered_dag_with(dag: &Dag<LoweredOp>) -> Result<Dag<DynOp>, ResolveError> {
    let data_values = daglang_lower::extract_data_values_from_dag(dag);
    resolve_lowered_dag_impl(dag, &data_values)
}

/// Resolve a lowered DAG with explicit data declaration values.
///
/// Prefer `resolve_lowered_dag_with` which extracts data from embedded DAG nodes.
/// This variant is kept for SubDag resolution where parent data_values are inherited.
fn resolve_lowered_dag_impl(
    dag: &Dag<LoweredOp>,
    data_values: &HashMap<String, Value>,
) -> Result<Dag<DynOp>, ResolveError> {
    // Collect all fn bodies from callable nodes for cross-fn evaluation.
    // Helper fns call sibling fns via evaluate_fn_body; this map provides
    // them at execution time.
    let sibling_fns = collect_sibling_fn_bodies(dag);

    // Pipeline nodes are compile-time metadata (used by emit/derive/driver)
    // but have no runtime representation. Collect their IDs to filter edges.
    let pipeline_ids: std::collections::HashSet<&str> = dag
        .nodes
        .iter()
        .filter_map(|node| match &node.body {
            NodeBody::Opaque(LoweredOp::Pipeline { .. }) => Some(node.id.0.as_str()),
            _ => None,
        })
        .collect();

    let mut resolved = Dag::new();
    for node in &dag.nodes {
        if pipeline_ids.contains(node.id.0.as_str()) {
            continue;
        }
        // Skip data declaration embed nodes — metadata only, extracted above.
        if node.id.0.starts_with(daglang_lower::DATA_DECL_NODE_PREFIX) {
            continue;
        }
        let mut resolved_node = Node {
            id: node.id.clone(),
            inputs: node.inputs.clone(),
            outputs: node.outputs.clone(),
            body: resolve_node_body(node, &sibling_fns, data_values)?,
            examples: node.examples.clone(),
            log_detail: node.log_detail,
            kind: node.kind,
            operation_key: node.operation_key.clone(),
            transport_class: node.transport_class,
            response_provider: node.response_provider,
            static_fingerprint: None,
            origin: node.origin.clone(),
            input_alias: node.input_alias.clone(),
        };
        normalize_release_resource_inputs(node, &mut resolved_node);
        if let Some(mode) = needs_transport_resource(node, &resolved_node) {
            resolved_node
                .inputs
                .push(Port::resource(RESOURCE_FILE, FILESYSTEM_HANDLE_TYPE, mode));
        }
        resolved.add_node(resolved_node);
    }
    // Filter edges referencing pipeline or data declaration nodes.
    resolved.edges = dag
        .edges
        .iter()
        .filter(|edge| {
            !pipeline_ids.contains(edge.from_node.0.as_str())
                && !pipeline_ids.contains(edge.to_node.0.as_str())
                && !edge
                    .from_node
                    .0
                    .starts_with(daglang_lower::DATA_DECL_NODE_PREFIX)
                && !edge
                    .to_node
                    .0
                    .starts_with(daglang_lower::DATA_DECL_NODE_PREFIX)
        })
        .cloned()
        .collect();
    wire_missing_filesystem_resources(&mut resolved);
    Ok(resolved)
}

/// Collect fn bodies from all callable nodes in a DAG, keyed by callable name.
///
/// This enables cross-fn evaluation: when `render_core_recipe` calls
/// `apply_prefix`, the evaluator can look up `apply_prefix`'s fn body.
fn collect_sibling_fn_bodies(
    dag: &Dag<LoweredOp>,
) -> HashMap<String, daglang_lower::LoweredFnBody> {
    let mut fns = HashMap::new();
    for node in &dag.nodes {
        if let NodeBody::Opaque(LoweredOp::Callable {
            name,
            fn_body: Some(body),
            ..
        }) = &node.body
        {
            fns.insert(name.clone(), body.as_ref().clone());
        }
    }
    fns
}

fn normalize_release_resource_inputs(lowered: &Node<LoweredOp>, node: &mut Node<DynOp>) {
    if lowered.kind != NodeKind::ResourceRelease {
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
    let empty_siblings = HashMap::new();
    let empty_data = HashMap::new();
    let node_id = node.id.0.clone();
    match &node.body {
        NodeBody::Opaque(op) => resolve_op(
            &node_id,
            op,
            &node.inputs,
            &node.outputs,
            &empty_siblings,
            &empty_data,
        ),
        NodeBody::SubDag(inner, _kind) => Ok(DynOp::new(SubDagDispatchOp {
            dag: resolve_lowered_dag_with(inner)?,
        })),
    }
}

fn resolve_node_body(
    node: &Node<LoweredOp>,
    sibling_fns: &HashMap<String, daglang_lower::LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<NodeBody<DynOp>, ResolveError> {
    match &node.body {
        NodeBody::Opaque(op) => Ok(NodeBody::Opaque(resolve_op(
            &node.id.0,
            op,
            &node.inputs,
            &node.outputs,
            sibling_fns,
            data_values,
        )?)),
        NodeBody::SubDag(inner, kind) => Ok(NodeBody::SubDag(
            resolve_lowered_dag_impl(inner, data_values)?,
            kind.clone(),
        )),
    }
}

fn resolve_op(
    node_id: &str,
    op: &LoweredOp,
    inputs: &[Port],
    outputs: &[Port],
    sibling_fns: &HashMap<String, daglang_lower::LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<DynOp, ResolveError> {
    match op {
        LoweredOp::Collection { kind, .. } => resolve_collection(kind),
        LoweredOp::Pipeline { .. } => {
            // Pipeline nodes are filtered in resolve_lowered_dag_with.
            // This arm is unreachable but kept for exhaustiveness.
            unreachable!("pipeline nodes are skipped before resolve_node_body is called")
        }
        LoweredOp::Primitive { kind, .. } => resolve_primitive(node_id, kind, inputs, outputs),
        LoweredOp::Callable {
            module,
            name,
            kind,
            fn_body,
            ..
        } => resolve_domain(
            node_id,
            module,
            name,
            *kind,
            outputs,
            None,
            fn_body.as_deref(),
            sibling_fns,
            data_values,
        ),
        LoweredOp::Transport {
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
            Some(service_metadata),
            None,
            sibling_fns,
            data_values,
        ),
        LoweredOp::Pattern(pattern_op) => Ok(DynOp::new(pattern_op.clone())),
        LoweredOp::UnsupportedPattern { name } => Err(ResolveError {
            node_id: node_id.to_string(),
            reason: format!(
                "unsupported pattern `{name}` — not yet implemented in daglang lowering"
            ),
        }),
    }
}

// ============================================================================
// Infrastructure resolution (cross-module patterns)
// ============================================================================

/// Resolve typed lowered primitive nodes shared across all modules.
///
/// For expression primitives, validates that the node's declared input ports
/// cover all ports required by the `PrimitiveOpKind`. This catches incomplete
/// primitive input maps at resolve time (before execution), rather than
/// deferring to runtime `require_input_port()` failures.
fn resolve_primitive(
    node_id: &str,
    kind: &PrimitiveOpKind,
    inputs: &[Port],
    outputs: &[Port],
) -> Result<DynOp, ResolveError> {
    // Validate declared ports cover the kind's required ports.
    if let Some(required) = kind.required_input_ports() {
        let declared: std::collections::HashSet<&str> =
            inputs.iter().map(|p| p.name.0.as_str()).collect();
        let missing: Vec<&str> = required
            .iter()
            .filter(|port| !declared.contains(port.as_str()))
            .map(String::as_str)
            .collect();
        if !missing.is_empty() {
            let mut declared_sorted: Vec<&str> = declared.into_iter().collect();
            declared_sorted.sort_unstable();
            return Err(ResolveError {
                node_id: node_id.to_string(),
                reason: format!(
                    "primitive node has incomplete input ports: missing [{}]; declared [{}]",
                    missing.join(", "),
                    declared_sorted.join(", "),
                ),
            });
        }
    }
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
        PrimitiveOpKind::ContentUpsertOutputPath { .. } => Ok(DynOp::new(ResourcePassthroughOp)),
        PrimitiveOpKind::GetField { field } => {
            if inputs.len() != 1 {
                return Err(ResolveError {
                    node_id: node_id.to_string(),
                    reason: format!(
                        "GetField `{field}`: expected exactly 1 declared input port, found {} (compiler bug)",
                        inputs.len()
                    ),
                });
            }
            let validated_input = inputs[0].name.0.clone();
            let output_port =
                outputs
                    .first()
                    .map(|p| p.name.0.clone())
                    .ok_or_else(|| ResolveError {
                        node_id: node_id.to_string(),
                        reason: format!(
                            "GetField `{field}`: node has no output port (compiler bug)"
                        ),
                    })?;
            Ok(DynOp::new(PurePrimitiveOp {
                kind: kind.clone(),
                output_port,
                has_else: false,
                input_port: Some(validated_input),
            }))
        }
        PrimitiveOpKind::Conditional => {
            let output_port = default_output_port(outputs);
            let has_else = inputs.iter().any(|port| port.name.0 == "else");
            Ok(DynOp::new(PurePrimitiveOp {
                kind: kind.clone(),
                output_port,
                has_else,
                input_port: None,
            }))
        }
        PrimitiveOpKind::StringInterpolate { .. }
        | PrimitiveOpKind::BinaryOp { .. }
        | PrimitiveOpKind::UnaryOp { .. }
        | PrimitiveOpKind::MatchDispatch { .. }
        | PrimitiveOpKind::RecordConstruct { .. }
        | PrimitiveOpKind::NullCoalesce
        | PrimitiveOpKind::VariantConstruct { .. }
        | PrimitiveOpKind::ListConstruct { .. } => Ok(DynOp::new(PurePrimitiveOp {
            kind: kind.clone(),
            output_port: default_output_port(outputs),
            has_else: false,
            input_port: None,
        })),
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
    fn_body: Option<&daglang_lower::LoweredFnBody>,
    sibling_fns: &HashMap<String, daglang_lower::LoweredFnBody>,
    data_values: &HashMap<String, Value>,
) -> Result<DynOp, ResolveError> {
    // 1. Modules with custom resolvers — return Some for known callables,
    //    None for unknown (which falls through to passthrough).
    if module == "std.resources" {
        return resolve_std_resources(name);
    }
    // 2. Service/workspace modules use generic transport dispatch — but only
    //    for nodes that ARE transport roles (prepare/execute/parse) or have
    //    service metadata. Pure fn items in provider modules fall through to
    //    the default DeclaredOutputCallableOp passthrough.
    if (module.starts_with("services.")
        || module.starts_with("workspace.")
        || module.starts_with("extdeps."))
        && (TransportRole::from_name(name).is_some() || service_metadata.is_some())
    {
        return resolve_service_transport(node_id, module, name, outputs, service_metadata);
    }
    // 4. Service transport nodes from non-service modules (e.g., loop body
    //    transport nodes which inherit the tool module name, not the service module).
    //    Route all transport roles through the transport resolver and fail
    //    closed when metadata/specs are missing.
    if TransportRole::from_name(name).is_some() {
        return resolve_service_transport(node_id, module, name, outputs, service_metadata);
    }
    // 5. C10: fn items with fn bodies use CallableOp to evaluate the
    //    body directly, producing outputs from the fn's computation.
    if let Some(body) = fn_body {
        return Ok(DynOp::new(CallableOp {
            fn_body: Some(body.clone()),
            output_ports: declared_output_ports(outputs),
            sibling_fns: sibling_fns.clone(),
            data_values: data_values.clone(),
        }));
    }
    // 5b. Pattern callables without fn_body: patterns are expanded inline as
    //     separate DAG nodes. The callable node is a structural marker whose
    //     __out: passthrough ports may not be wired (the expanded nodes
    //     produce values directly). All outputs are optional here.
    // 5b-6: Callables without fn_body (patterns, transport-backed funcs, defaults)
    //        use passthrough: map SubDag / __out: wired results to output ports.
    let ports = if _kind == CallableKind::Pattern {
        outputs.iter().map(|p| (p.name.0.clone(), true)).collect()
    } else {
        declared_output_ports(outputs)
    };
    Ok(DynOp::new(CallableOp {
        fn_body: None,
        output_ports: ports,
        sibling_fns: HashMap::new(),
        data_values: HashMap::new(),
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
    Ok(DynOp::new(ResourcePassthroughOp))
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
    // Filesystem interface gets a concrete binding via FilesystemExecuteOp.
    if role == Some(TransportRole::Execute) {
        if let Some(metadata) = service_metadata {
            if let Some(ServiceOperationSpec::InterfaceStub {
                interface,
                capability,
            }) = &metadata.spec
            {
                if interface == "Filesystem" {
                    return Ok(DynOp::new(FilesystemExecuteOp {
                        capability: capability.clone(),
                    }));
                }
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
            match role {
                Some(TransportRole::Prepare) => {
                    return Ok(DynOp::new(GenericPrepareOp { spec: spec.clone() }));
                }
                Some(TransportRole::Parse) => {
                    return Ok(DynOp::new(GenericParseOp {
                        spec: spec.clone(),
                        service_name: metadata.service.clone(),
                        operation_name: metadata.operation.clone(),
                    }));
                }
                // Execute role is handled by the early return above.
                Some(TransportRole::Execute) | None => {}
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
    Ok(DynOp::new(PatternOp::CollectionAggregate { kind: *kind }))
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

    // Only add if not already present (S15: structural query).
    let already_has = resolved.inputs.iter().any(is_filesystem_resource_port);
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
            if !is_filesystem_resource_port(port) {
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
            .find(|port| port.type_id.0 == FILESYSTEM_HANDLE_TYPE)
            .map(|port| port.name.0.clone())
            .unwrap_or_else(|| FILESYSTEM_HANDLE_TYPE.to_string())
    } else {
        dag.add_node(
            Node::opaque(
                fs_node_id.as_str(),
                vec![],
                vec![Port::new(FILESYSTEM_HANDLE_TYPE, FILESYSTEM_HANDLE_TYPE)],
                DynOp::new(DslFsEnvOp),
            )
            .with_kind(gunbc_ir::NodeKind::ResourceEnvironment),
        );
        FILESYSTEM_HANDLE_TYPE.to_string()
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
    use daglang_lower::{
        CallableKind, CallableObligation, PrimitiveLiteral, PrimitiveOpKind, TransportObligation,
    };
    use gunbc_exec::{
        execute_dag, BoundaryMocks, DryRunStrictness, ExecuteConfig, ExecutionMode, NodeRole,
    };
    use gunbc_ir::{Edge, Node, Port};

    fn callable_node(
        id: &str,
        module: &str,
        name: &str,
        obligation: CallableObligation,
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

    // NOTE: resolve_tools_infra_entrypoint_emits_plan_summary test moved to
    // gunbc-tests (requires GunbcExternResolver for tools.infra::infra dispatch).

    #[test]
    fn resolve_pipeline_node_is_skipped() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "pipeline_demo",
            vec![],
            vec![Port::new("stages", "Int")],
            LoweredOp::Pipeline {
                module: "pipelines.demo".to_string(),
                name: "demo".to_string(),
                stages: 2,
                stage_names: vec!["fetch".to_string(), "design".to_string()],
            },
        ));
        let resolved = resolve_lowered_dag_with(&dag).expect("should succeed");
        assert!(
            resolved.nodes.is_empty(),
            "pipeline nodes should be filtered out, got {} nodes",
            resolved.nodes.len()
        );
    }

    #[test]
    fn resolve_and_execute_transport_and_pipeline_nodes_before_interpreter() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "pipeline_demo",
            vec![],
            vec![Port::new("stages", "Int")],
            LoweredOp::Pipeline {
                module: "pipelines.demo".to_string(),
                name: "demo".to_string(),
                stages: 2,
                stage_names: vec!["fetch".to_string(), "design".to_string()],
            },
        ));
        dag.add_node(
            Node::opaque(
                "execute_transport",
                vec![],
                vec![Port::new("response", "TransportResponse")],
                LoweredOp::Transport {
                    module: "extdeps.shell".to_string(),
                    kind: CallableKind::Fn,
                    name: "service_transport::execute::shell.Codegen::Check".to_string(),
                    obligation: TransportObligation::Execute,
                    service_metadata: Box::new(codegen_check_metadata()),
                    is_interactive: false,
                    resource_target: None,
                },
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let resolved = resolve_lowered_dag_with(&dag).expect("resolve lowered dag");
        assert!(
            resolved.get_node(&"pipeline_demo".into()).is_none(),
            "pipeline metadata should be filtered before execution"
        );

        let mut mocks = BoundaryMocks::new();
        mocks.set_value("execute_transport", "response", Value::Skipped);

        let log = execute_dag(
            &resolved,
            ExecuteConfig {
                mode: ExecutionMode::DryRun(mocks),
                ..Default::default()
            },
        )
        .expect("resolved transport node should be intercepted in dry-run");

        assert!(
            log.get("pipeline_demo").is_none(),
            "pipeline metadata should never reach the executor"
        );
        assert!(
            log.get("execute_transport")
                .expect("transport node should remain executable")
                .was_intercepted,
            "transport execute node should be intercepted before any interpreter path"
        );
    }

    #[test]
    fn resolve_and_execute_missing_required_primitive_input_fails_closed() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "left_src",
            vec![],
            vec![Port::new("out", "Int")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "left_src".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::Int(1),
                },
            },
        ));
        dag.add_node(Node::opaque(
            "binary",
            vec![Port::new("left", "Int"), Port::new("right", "Int")],
            vec![Port::new("result", "Int")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "binary".to_string(),
                kind: PrimitiveOpKind::BinaryOp {
                    op: daglang_lower::expr::LoweredBinOp::Add,
                },
            },
        ));
        dag.add_edge(Edge::new("left_src", "out", "binary", "left"));

        let resolved = resolve_lowered_dag_with(&dag).expect("resolve lowered dag");
        let err = execute_dag(
            &resolved,
            ExecuteConfig {
                mode: ExecutionMode::Real,
                strictness: DryRunStrictness::Lenient,
                ..Default::default()
            },
        )
        .expect_err("missing primitive input should fail through the executor path");

        assert_eq!(
            err.message,
            "BinaryOp: missing required input port `right`; available inputs: [left]"
        );
        let node_trace = err
            .node_trace()
            .expect("executor path should annotate primitive failures with node trace");
        assert_eq!(node_trace.node_id, "binary");
        assert_eq!(node_trace.role, NodeRole::Pure);
    }

    #[test]
    fn resolve_rejects_primitive_with_incomplete_declared_ports() {
        // A BinaryOp node that only declares "left" — missing required "right".
        // This must fail at resolve time, not wait until execution.
        let node = Node::opaque(
            "binary",
            vec![Port::new("left", "Int")],
            vec![Port::new("result", "Int")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "binary".to_string(),
                kind: PrimitiveOpKind::BinaryOp {
                    op: daglang_lower::expr::LoweredBinOp::Add,
                },
            },
        );
        let err = resolve_node(&node)
            .expect_err("resolve should reject primitive with incomplete declared ports");
        assert!(
            err.reason.contains("missing [right]"),
            "error should name the missing port: {}",
            err.reason
        );
        assert_eq!(err.node_id, "binary");
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
    fn obligation_from_transport_name(name: &str) -> TransportObligation {
        if name.contains("::prepare::") {
            TransportObligation::Prepare
        } else if name.contains("::execute::") {
            TransportObligation::Execute
        } else if name.contains("::parse::") {
            TransportObligation::Parse
        } else {
            panic!("unknown transport name pattern: {name}")
        }
    }

    fn service_callable_node(
        id: &str,
        module: &str,
        name: &str,
        obligation: TransportObligation,
        metadata: ServiceCallMetadata,
    ) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![],
            vec![Port::new("out", "String")],
            LoweredOp::Transport {
                module: module.to_string(),
                kind: CallableKind::Fn,
                name: name.to_string(),
                obligation,
                service_metadata: Box::new(metadata),
                is_interactive: false,
                resource_target: None,
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
            response_provider: None,
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
            spec: Some(ServiceOperationSpec::Shell(ShellOperationSpec {
                argv_template: vec![
                    ArgvSegment::Literal("cargo".to_string()),
                    ArgvSegment::Literal("run".to_string()),
                    ArgvSegment::Literal("-p".to_string()),
                    ArgvSegment::Literal("gunbc-codegen".to_string()),
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
            response_provider: None,
        }
    }

    #[test]
    fn resolve_services_shell_codegen_transport_ops() {
        // Prepare nodes use generic shell prepare/parse with spec from metadata.
        let cases = [
            (
                "service_transport::prepare::shell.Codegen::Check",
                codegen_check_metadata(),
                "GenericPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Check",
                codegen_check_metadata(),
                "GenericParseOp",
            ),
            (
                "service_transport::prepare::shell.Codegen::Run",
                codegen_run_metadata(),
                "GenericPrepareOp",
            ),
            (
                "service_transport::parse::shell.Codegen::Run",
                codegen_run_metadata(),
                "GenericParseOp",
            ),
        ];

        for (name, metadata, expected_debug) in cases {
            let node = service_callable_node(
                name,
                "extdeps.shell",
                name,
                obligation_from_transport_name(name),
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
            CallableObligation::None,
        );
        let result =
            resolve_node(&node).expect("tools.codegen::codegen should resolve via passthrough");
        let debug = format!("{result:?}");
        assert!(
            debug.contains("CallableOp"),
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
            "tools.bootstrap",
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
            "tools.bootstrap",
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
            "tools.bootstrap",
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
            "tools.bootstrap",
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
            "tools.bootstrap",
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
                body_schema: None,
                headers: vec![],
                auth_scheme: None,
                auth_input: None,
                middleware: None,
                response_mapping: vec![],
                output_shape: None,
                mock_responses: vec![],
            }))),
            response_provider: None,
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
                body_schema: None,
                headers: vec![],
                auth_scheme: None,
                auth_input: None,
                middleware: None,
                response_mapping: vec![],
                output_shape: None,
                mock_responses: vec![],
            }))),
            response_provider: None,
        }
    }

    #[test]
    fn resolve_services_gcp_transport_ops() {
        let cases = [
            (
                "extdeps.cloud.gcp.sts",
                "service_transport::prepare::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericPrepareOp",
            ),
            (
                "extdeps.cloud.gcp.sts",
                "service_transport::parse::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericParseOp",
            ),
            (
                "extdeps.cloud.gcp.secret_manager",
                "service_transport::prepare::gcp.SecretManager::AccessVersion",
                secret_manager_metadata(),
                "GenericPrepareOp",
            ),
            (
                "extdeps.cloud.gcp.secret_manager",
                "service_transport::parse::gcp.SecretManager::AccessVersion",
                secret_manager_metadata(),
                "GenericParseOp",
            ),
        ];
        for (module, name, metadata, expected_debug) in cases {
            let node = service_callable_node(
                name,
                module,
                name,
                obligation_from_transport_name(name),
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
    fn resolve_collection_map() {
        let node = collection_node("map_items", CollectionOpKind::Map);
        let result = resolve_node(&node).expect("map");
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(std::sync::Arc::new(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ])),
        );
        let outputs = result
            .execute(inputs)
            .expect("collection map should execute");
        assert_eq!(
            outputs.get("items"),
            Some(&Value::List(std::sync::Arc::new(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string())
            ])))
        );
    }

    #[test]
    fn resolve_collection_len() {
        let node = collection_node("len_items", CollectionOpKind::Len);
        let result = resolve_node(&node).expect("len");
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(std::sync::Arc::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])),
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
            Value::List(std::sync::Arc::new(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ])),
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
            CallableObligation::None,
        );
        let result = resolve_node(&node).expect("unknown modules should resolve via passthrough");
        let debug = format!("{result:?}");
        assert!(
            debug.contains("CallableOp"),
            "should use DeclaredOutputCallableOp: {debug}"
        );
    }

    #[test]
    fn resolve_unknown_callable_in_custom_module_uses_passthrough() {
        let node = callable_node(
            "bad_op",
            "tools.bootstrap",
            "nonexistent_op",
            CallableObligation::None,
        );
        let result = resolve_node(&node).expect("unknown callable should resolve via passthrough");
        let debug = format!("{result:?}");
        assert!(
            debug.contains("CallableOp"),
            "should use DeclaredOutputCallableOp: {debug}"
        );
    }

    // NOTE: resolve_infra_callable_maps_to_infra_dispatch_op test moved to
    // gunbc-tests (requires GunbcExternResolver for tools.infra::infra dispatch).

    #[test]
    fn resolve_unknown_service_transport_prepare_fails() {
        let node = service_callable_node(
            "bad_service_prepare",
            "extdeps.cloud.gcp.sts",
            "service_transport::prepare::gcp.STS::Refresh",
            TransportObligation::Prepare,
            ServiceCallMetadata {
                service: "gcp.STS".to_string(),
                operation: "Refresh".to_string(),
                transport: daglang_lower::ServiceTransportClass::RestNetwork,
                idempotent: false,
                readonly: false,
                spec: None,
                response_provider: None,
            },
        );
        let err = resolve_node(&node).unwrap_err();
        assert!(
            err.reason.contains("unknown callable")
                || err.reason.contains("no matching operation spec"),
            "expected failure for unknown service transport prepare, got: {}",
            err.reason
        );
    }

    #[test]
    fn resolve_full_dag_preserves_edges() {
        let mut dag = Dag::new();
        dag.add_node(callable_node(
            "render",
            "tools.bootstrap",
            "render_clippy_toml",
            CallableObligation::None,
        ));
        dag.add_node(primitive_node(
            "prepare_read",
            "tools.bootstrap",
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

        let resolved = resolve_lowered_dag_with(&dag).expect("resolve dag");
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
            vec![Port::resource(
                "file",
                FILESYSTEM_HANDLE_TYPE,
                AccessMode::Read,
            )],
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
            vec![Port::new(FILESYSTEM_HANDLE_TYPE, FILESYSTEM_HANDLE_TYPE)],
            DynOp::new(DslFsEnvOp),
        ));
        dag.add_node(Node::opaque(
            "execute_read_makegen",
            vec![Port::resource(
                "file",
                FILESYSTEM_HANDLE_TYPE,
                AccessMode::Read,
            )],
            vec![Port::new("response", "TransportResponse")],
            DynOp::new(DslFsEnvOp),
        ));

        wire_missing_filesystem_resources(&mut dag);

        let has_edge = dag.edges.iter().any(|edge| {
            edge.from_node.0 == "fs_env"
                && edge.from_port.0 == FILESYSTEM_HANDLE_TYPE
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
    fn normalize_release_resource_inputs_uses_structural_resource_release_kind() {
        let mut dag = Dag::new();
        dag.add_node(
            Node::opaque(
                "release_filesystem",
                vec![Port::new("resource_handle", "ResourceHandle")],
                vec![Port::new("released", "Bool")],
                LoweredOp::Callable {
                    module: "std.resources".to_string(),
                    kind: CallableKind::Pattern,
                    name: "resource_lifecycle::release::Filesystem".to_string(),
                    obligation: CallableObligation::ResourceRelease,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::ResourceRelease),
        );

        let resolved = resolve_lowered_dag_with(&dag).expect("release node should resolve");
        let release_node = resolved
            .get_node(&"release_filesystem".into())
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

        let resolved = resolve_lowered_dag_with(&dag).expect("resolve dag with SubDag");
        let wrapper = resolved
            .get_node(&"wrapper".into())
            .expect("wrapper node should exist");
        assert!(
            matches!(wrapper.body, NodeBody::SubDag(..)),
            "production resolver should preserve SubDag structure, not flatten it"
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
        let output_ports = vec![("result".to_string(), false), ("status".to_string(), false)];
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
    fn passthrough_zero_wired_missing_required_is_error() {
        // Required outputs must always be wired via __out:* passthrough inputs.
        let output_ports = vec![("result".to_string(), false)];
        let inputs = HashMap::new();
        let err = execute_with_declared_output_passthrough(&output_ports, inputs)
            .expect_err("zero-wired required output should error");
        let msg = err.to_string();
        assert!(
            msg.contains("missing required declared output passthrough: `result`"),
            "error should report missing required passthrough output: {msg}"
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
        let output_ports = vec![("result".to_string(), false), ("status".to_string(), false)];
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
        assert_eq!(outputs.get("result").and_then(Value::as_str), Some("data"));
        assert_eq!(outputs.get("status").and_then(Value::as_str), Some("ok"));
        assert_eq!(
            outputs.get("extra"),
            Some(&Value::Int(42)),
            "non-passthrough inputs should be forwarded"
        );
    }

    // ── C24 Structural Primitive Tests ──────────────────────────────────────

    #[test]
    fn resolve_string_interpolate_concatenates_parts() {
        let node = Node::opaque(
            "interp",
            vec![Port::new("name", "String"), Port::new("count", "Int")],
            vec![Port::new("result", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "string_interpolate".to_string(),
                kind: PrimitiveOpKind::StringInterpolate {
                    parts: vec![
                        "hello ".to_string(),
                        ", you have ".to_string(),
                        " items".to_string(),
                    ],
                    input_ports: vec!["name".to_string(), "count".to_string()],
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::Str("Alice".to_string()));
        inputs.insert("count".to_string(), Value::Int(42));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(
            outputs.get("result").and_then(Value::as_str),
            Some("hello Alice, you have 42 items")
        );
    }

    #[test]
    fn resolve_binary_op_add_strings() {
        let node = Node::opaque(
            "binop",
            vec![Port::new("left", "String"), Port::new("right", "String")],
            vec![Port::new("result", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "binary_op".to_string(),
                kind: PrimitiveOpKind::BinaryOp {
                    op: daglang_lower::expr::LoweredBinOp::Add,
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("left".to_string(), Value::Str("foo".to_string()));
        inputs.insert("right".to_string(), Value::Str("bar".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(
            outputs.get("result").and_then(Value::as_str),
            Some("foobar")
        );
    }

    #[test]
    fn resolve_binary_op_compare_ints() {
        let node = Node::opaque(
            "binop",
            vec![Port::new("left", "Int"), Port::new("right", "Int")],
            vec![Port::new("result", "Bool")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "binary_op".to_string(),
                kind: PrimitiveOpKind::BinaryOp {
                    op: daglang_lower::expr::LoweredBinOp::Gt,
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("left".to_string(), Value::Int(10));
        inputs.insert("right".to_string(), Value::Int(5));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(outputs.get("result"), Some(&Value::Bool(true)));
    }

    #[test]
    fn resolve_unary_op_not() {
        let node = Node::opaque(
            "unop",
            vec![Port::new("operand", "Bool")],
            vec![Port::new("result", "Bool")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "unary_op".to_string(),
                kind: PrimitiveOpKind::UnaryOp {
                    op: daglang_lower::expr::LoweredUnaryOp::Not,
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("operand".to_string(), Value::Bool(true));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(outputs.get("result"), Some(&Value::Bool(false)));
    }

    #[test]
    fn resolve_conditional_selects_then_branch() {
        let node = Node::opaque(
            "cond",
            vec![
                Port::new("condition", "Bool"),
                Port::new("then", "String"),
                Port::new("else", "String"),
            ],
            vec![Port::new("result", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "conditional".to_string(),
                kind: PrimitiveOpKind::Conditional,
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("condition".to_string(), Value::Bool(true));
        inputs.insert("then".to_string(), Value::Str("yes".to_string()));
        inputs.insert("else".to_string(), Value::Str("no".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(outputs.get("result").and_then(Value::as_str), Some("yes"));
    }

    #[test]
    fn resolve_conditional_selects_else_branch() {
        let node = Node::opaque(
            "cond",
            vec![
                Port::new("condition", "Bool"),
                Port::new("then", "String"),
                Port::new("else", "String"),
            ],
            vec![Port::new("result", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "conditional".to_string(),
                kind: PrimitiveOpKind::Conditional,
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("condition".to_string(), Value::Bool(false));
        inputs.insert("then".to_string(), Value::Str("yes".to_string()));
        inputs.insert("else".to_string(), Value::Str("no".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(outputs.get("result").and_then(Value::as_str), Some("no"));
    }

    #[test]
    fn resolve_match_dispatch_selects_matching_arm() {
        use daglang_lower::expr::*;
        let node = Node::opaque(
            "match",
            vec![Port::new("scrutinee", "String")],
            vec![Port::new("result", "Int")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "match_dispatch".to_string(),
                kind: PrimitiveOpKind::MatchDispatch {
                    arms: vec![
                        LoweredMatchArm {
                            pattern: LoweredPattern::Literal(LoweredLiteral::String(
                                "a".to_string(),
                            )),
                            guard: None,
                            body: LoweredExpr::Literal(LoweredLiteral::Int(1)),
                        },
                        LoweredMatchArm {
                            pattern: LoweredPattern::Literal(LoweredLiteral::String(
                                "b".to_string(),
                            )),
                            guard: None,
                            body: LoweredExpr::Literal(LoweredLiteral::Int(2)),
                        },
                        LoweredMatchArm {
                            pattern: LoweredPattern::Wildcard,
                            guard: None,
                            body: LoweredExpr::Literal(LoweredLiteral::Int(0)),
                        },
                    ],
                    sibling_fns: std::collections::BTreeMap::new(),
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("scrutinee".to_string(), Value::Str("b".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(outputs.get("result"), Some(&Value::Int(2)));
    }

    #[test]
    fn resolve_record_construct_builds_map() {
        let node = Node::opaque(
            "record",
            vec![Port::new("x", "Int"), Port::new("y", "String")],
            vec![Port::new("result", "Record")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "record_construct".to_string(),
                kind: PrimitiveOpKind::RecordConstruct {
                    fields: vec!["x".to_string(), "y".to_string()],
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("x".to_string(), Value::Int(42));
        inputs.insert("y".to_string(), Value::Str("hello".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        match outputs.get("result") {
            Some(Value::Map(m)) => {
                assert_eq!(m.get("x"), Some(&Value::Int(42)));
                assert_eq!(m.get("y"), Some(&Value::Str("hello".to_string())));
            }
            other => panic!("expected Map, got {:?}", other),
        }
    }

    #[test]
    fn resolve_null_coalesce_returns_value_when_present() {
        let node = Node::opaque(
            "coalesce",
            vec![Port::new("value", "String"), Port::new("default", "String")],
            vec![Port::new("result", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "null_coalesce".to_string(),
                kind: PrimitiveOpKind::NullCoalesce,
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), Value::Str("real".to_string()));
        inputs.insert("default".to_string(), Value::Str("fallback".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(outputs.get("result").and_then(Value::as_str), Some("real"));
    }

    #[test]
    fn resolve_null_coalesce_returns_default_when_null() {
        let node = Node::opaque(
            "coalesce",
            vec![Port::new("value", "String"), Port::new("default", "String")],
            vec![Port::new("result", "String")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "null_coalesce".to_string(),
                kind: PrimitiveOpKind::NullCoalesce,
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), Value::Unit);
        inputs.insert("default".to_string(), Value::Str("fallback".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        assert_eq!(
            outputs.get("result").and_then(Value::as_str),
            Some("fallback")
        );
    }

    #[test]
    fn resolve_variant_construct_unit() {
        let node = Node::opaque(
            "variant",
            vec![],
            vec![Port::new("result", "BoxStyle")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "variant_construct".to_string(),
                kind: PrimitiveOpKind::VariantConstruct {
                    tag: "Closed".to_string(),
                    fields: vec![],
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let outputs = result.execute(HashMap::new()).expect("should execute");
        match outputs.get("result") {
            Some(Value::Enum { variant, .. }) => assert_eq!(variant, "Closed"),
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn resolve_variant_construct_payload() {
        let node = Node::opaque(
            "variant",
            vec![Port::new("value", "String")],
            vec![Port::new("result", "Result")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "variant_construct".to_string(),
                kind: PrimitiveOpKind::VariantConstruct {
                    tag: "Ok".to_string(),
                    fields: vec!["value".to_string()],
                },
            },
        );
        let result = resolve_node(&node).expect("should resolve");
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), Value::Str("data".to_string()));
        let outputs = result.execute(inputs).expect("should execute");
        match outputs.get("result") {
            Some(Value::Map(m)) => {
                assert_eq!(m.get("_variant"), Some(&Value::Str("Ok".to_string())));
                assert_eq!(m.get("value"), Some(&Value::Str("data".to_string())));
            }
            other => panic!("expected Map with _variant, got {:?}", other),
        }
    }

    #[test]
    fn conditional_op_without_else_returns_skipped() {
        let op = PurePrimitiveOp {
            kind: PrimitiveOpKind::Conditional,
            output_port: "result".to_string(),
            has_else: false,
            input_port: None,
        };
        let mut inputs = HashMap::new();
        inputs.insert("condition".to_string(), Value::Bool(false));
        // No "else" input → should return Skipped, not Unit
        let outputs = op.execute(inputs).expect("should execute");
        assert_eq!(
            outputs.get("result"),
            Some(&Value::Skipped),
            "missing else branch should produce Skipped, not Unit"
        );
    }

    #[test]
    fn conditional_op_missing_condition_errors() {
        let op = PurePrimitiveOp {
            kind: PrimitiveOpKind::Conditional,
            output_port: "result".to_string(),
            has_else: true,
            input_port: None,
        };
        let mut inputs = HashMap::new();
        inputs.insert("then".to_string(), Value::Str("yes".to_string()));
        inputs.insert("else".to_string(), Value::Str("no".to_string()));
        let err = op
            .execute(inputs)
            .expect_err("missing condition should be a hard error");
        let msg = err.to_string();
        assert!(
            msg.contains("missing required input port `condition`"),
            "diagnostic should mention missing condition port: {msg}"
        );
    }

    #[test]
    fn binary_op_missing_right_errors() {
        let op = PurePrimitiveOp {
            kind: PrimitiveOpKind::BinaryOp {
                op: daglang_lower::expr::LoweredBinOp::Add,
            },
            output_port: "result".to_string(),
            has_else: false,
            input_port: None,
        };
        let mut inputs = HashMap::new();
        inputs.insert("left".to_string(), Value::Int(1));
        let err = op
            .execute(inputs)
            .expect_err("missing right input should be a hard error");
        let msg = err.to_string();
        assert!(
            msg.contains("missing required input port `right`"),
            "diagnostic should mention missing right port: {msg}"
        );
    }

    #[test]
    fn record_construct_missing_field_errors() {
        let op = PurePrimitiveOp {
            kind: PrimitiveOpKind::RecordConstruct {
                fields: vec!["x".to_string(), "y".to_string()],
            },
            output_port: "result".to_string(),
            has_else: false,
            input_port: None,
        };
        let mut inputs = HashMap::new();
        inputs.insert("x".to_string(), Value::Int(1));
        let err = op
            .execute(inputs)
            .expect_err("missing field input should be a hard error");
        let msg = err.to_string();
        assert!(
            msg.contains("missing required input port `y`"),
            "diagnostic should mention missing field port: {msg}"
        );
    }

    #[test]
    fn collection_aggregate_skipped_input_propagates() {
        let op = PatternOp::CollectionAggregate {
            kind: CollectionOpKind::Map,
        };
        let mut inputs = HashMap::new();
        inputs.insert("items".to_string(), Value::Skipped);
        let outputs = op.execute(inputs).expect("should execute");
        assert_eq!(
            outputs.get("items"),
            Some(&Value::Skipped),
            "Skipped input should propagate as Skipped, not empty list"
        );
    }

    #[test]
    fn resolve_get_field_rejects_no_declared_inputs() {
        let node = Node::opaque(
            "extract_field",
            vec![],
            vec![Port::scalar("value", "String")],
            LoweredOp::Primitive {
                module: "test".into(),
                name: "get_field::test::empty::name".into(),
                kind: PrimitiveOpKind::GetField {
                    field: "name".into(),
                },
            },
        );
        let err = resolve_node(&node)
            .expect_err("GetField with no declared inputs must fail at resolve time");
        assert!(
            err.reason
                .contains("expected exactly 1 declared input port"),
            "error should mention input count: {}",
            err.reason
        );
    }

    #[test]
    fn resolve_get_field_rejects_multiple_declared_inputs() {
        let node = Node::opaque(
            "extract_field",
            vec![
                Port::scalar("record", "Json"),
                Port::scalar("other", "Json"),
            ],
            vec![Port::scalar("value", "String")],
            LoweredOp::Primitive {
                module: "test".into(),
                name: "get_field::test::multi::name".into(),
                kind: PrimitiveOpKind::GetField {
                    field: "name".into(),
                },
            },
        );
        let err = resolve_node(&node)
            .expect_err("GetField with multiple declared inputs must fail at resolve time");
        assert!(
            err.reason
                .contains("expected exactly 1 declared input port"),
            "error should mention input count: {}",
            err.reason
        );
    }

    #[test]
    fn resolve_get_field_derives_input_from_schema() {
        // GetField derives its input port from the node's sole declared input,
        // not from a duplicated field in PrimitiveOpKind.
        let node = Node::opaque(
            "extract_field",
            vec![Port::scalar("record", "Json")],
            vec![Port::scalar("value", "String")],
            LoweredOp::Primitive {
                module: "test".into(),
                name: "get_field::test::record::name".into(),
                kind: PrimitiveOpKind::GetField {
                    field: "name".into(),
                },
            },
        );
        let op = resolve_node(&node).expect("GetField with one declared input should resolve");

        let mut inputs = HashMap::new();
        inputs.insert(
            "record".to_string(),
            Value::Map(
                [("name".to_string(), Value::Str("alice".to_string()))]
                    .into_iter()
                    .collect(),
            ),
        );

        let outputs = op
            .execute(inputs)
            .expect("GetField should execute using schema-derived input port");
        assert_eq!(outputs.get("value").and_then(Value::as_str), Some("alice"),);
    }
}
