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
//! 2. **Domain** (per-module): Module-specific callables (e.g., `tools.gist`
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

use daglang_lower::{
    CallableKind, CollectionOpKind, LoweredOp, PrimitiveLiteral, PrimitiveOpKind,
    ServiceCallMetadata, ServiceOperationSpec,
};
use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::node::NodeBody;
use gunbc_ir::patterns::PatternOp;
use gunbc_ir::resource::{AccessMode, RESOURCE_FILE, RESOURCE_FILE_PREFIX};
use gunbc_ir::transport::{FileRequest, TransportRequest};
use gunbc_ir::types::PortName;
use gunbc_ir::{Cardinality, Dag, Edge, Node, Port, Value};
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{filename, FsEnv};

use crate::service_ops::{
    FilesystemExecuteOp, GenericFileParseOp, GenericFilePrepareOp, GenericLocalParseOp,
    GenericLocalPrepareOp, GenericRestParseOp, GenericRestPrepareOp, GenericShellParseOp,
    GenericShellPrepareOp, InterfaceStubExecuteOp, InterfaceStubParseOp, InterfaceStubPrepareOp,
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

/// C10: Callable op for `fn` items that evaluates the fn body directly.
///
/// Evaluates the fn's lowered body using the expression evaluator. The fn body
/// can handle if/else, for-loops, match, pipes, records, and all other pure
/// expression forms — producing the return value directly from the fn's logic.
///
/// When the fn is a helper called only via `evaluate_fn_body` from sibling
/// fns, the DAG executor may run it standalone with no real inputs. In that
/// case (all user-facing inputs are absent or Skipped), evaluation failure is
/// expected and Skipped outputs are produced. If real inputs are present and
/// evaluation still fails, the error propagates.
#[derive(Clone)]
struct FnBodyCallableOp {
    fn_body: daglang_lower::LoweredFnBody,
    output_ports: Vec<(String, bool)>,
    /// Sibling fn bodies from the same DAG, keyed by callable name.
    /// Used by `evaluate_fn_body` for cross-fn calls within the same module.
    sibling_fns: HashMap<String, daglang_lower::LoweredFnBody>,
}

impl std::fmt::Debug for FnBodyCallableOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnBodyCallableOp")
            .field("output_ports", &self.output_ports)
            .finish()
    }
}

impl Executable for FnBodyCallableOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // Build the fn body evaluation environment from non-passthrough inputs.
        let mut eval_inputs = HashMap::new();
        let mut passthrough_inputs = HashMap::new();
        for (key, value) in &inputs {
            if key.starts_with(PortName::OUTPUT_PASSTHROUGH_PREFIX) {
                passthrough_inputs.insert(key.clone(), value.clone());
            } else if key != "__deps" && key != "_freshness" {
                eval_inputs.insert(key.clone(), value.clone());
            }
        }

        // Try to evaluate the fn body. On success, map the results to declared
        // output ports. Passthrough inputs override fn body results (they come
        // from explicit DAG wiring and are more authoritative).
        match daglang_lower::eval::evaluate_fn_body(&self.fn_body, &eval_inputs, &self.sibling_fns)
        {
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
                    // 3. Single-output "return" port: if the fn body produced
                    //    a record expression (unwrapped into individual fields
                    //    by the evaluator), reassemble them as the return value.
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
                // Helper fn items are called via evaluate_fn_body from sibling
                // fn bodies, not through DAG edge wiring. When the DAG executor
                // runs them standalone, parameter inputs are absent or Skipped.
                // Only suppress the error in that case; if real inputs were
                // provided and evaluation still failed, propagate the error
                // (README: "no silent degradation").
                let has_real_inputs = eval_inputs
                    .values()
                    .any(|v| !matches!(v, Value::Skipped | Value::Unit));
                if has_real_inputs {
                    return Err(ExecError::new(format!(
                        "FnBody evaluation failed with real inputs present: {eval_err}"
                    )));
                }
                let mut outputs = HashMap::new();
                for (port_name, _) in &self.output_ports {
                    outputs.insert(port_name.clone(), Value::Skipped);
                }
                Ok(outputs)
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

/// C24: Extract a named field from a Map/Record/JSON input.
/// Pure structural projection — no runtime interpreter needed.
/// Fail-closed: missing field, missing input port, or non-Map/Json input → ExecError.
#[derive(Debug, Clone)]
struct GetFieldOp {
    input_port: String,
    field: String,
    output_port: String,
}

impl Executable for GetFieldOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let value = match inputs.get(&self.input_port) {
            Some(value) => value.clone(),
            None => {
                return Err(ExecError::new(format!(
                    "GetField `{}`: missing input port `{}` in inputs map (resolver/runtime bug)",
                    self.field, self.input_port
                )));
            }
        };
        let extracted = match &value {
            Value::Map(fields) => fields.get(&self.field).cloned().ok_or_else(|| {
                let mut available: Vec<&String> = fields.keys().collect();
                available.sort();
                ExecError::new(format!(
                    "GetField `{}`: field not found in Map on port `{}`. Available fields: {:?}",
                    self.field, self.input_port, available
                ))
            })?,
            Value::Json(serde_json::Value::Object(map)) => {
                map.get(&self.field).map(|v| Value::Json(v.clone())).ok_or_else(|| {
                    let mut available: Vec<&String> = map.keys().collect();
                    available.sort();
                    ExecError::new(format!(
                        "GetField `{}`: field not found in Json object on port `{}`. Available fields: {:?}",
                        self.field, self.input_port, available
                    ))
                })?
            }
            Value::Skipped => {
                return Err(ExecError::new(format!(
                    "GetField `{}`: input port `{}` is Skipped (unwired or missing upstream)",
                    self.field, self.input_port
                )));
            }
            other => {
                return Err(ExecError::new(format!(
                    "GetField `{}`: expected Map or Json object on port `{}`, got {:?}",
                    self.field, self.input_port, other
                )));
            }
        };
        OutputMap::new()
            .value(self.output_port.as_str(), extracted)
            .ok()
    }
}

/// C24: String interpolation — concatenate static parts with dynamic values.
/// Inputs: one port per interpolated expression. Output: concatenated string.
#[derive(Debug, Clone)]
struct StringInterpolateOp {
    parts: Vec<String>,
    input_ports: Vec<String>,
    output_port: String,
}

impl Executable for StringInterpolateOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        // If any interpolated input is Skipped, propagate Skipped.
        for port in &self.input_ports {
            if matches!(
                require_input_port(&inputs, port, "StringInterpolate")?,
                Value::Skipped
            ) {
                return OutputMap::new()
                    .value(&self.output_port, Value::Skipped)
                    .ok();
            }
        }
        let mut result = String::new();
        for (i, part) in self.parts.iter().enumerate() {
            result.push_str(part);
            if i < self.input_ports.len() {
                let value =
                    require_input_port(&inputs, &self.input_ports[i], "StringInterpolate")?.clone();
                result.push_str(&daglang_lower::eval::value_to_string(&value));
            }
        }
        OutputMap::new()
            .value(&self.output_port, Value::Str(result))
            .ok()
    }
}

/// C24: Binary operation — applies an operator to two input values.
/// Inputs: `left`, `right`. Output: result of the operation.
#[derive(Debug, Clone)]
struct BinaryOpOp {
    op: daglang_lower::expr::LoweredBinOp,
    output_port: String,
}

impl Executable for BinaryOpOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let left = require_input_port(&inputs, "left", "BinaryOp")?.clone();
        let right = require_input_port(&inputs, "right", "BinaryOp")?.clone();
        // Handle short-circuit semantics for logical/null-coalesce ops
        let result = match self.op {
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
            daglang_lower::expr::LoweredBinOp::NullCoalesce => {
                if !matches!(left, Value::Unit | Value::Skipped) {
                    left
                } else {
                    right
                }
            }
            op => {
                if matches!(left, Value::Skipped) || matches!(right, Value::Skipped) {
                    Value::Skipped
                } else {
                    daglang_lower::eval::eval_binop(&left, op, &right)
                        .map_err(|e| ExecError::new(format!("BinaryOp {:?}: {}", self.op, e)))?
                }
            }
        };
        OutputMap::new().value(&self.output_port, result).ok()
    }
}

/// C24: Unary operation — `!x` or `-x`.
/// Input: `operand`. Output: result.
#[derive(Debug, Clone)]
struct UnaryOpOp {
    op: daglang_lower::expr::LoweredUnaryOp,
    output_port: String,
}

impl Executable for UnaryOpOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let val = require_input_port(&inputs, "operand", "UnaryOp")?.clone();
        if matches!(val, Value::Skipped) {
            return OutputMap::new()
                .value(&self.output_port, Value::Skipped)
                .ok();
        }
        let result = match self.op {
            daglang_lower::expr::LoweredUnaryOp::Not => {
                Value::Bool(!daglang_lower::eval::value_truthy(&val))
            }
            daglang_lower::expr::LoweredUnaryOp::Neg => match val {
                Value::Int(i) => Value::Int(-i),
                Value::Float(f) => Value::Float(-f),
                other => {
                    return Err(ExecError::new(format!(
                        "UnaryOp Neg: cannot negate {:?}",
                        other
                    )));
                }
            },
        };
        OutputMap::new().value(&self.output_port, result).ok()
    }
}

/// C24: Conditional — selects between `then` and `else` based on `condition`.
/// Inputs: `condition`, `then`, `else`. Output: selected branch.
#[derive(Debug, Clone)]
struct ConditionalOp {
    output_port: String,
    has_else: bool,
}

impl Executable for ConditionalOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let condition = require_input_port(&inputs, "condition", "Conditional")?.clone();
        if matches!(condition, Value::Skipped) {
            return OutputMap::new()
                .value(&self.output_port, Value::Skipped)
                .ok();
        }
        let result = if daglang_lower::eval::value_truthy(&condition) {
            require_input_port(&inputs, "then", "Conditional")?.clone()
        } else if self.has_else {
            require_input_port(&inputs, "else", "Conditional")?.clone()
        } else {
            Value::Skipped
        };
        OutputMap::new().value(&self.output_port, result).ok()
    }
}

/// C24: Match dispatch — evaluates match arms against a scrutinee value.
/// Input: `scrutinee` (plus any captured env values by name). Output: matched body result.
#[derive(Clone)]
struct MatchDispatchOp {
    arms: Vec<daglang_lower::expr::LoweredMatchArm>,
    sibling_fns: HashMap<String, daglang_lower::LoweredFnBody>,
    output_port: String,
}

impl std::fmt::Debug for MatchDispatchOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchDispatchOp")
            .field("arms", &self.arms.len())
            .field("sibling_fns", &self.sibling_fns.len())
            .field("output_port", &self.output_port)
            .finish()
    }
}

impl Executable for MatchDispatchOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let scrutinee = require_input_port(&inputs, "scrutinee", "MatchDispatch")?.clone();
        if matches!(scrutinee, Value::Skipped) {
            return OutputMap::new()
                .value(&self.output_port, Value::Skipped)
                .ok();
        }
        // Build an env from all non-scrutinee inputs for arm body evaluation
        let env: HashMap<String, Value> = inputs
            .iter()
            .filter(|(k, _)| k.as_str() != "scrutinee")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let result =
            daglang_lower::eval::eval_match(&scrutinee, &self.arms, &env, &self.sibling_fns)
                .map_err(|e| ExecError::new(format!("MatchDispatch: {e}")))?;
        OutputMap::new().value(&self.output_port, result).ok()
    }
}

/// C24: Record construction — assembles named fields into a Value::Map.
/// Inputs: one port per field name. Output: Value::Map with all fields.
#[derive(Debug, Clone)]
struct RecordConstructOp {
    fields: Vec<String>,
    output_port: String,
}

impl Executable for RecordConstructOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut map = std::collections::BTreeMap::new();
        for field in &self.fields {
            let value = require_input_port(&inputs, field, "RecordConstruct")?.clone();
            if matches!(value, Value::Skipped) {
                return OutputMap::new()
                    .value(&self.output_port, Value::Skipped)
                    .ok();
            }
            map.insert(field.clone(), value);
        }
        OutputMap::new()
            .value(&self.output_port, Value::Map(map))
            .ok()
    }
}

/// C24: Null coalesce — `a ?? b`. Returns `value` if non-null, else `default`.
/// Inputs: `value`, `default`. Output: coalesced result.
#[derive(Debug, Clone)]
struct NullCoalesceOp {
    output_port: String,
}

impl Executable for NullCoalesceOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let value = require_input_port(&inputs, "value", "NullCoalesce")?.clone();
        let default_value = require_input_port(&inputs, "default", "NullCoalesce")?.clone();
        let result = if matches!(value, Value::Unit | Value::Skipped) {
            default_value
        } else {
            value
        };
        OutputMap::new().value(&self.output_port, result).ok()
    }
}

/// C24: Variant construction — produces tagged sum-type values.
/// Unit variants → Value::Str(tag). Payload variants → Value::Map with `_variant` tag.
#[derive(Debug, Clone)]
struct VariantConstructOp {
    tag: String,
    fields: Vec<String>,
    output_port: String,
}

impl Executable for VariantConstructOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let result = if self.fields.is_empty() {
            // Unit variant
            Value::Enum {
                ty: String::new(),
                variant: self.tag.clone(),
            }
        } else {
            // Payload variant — if any field is Skipped, propagate Skipped.
            let mut map = std::collections::BTreeMap::new();
            map.insert("_variant".to_string(), Value::Str(self.tag.clone()));
            for field in &self.fields {
                let value = require_input_port(&inputs, field, "VariantConstruct")?.clone();
                if matches!(value, Value::Skipped) {
                    return OutputMap::new()
                        .value(&self.output_port, Value::Skipped)
                        .ok();
                }
                map.insert(field.clone(), value);
            }
            Value::Map(map)
        };
        OutputMap::new().value(&self.output_port, result).ok()
    }
}

/// C24: List construction — collects `elem_0`, `elem_1`, ... input ports into Value::List.
#[derive(Debug, Clone)]
struct ListConstructOp {
    count: usize,
    output_port: String,
}

impl Executable for ListConstructOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut elements = Vec::with_capacity(self.count);
        for i in 0..self.count {
            let port = format!("elem_{i}");
            let value = require_input_port(&inputs, &port, "ListConstruct")?.clone();
            if matches!(value, Value::Skipped) {
                return OutputMap::new()
                    .value(&self.output_port, Value::Skipped)
                    .ok();
            }
            elements.push(value);
        }
        OutputMap::new()
            .value(&self.output_port, Value::List(elements))
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
pub fn resolve_lowered_dag_with(dag: &Dag<LoweredOp>) -> Result<Dag<DynOp>, ResolveError> {
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
        let mut resolved_node = Node {
            id: node.id.clone(),
            inputs: node.inputs.clone(),
            outputs: node.outputs.clone(),
            body: resolve_node_body(node, &sibling_fns)?,
            examples: node.examples.clone(),
            log_detail: node.log_detail,
            kind: node.kind,
            operation_key: node.operation_key.clone(),
            transport_class: node.transport_class,
            static_fingerprint: None,
            origin: node.origin.clone(),
        };
        normalize_release_resource_inputs(&mut resolved_node);
        if let Some(mode) = needs_transport_resource(node, &resolved_node) {
            resolved_node
                .inputs
                .push(Port::resource(RESOURCE_FILE, "FilesystemHandle", mode));
        }
        resolved.add_node(resolved_node);
    }
    // Filter edges referencing pipeline nodes.
    resolved.edges = dag
        .edges
        .iter()
        .filter(|edge| {
            !pipeline_ids.contains(edge.from_node.0.as_str())
                && !pipeline_ids.contains(edge.to_node.0.as_str())
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
    let empty_siblings = HashMap::new();
    let node_id = node.id.0.clone();
    match &node.body {
        NodeBody::Opaque(op) => {
            resolve_op(&node_id, op, &node.inputs, &node.outputs, &empty_siblings)
        }
        NodeBody::SubDag(inner, _kind) => Ok(DynOp::new(SubDagDispatchOp {
            dag: resolve_lowered_dag_with(inner)?,
        })),
    }
}

fn resolve_node_body(
    node: &Node<LoweredOp>,
    sibling_fns: &HashMap<String, daglang_lower::LoweredFnBody>,
) -> Result<NodeBody<DynOp>, ResolveError> {
    match &node.body {
        NodeBody::Opaque(op) => Ok(NodeBody::Opaque(resolve_op(
            &node.id.0,
            op,
            &node.inputs,
            &node.outputs,
            sibling_fns,
        )?)),
        NodeBody::SubDag(inner, kind) => Ok(NodeBody::SubDag(
            resolve_lowered_dag_with(inner)?,
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
) -> Result<DynOp, ResolveError> {
    match op {
        LoweredOp::Collection { kind, .. } => resolve_collection(kind),
        LoweredOp::Pipeline { .. } => {
            // Pipeline nodes are filtered in resolve_lowered_dag_with.
            // This arm is unreachable but kept for exhaustiveness.
            unreachable!("pipeline nodes are skipped before resolve_node_body is called")
        }
        LoweredOp::Primitive { kind, .. } => resolve_primitive(kind, inputs, outputs),
        LoweredOp::Callable {
            module,
            name,
            kind,
            service_metadata,
            fn_body,
            ..
        } => resolve_domain(
            node_id,
            module,
            name,
            *kind,
            outputs,
            service_metadata.as_deref(),
            fn_body.as_deref(),
            sibling_fns,
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
        PrimitiveOpKind::ContentUpsertOutputPath { .. } => Ok(DynOp::new(ResourcePassthroughOp)),
        PrimitiveOpKind::GetField { field } => {
            let input_port =
                inputs
                    .first()
                    .map(|p| p.name.0.clone())
                    .ok_or_else(|| ResolveError {
                        node_id: String::new(),
                        reason: format!(
                            "GetField `{field}`: node has no input port (compiler bug)"
                        ),
                    })?;
            let output_port =
                outputs
                    .first()
                    .map(|p| p.name.0.clone())
                    .ok_or_else(|| ResolveError {
                        node_id: String::new(),
                        reason: format!(
                            "GetField `{field}`: node has no output port (compiler bug)"
                        ),
                    })?;
            Ok(DynOp::new(GetFieldOp {
                input_port,
                field: field.clone(),
                output_port,
            }))
        }
        PrimitiveOpKind::StringInterpolate { parts, input_ports } => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(StringInterpolateOp {
                parts: parts.clone(),
                input_ports: input_ports.clone(),
                output_port,
            }))
        }
        PrimitiveOpKind::BinaryOp { op } => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(BinaryOpOp {
                op: *op,
                output_port,
            }))
        }
        PrimitiveOpKind::UnaryOp { op } => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(UnaryOpOp {
                op: *op,
                output_port,
            }))
        }
        PrimitiveOpKind::Conditional => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            let has_else = inputs.iter().any(|port| port.name.0 == "else");
            Ok(DynOp::new(ConditionalOp {
                output_port,
                has_else,
            }))
        }
        PrimitiveOpKind::MatchDispatch { arms, sibling_fns } => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(MatchDispatchOp {
                arms: arms.clone(),
                sibling_fns: sibling_fns
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                output_port,
            }))
        }
        PrimitiveOpKind::RecordConstruct { fields } => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(RecordConstructOp {
                fields: fields.clone(),
                output_port,
            }))
        }
        PrimitiveOpKind::NullCoalesce => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(NullCoalesceOp { output_port }))
        }
        PrimitiveOpKind::VariantConstruct { tag, fields } => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(VariantConstructOp {
                tag: tag.clone(),
                fields: fields.clone(),
                output_port,
            }))
        }
        PrimitiveOpKind::ListConstruct { count } => {
            let output_port = outputs
                .first()
                .map(|p| p.name.0.clone())
                .unwrap_or_else(|| "result".to_string());
            Ok(DynOp::new(ListConstructOp {
                count: *count,
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
    fn_body: Option<&daglang_lower::LoweredFnBody>,
    sibling_fns: &HashMap<String, daglang_lower::LoweredFnBody>,
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
    // 5. C10: fn items with fn bodies use FnBodyCallableOp to evaluate the
    //    body directly, producing outputs from the fn's computation.
    if let Some(body) = fn_body {
        return Ok(DynOp::new(FnBodyCallableOp {
            fn_body: body.clone(),
            output_ports: declared_output_ports(outputs),
            sibling_fns: sibling_fns.clone(),
        }));
    }
    // 5b. Pattern callables without fn_body: patterns are expanded inline as
    //     separate DAG nodes. The callable node is a structural marker whose
    //     __out: passthrough ports may not be wired (the expanded nodes
    //     produce values directly). All outputs are optional here.
    if _kind == CallableKind::Pattern {
        let optional_ports: Vec<(String, bool)> =
            outputs.iter().map(|p| (p.name.0.clone(), true)).collect();
        return Ok(DynOp::new(DeclaredOutputCallableOp {
            output_ports: optional_ports,
        }));
    }
    // 5c. Func callables without fn_body: the body is expressed as transport
    //     nodes (prepare/execute/parse) which wire __out: passthrough ports.
    //     Preserve declared port optionality — if a required output is
    //     unwired, that's a lowering bug that should surface, not be masked.
    if _kind == CallableKind::Func {
        return Ok(DynOp::new(DeclaredOutputCallableOp {
            output_ports: declared_output_ports(outputs),
        }));
    }
    // 6. Default: identity callable for compiler-validated callables.
    //
    // All LoweredOp::Callable nodes are produced by the DSL compiler (the
    // lowerer only emits Callable for items in the typed project). The
    // callable's logic is wired as separate nodes/edges in the DAG; this
    // wrapper node maps SubDag results to output ports via passthrough.
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
                        auth_scheme: rest_spec
                            .auth_scheme
                            .clone()
                            .unwrap_or_else(|| "none".to_string()),
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
                "extdeps.shell",
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
                headers: vec![],
                auth_scheme: None,
                auth_input: None,
                middleware: None,
                response_mapping: vec![],
                output_shape: None,
                mock_responses: vec![],
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
                output_shape: None,
                mock_responses: vec![],
            }))),
        }
    }

    #[test]
    fn resolve_services_gcp_transport_ops() {
        let cases = [
            (
                "extdeps.cloud.gcp.sts",
                "service_transport::prepare::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericRestPrepareOp",
            ),
            (
                "extdeps.cloud.gcp.sts",
                "service_transport::parse::gcp.STS::Exchange",
                sts_exchange_metadata(),
                "GenericRestParseOp",
            ),
            (
                "extdeps.cloud.gcp.secret_manager",
                "service_transport::prepare::gcp.SecretManager::AccessVersion",
                secret_manager_metadata(),
                "GenericRestPrepareOp",
            ),
            (
                "extdeps.cloud.gcp.secret_manager",
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
            "tools.bootstrap",
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

    // NOTE: resolve_infra_callable_maps_to_infra_dispatch_op test moved to
    // gunbc-tests (requires GunbcExternResolver for tools.infra::infra dispatch).

    #[test]
    fn resolve_unknown_service_transport_prepare_fails() {
        let node = callable_node(
            "bad_service_prepare",
            "extdeps.cloud.gcp.sts",
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
            "tools.bootstrap",
            "render_clippy_toml",
            ObligationCategory::None,
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

        let resolved = resolve_lowered_dag_with(&dag).expect("release node should resolve");
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
        let op = ConditionalOp {
            output_port: "result".to_string(),
            has_else: false,
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
        let op = ConditionalOp {
            output_port: "result".to_string(),
            has_else: true,
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
        let op = BinaryOpOp {
            op: daglang_lower::expr::LoweredBinOp::Add,
            output_port: "result".to_string(),
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
        let op = RecordConstructOp {
            fields: vec!["x".to_string(), "y".to_string()],
            output_port: "result".to_string(),
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
}
