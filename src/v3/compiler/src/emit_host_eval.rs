//! T-22 substrate eval dispatch for `v4.compiler.emit_host` → `emit_host_runner`.
//!
//! **Rust row (`run_emit_host_rust`):** runs the runner, then assembles receipts via
//! `emit_host_receipt_from_source` / [`evaluator::eval_callable_declaration`] (substrate eval).
//!
//! **Python row (`run_emit_host_python`):** runs the runner and hand-reifies `v4.std.host_run`
//! carriers on this hook (substrate fn body stays `transport_not_wired`).
//!
//! **Go row (`run_emit_host_go`):** same substrate eval pattern as rust (`emit_host_receipt_from_source`).
//!
//! **P5:** SG-0 census in `sg0_census_test.rs` (T-PB-B); dissolution when substrate owns all rows.

use std::path::Path;

use crate::dag::{ArrowBody, AtomPayload, Dag, DeclarationId, LiteralBits, TypeConnective};
use crate::evaluator::{EvalError, EvalStateStack, EvalStrategy, NamedField, Value};
use emit_host_runner::{
    EmitHostRunReceipt, EmitHostTransportInputs, ExitWitness, HostExit, HostExitOutcome,
    HostLogicalFailure, HostSetupFailure,
};

#[cfg(test)]
use emit_host_runner::EmitHostFixtureInputs;

type HostTransportRun =
    fn(&str, &EmitHostTransportInputs, &Path) -> Result<EmitHostRunReceipt, HostSetupFailure>;

/// Modeled carrier authority for `v4.std.diagnostic` (`emit_host.dag` import graph).
const V4_STD_DIAGNOSTIC_AUTHORITY: &str = "src/v4/std/diagnostic.dag";
/// Modeled carrier authority for `v4.std.witness` (`host_run.dag` / `emit_host.dag` imports).
const V4_STD_WITNESS_AUTHORITY: &str = "src/v4/std/witness.dag";
/// `List<T> = FreeMonoid<T>` alias authority (`host_run.dag` import graph).
const V4_STD_COLLECTION_AUTHORITY: &str = "src/v4/std/collection.dag";
/// `FreeMonoid` `Empty`/`Cons` arm authority for list reification.
const V4_STD_ALGEBRA_AUTHORITY: &str = "src/v4/std/algebra.dag";
/// `Byte` / `List<Bit>` authority (`host_run` `ByteString = List<Byte>`).
const V4_STD_MACHINE_AUTHORITY: &str = "src/v4/std/machine.dag";
/// `String` authority (`host_run` `BuildLog.lines: List<String>`).
const V4_STD_TEXT_AUTHORITY: &str = "src/v4/std/text.dag";

/// Canonical substrate authority path for `run_emit_host_*` rows (see `src/v4/compiler/emit_host.dag`).
const EMIT_HOST_DAG_AUTHORITY_PATH: &str = "src/v4/compiler/emit_host.dag";

/// Eval-time dispatch for `run_emit_host_rust` (substrate `emit_host.dag` only).
pub fn try_dispatch_emit_host_rust(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Option<Result<Value, EvalError>> {
    if !is_run_emit_host_rust_decl(dag, callee_decl) {
        return None;
    }
    if operands.len() != 3 {
        return Some(Err(EvalError::TransformArityMismatch {
            expected: 3,
            got: operands.len(),
        }));
    }

    let source = match expect_string_operand(&operands[1]) {
        Ok(source) => source,
        Err(err) => return Some(Err(err)),
    };
    let inputs = match emit_host_transport_inputs(dag, &operands[2], state, strategy) {
        Ok(inputs) => inputs,
        Err(err) => return Some(Err(err)),
    };
    let work_dir = emit_host_runner::unique_work_dir("gunbc_eval_emit_host_rust");
    let receipt = match emit_host_runner::run_emit_host_rust(source, &inputs, &work_dir) {
        Ok(receipt) => receipt,
        Err(setup) => return Some(run_emit_host_setup_rejected(dag, &setup)),
    };
    let callee = match find_fn_decl(dag, "emit_host_receipt_from_source") {
        Ok(id) => id,
        Err(err) => return Some(Err(err)),
    };
    let exit = match host_exit_value(dag, &receipt.exit) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stdout = match byte_string_value(dag, &receipt.stdout_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stderr = match byte_string_value(dag, &receipt.stderr_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let build_log = match build_log_value(dag, &receipt.build_log.lines) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    Some(
        crate::evaluator::eval_callable_declaration(
            dag,
            callee,
            vec![
                operands[0].clone(),
                Value::LiteralValue(LiteralBits::String(receipt.source_text)),
                exit,
                stdout,
                stderr,
                build_log,
            ],
            state,
            strategy,
        )
        .and_then(|value| accepted_variant(dag, value)),
    )
}

/// Eval-time dispatch for `run_emit_host_go` (substrate `emit_host.dag` only).
pub fn try_dispatch_emit_host_go(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Option<Result<Value, EvalError>> {
    if !is_run_emit_host_go_decl(dag, callee_decl) {
        return None;
    }
    if operands.len() != 3 {
        return Some(Err(EvalError::TransformArityMismatch {
            expected: 3,
            got: operands.len(),
        }));
    }

    let source = match expect_string_operand(&operands[1]) {
        Ok(source) => source,
        Err(err) => return Some(Err(err)),
    };
    let inputs = match emit_host_transport_inputs(dag, &operands[2], state, strategy) {
        Ok(inputs) => inputs,
        Err(err) => return Some(Err(err)),
    };
    let work_dir = emit_host_runner::unique_work_dir("gunbc_eval_emit_host_go");
    let receipt = match emit_host_runner::run_emit_host_go(source, &inputs, &work_dir) {
        Ok(receipt) => receipt,
        Err(setup) => return Some(run_emit_host_setup_rejected(dag, &setup)),
    };
    let callee = match find_fn_decl(dag, "emit_host_receipt_from_source") {
        Ok(id) => id,
        Err(err) => return Some(Err(err)),
    };
    let exit = match host_exit_value(dag, &receipt.exit) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stdout = match byte_string_value(dag, &receipt.stdout_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stderr = match byte_string_value(dag, &receipt.stderr_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let build_log = match build_log_value(dag, &receipt.build_log.lines) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    Some(
        crate::evaluator::eval_callable_declaration(
            dag,
            callee,
            vec![
                operands[0].clone(),
                Value::LiteralValue(LiteralBits::String(receipt.source_text)),
                exit,
                stdout,
                stderr,
                build_log,
            ],
            state,
            strategy,
        )
        .and_then(|value| accepted_variant(dag, value)),
    )
}

/// B3 omni-emission — generic `run_host_process` substrate row (`emit_host.dag`).
pub fn try_dispatch_run_host_process(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Option<Result<Value, EvalError>> {
    if !is_run_host_process_decl(dag, callee_decl) {
        return None;
    }
    if operands.len() != 4 {
        return Some(Err(EvalError::TransformArityMismatch {
            expected: 4,
            got: operands.len(),
        }));
    }

    let descriptor_identity = match host_transport_descriptor_identity(&operands[0]) {
        Some(identity) => identity,
        None => {
            return Some(Err(EvalError::BadTransformOperands {
                reason: "run_host_process descriptor missing identity",
            }));
        }
    };
    let source = match expect_string_operand(&operands[2]) {
        Ok(source) => source,
        Err(err) => return Some(Err(err)),
    };
    let inputs = match emit_host_transport_inputs(dag, &operands[3], state, strategy) {
        Ok(inputs) => inputs,
        Err(err) => return Some(Err(err)),
    };
    let work_dir = emit_host_runner::unique_work_dir("gunbc_eval_run_host_process");
    let receipt = match emit_host_runner::run_host_process(
        &descriptor_identity,
        source,
        &inputs,
        &work_dir,
    ) {
        Ok(receipt) => receipt,
        Err(setup) => return Some(run_emit_host_setup_rejected(dag, &setup)),
    };
    let callee = match find_fn_decl(dag, "emit_host_receipt_from_source") {
        Ok(id) => id,
        Err(err) => return Some(Err(err)),
    };
    let exit = match host_exit_value(dag, &receipt.exit) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stdout = match byte_string_value(dag, &receipt.stdout_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let stderr = match byte_string_value(dag, &receipt.stderr_bytes) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    let build_log = match build_log_value(dag, &receipt.build_log.lines) {
        Ok(v) => v,
        Err(err) => return Some(Err(err)),
    };
    Some(
        crate::evaluator::eval_callable_declaration(
            dag,
            callee,
            vec![
                operands[1].clone(),
                Value::LiteralValue(LiteralBits::String(receipt.source_text)),
                exit,
                stdout,
                stderr,
                build_log,
            ],
            state,
            strategy,
        )
        .and_then(|value| accepted_variant(dag, value)),
    )
}

/// Decode `SignedI32Le` runtime primitive bytes (codec-driven, not target-specific).
pub fn try_dispatch_runtime_value_signed_i32_le_as_int(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    _state: &mut EvalStateStack<Value>,
    _strategy: &EvalStrategy,
) -> Option<Result<Value, EvalError>> {
    if !is_runtime_value_signed_i32_le_as_int_decl(dag, callee_decl) {
        return None;
    }
    if operands.len() != 1 {
        return Some(Err(EvalError::TransformArityMismatch {
            expected: 1,
            got: operands.len(),
        }));
    }
    let bytes = match runtime_value_primitive_bytes(dag, &operands[0]) {
        Ok(bytes) => bytes,
        Err(err) => return Some(Err(err)),
    };
    let parsed = match emit_host_runner::runtime_value_parse_signed_i32_le(&bytes) {
        Ok(n) => n,
        Err(_) => {
            return Some(Ok(rejected_outcome_variant(
                dag,
                non_empty_diagnostics_singleton(dag, emit_host_parse_failure_diagnostic(dag)?)?,
            )?));
        }
    };
    Some(Ok(accepted_variant(
        dag,
        Value::LiteralValue(LiteralBits::Int(parsed.to_string())),
    )?))
}

fn is_run_host_process_decl(dag: &Dag, callee_decl: DeclarationId) -> bool {
    is_emit_host_fn_decl(dag, callee_decl, "run_host_process", 4)
}

fn is_runtime_value_signed_i32_le_as_int_decl(dag: &Dag, callee_decl: DeclarationId) -> bool {
    is_emit_host_fn_decl(dag, callee_decl, "runtime_value_signed_i32_le_as_int", 1)
}

fn is_emit_host_fn_decl(
    dag: &Dag,
    callee_decl: DeclarationId,
    name: &str,
    arity: usize,
) -> bool {
    let Some(callee) = dag.declaration_opt(&callee_decl) else {
        return false;
    };
    if callee.name.as_deref() != Some(name) {
        return false;
    }
    if !callee.span.file.ends_with(EMIT_HOST_DAG_AUTHORITY_PATH) {
        return false;
    }
    let TypeConnective::Arrow { inputs, body, .. } = &callee.connective else {
        return false;
    };
    inputs.len() == arity && matches!(body, ArrowBody::UserDefined(_))
}

fn host_transport_descriptor_identity(value: &Value) -> Option<String> {
    record_fields(value).and_then(|fields| {
        fields
            .iter()
            .find(|f| f.label == "identity")
            .and_then(|f| atom_identity_literal(&f.value))
    })
}

fn runtime_value_primitive_bytes(dag: &Dag, value: &Value) -> Result<Vec<u8>, EvalError> {
    let payload = match value {
        Value::VariantValue { payload, .. } => payload.as_ref(),
        _ => {
            return Err(EvalError::BadTransformOperands {
                reason: "runtime value is not a variant",
            });
        }
    };
    let prim_fields = record_fields(payload).ok_or(EvalError::BadTransformOperands {
        reason: "RuntimePrimitive payload is not a record",
    })?;
    let value_field = prim_fields
        .iter()
        .find(|f| f.label == "value")
        .ok_or(EvalError::BadTransformOperands {
            reason: "RuntimePrimitiveValue missing value field",
        })?;
    let inner = record_fields(&value_field.value).ok_or(EvalError::BadTransformOperands {
        reason: "RuntimePrimitiveValue is not a record",
    })?;
    let bytes_field = inner
        .iter()
        .find(|f| f.label == "bytes")
        .ok_or(EvalError::BadTransformOperands {
            reason: "RuntimePrimitiveValue missing bytes field",
        })?;
    byte_list_to_vec(dag, &bytes_field.value)
}

fn byte_list_to_vec(dag: &Dag, list: &Value) -> Result<Vec<u8>, EvalError> {
    let byte_ty = v4_carrier_decl_id(dag, V4_STD_MACHINE_AUTHORITY, "Byte")?;
    let list_ty = list_instantiation_for_element(dag, byte_ty)?;
    let empty_tag = find_list_variant_tag(dag, list_ty, "Empty")?;
    let cons_tag = find_list_variant_tag(dag, list_ty, "Cons")?;
    let mut out = Vec::new();
    let mut current = list;
    loop {
        let Value::VariantValue { tag, payload } = current else {
            return Err(EvalError::BadTransformOperands {
                reason: "byte list is not a variant",
            });
        };
        if *tag == empty_tag {
            break;
        }
        if *tag != cons_tag {
            return Err(EvalError::BadTransformOperands {
                reason: "byte list variant is not Empty or Cons",
            });
        }
        let fields = record_fields(payload).ok_or(EvalError::BadTransformOperands {
            reason: "byte list Cons is not a record",
        })?;
        let head = fields
            .iter()
            .find(|f| f.label == "head")
            .ok_or(EvalError::BadTransformOperands {
                reason: "byte list Cons missing head",
            })?;
        out.push(byte_value_to_u8(dag, &head.value)?);
        let tail = fields
            .iter()
            .find(|f| f.label == "tail")
            .ok_or(EvalError::BadTransformOperands {
                reason: "byte list Cons missing tail",
            })?;
        current = &tail.value;
    }
    Ok(out)
}

fn byte_value_to_u8(dag: &Dag, value: &Value) -> Result<u8, EvalError> {
    let fields = record_fields(value).ok_or(EvalError::BadTransformOperands {
        reason: "byte is not a record",
    })?;
    let bits_value = fields
        .iter()
        .find(|f| f.label == "bits")
        .ok_or(EvalError::BadTransformOperands {
            reason: "byte missing bits field",
        })?;
    let bits = bit_list_to_bools(dag, &bits_value.value)?;
    if bits.len() != 8 {
        return Err(EvalError::BadTransformOperands {
            reason: "byte bits length is not 8",
        });
    }
    let mut byte = 0u8;
    for (shift, bit) in bits.iter().enumerate() {
        if *bit {
            byte |= 1 << (7 - shift);
        }
    }
    Ok(byte)
}

fn bit_list_to_bools(dag: &Dag, list: &Value) -> Result<Vec<bool>, EvalError> {
    let bit_ty = v4_carrier_decl_id(dag, V4_STD_MACHINE_AUTHORITY, "Bit")?;
    let list_ty = list_instantiation_for_element(dag, bit_ty)?;
    let empty_tag = find_list_variant_tag(dag, list_ty, "Empty")?;
    let cons_tag = find_list_variant_tag(dag, list_ty, "Cons")?;
    let mut out = Vec::new();
    let mut current = list;
    loop {
        let Value::VariantValue { tag, payload } = current else {
            return Err(EvalError::BadTransformOperands {
                reason: "bit list is not a variant",
            });
        };
        if *tag == empty_tag {
            break;
        }
        if *tag != cons_tag {
            return Err(EvalError::BadTransformOperands {
                reason: "bit list variant is not Empty or Cons",
            });
        }
        let fields = record_fields(payload).ok_or(EvalError::BadTransformOperands {
            reason: "bit list Cons is not a record",
        })?;
        let head = fields
            .iter()
            .find(|f| f.label == "head")
            .ok_or(EvalError::BadTransformOperands {
                reason: "bit list Cons missing head",
            })?;
        match &head.value {
            Value::LiteralValue(LiteralBits::Bool(bit)) => out.push(*bit),
            _ => {
                return Err(EvalError::BadTransformOperands {
                    reason: "bit list head is not Bool",
                });
            }
        }
        let tail = fields
            .iter()
            .find(|f| f.label == "tail")
            .ok_or(EvalError::BadTransformOperands {
                reason: "bit list Cons missing tail",
            })?;
        current = &tail.value;
    }
    Ok(out)
}

fn emit_host_parse_failure_diagnostic(dag: &Dag) -> Result<Value, EvalError> {
    emit_host_port_diagnostic(dag, "emit_host_parse_failure")
}

fn is_run_emit_host_rust_decl(dag: &Dag, callee_decl: DeclarationId) -> bool {
    is_run_emit_host_decl(dag, callee_decl, "run_emit_host_rust")
}

fn is_run_emit_host_go_decl(dag: &Dag, callee_decl: DeclarationId) -> bool {
    is_run_emit_host_decl(dag, callee_decl, "run_emit_host_go")
}

fn is_run_emit_host_decl(dag: &Dag, callee_decl: DeclarationId, name: &str) -> bool {
    let Some(callee) = dag.declaration_opt(&callee_decl) else {
        return false;
    };
    if callee.name.as_deref() != Some(name) {
        return false;
    }
    if !callee.span.file.ends_with(EMIT_HOST_DAG_AUTHORITY_PATH) {
        return false;
    }
    let TypeConnective::Arrow { inputs, body, .. } = &callee.connective else {
        return false;
    };
    inputs.len() == 3 && matches!(body, ArrowBody::UserDefined(_))
}

fn find_fn_decl(dag: &Dag, name: &str) -> Result<DeclarationId, EvalError> {
    dag.declarations()
        .iter()
        .find(|decl| {
            decl.name.as_deref() == Some(name)
                && decl.span.file.ends_with(EMIT_HOST_DAG_AUTHORITY_PATH)
        })
        .map(|decl| decl.id)
        .ok_or(EvalError::BadTransformOperands {
            reason: "substrate fn declaration not found in eval dag",
        })
}

/// Eval-time dispatch chain for remaining substrate `run_emit_host_*` rows.
///
/// Rust and Go use dedicated entrypoints in `lib.rs` (`emit_host_receipt_from_source`);
/// this wrapper keeps python on the same extension point if additional rows later share
/// the hand-reify transport path.
pub fn try_dispatch_emit_host(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Option<Result<Value, EvalError>> {
    if let Some(result) = try_dispatch_emit_host_python(dag, callee_decl, operands, state, strategy)
    {
        return Some(result);
    }
    None
}

/// Eval-time dispatch for `run_emit_host_python` in `emit_host.dag`.
pub fn try_dispatch_emit_host_python(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Option<Result<Value, EvalError>> {
    try_dispatch_emit_host_transport(
        dag,
        callee_decl,
        operands,
        state,
        strategy,
        "run_emit_host_python",
        "gunbc_eval_emit_host_python",
        emit_host_runner::run_emit_host_python,
    )
}

fn try_dispatch_emit_host_transport(
    dag: &Dag,
    callee_decl: DeclarationId,
    operands: &[Value],
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
    fn_name: &str,
    work_dir_prefix: &str,
    run: HostTransportRun,
) -> Option<Result<Value, EvalError>> {
    let callee = dag.declaration_opt(&callee_decl)?;
    if callee.name.as_deref() != Some(fn_name)
        || !callee.span.file.ends_with(EMIT_HOST_DAG_AUTHORITY_PATH)
    {
        return None;
    }
    if operands.len() != 3 {
        return Some(Err(EvalError::TransformArityMismatch {
            expected: 3,
            got: operands.len(),
        }));
    }

    let source = match expect_string_operand(&operands[1]) {
        Ok(source) => source,
        Err(err) => return Some(Err(err)),
    };
    let inputs = match emit_host_transport_inputs(dag, &operands[2], state, strategy) {
        Ok(inputs) => inputs,
        Err(err) => return Some(Err(err)),
    };
    let work_dir = emit_host_runner::unique_work_dir(work_dir_prefix);
    let receipt = match run(source, &inputs, &work_dir) {
        Ok(receipt) => receipt,
        Err(setup) => {
            return Some(run_emit_host_setup_rejected(dag, &setup));
        }
    };
    Some(
        emit_host_receipt_value(dag, &operands[0], receipt)
            .and_then(|value| accepted_variant(dag, value)),
    )
}

fn run_emit_host_setup_rejected(dag: &Dag, setup: &HostSetupFailure) -> Result<Value, EvalError> {
    rejected_outcome_variant(
        dag,
        non_empty_diagnostics_singleton(dag, host_setup_failure_diagnostic(dag, setup)?)?,
    )
}

fn expect_string_operand(value: &Value) -> Result<&str, EvalError> {
    match value {
        Value::LiteralValue(LiteralBits::String(s)) => Ok(s),
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected String operand",
        }),
    }
}

#[cfg(test)]
fn emit_host_fixture_inputs(
    dag: &Dag,
    value: &Value,
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Result<EmitHostFixtureInputs, EvalError> {
    let claim_root = inputs_root_field(value)?;
    let claim_pin_source =
        inputs_optional_present_node_field(dag, value, "host_claim_pin")?.unwrap_or(claim_root);
    let claim_pin = host_pin_from_inputs_root(dag, claim_pin_source, state, strategy)?;
    let expected_root = inputs_expected_eval_root_field(dag, value)?;
    let expected_pin = host_pin_from_inputs_root(dag, expected_root, state, strategy)?;
    Ok(EmitHostFixtureInputs {
        claim_input_root: claim_pin,
        expected_eval_root: expected_pin,
    })
}

fn inputs_root_field(inputs: &Value) -> Result<&Value, EvalError> {
    inputs_record_field(inputs, "root", "expected Inputs.root field")
}

#[cfg(test)]
fn inputs_expected_eval_root_field<'a>(
    dag: &Dag,
    inputs: &'a Value,
) -> Result<&'a Value, EvalError> {
    let optional = inputs_record_field(
        inputs,
        "expected_eval_root",
        "expected Inputs.expected_eval_root field",
    )?;
    optional_node_payload(dag, optional)?.ok_or(EvalError::BadTransformOperands {
        reason: "emit host dispatch requires Inputs.expected_eval_root Present",
    })
}

fn inputs_optional_present_node_field<'a>(
    dag: &Dag,
    inputs: &'a Value,
    label: &str,
) -> Result<Option<&'a Value>, EvalError> {
    let Some(optional) = inputs_optional_record_field(inputs, label)? else {
        return Ok(None);
    };
    optional_node_payload(dag, optional)
}

fn inputs_optional_record_field<'a>(
    inputs: &'a Value,
    label: &str,
) -> Result<Option<&'a Value>, EvalError> {
    match inputs {
        Value::RecordValue(fields) => Ok(fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)),
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected Inputs record",
        }),
    }
}

fn inputs_record_field<'a>(
    inputs: &'a Value,
    label: &str,
    missing_reason: &'static str,
) -> Result<&'a Value, EvalError> {
    match inputs {
        Value::RecordValue(fields) => fields
            .iter()
            .find(|field| field.label == label)
            .map(|field| &field.value)
            .ok_or(EvalError::BadTransformOperands {
                reason: missing_reason,
            }),
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected Inputs record",
        }),
    }
}

fn optional_node_payload<'a>(dag: &Dag, value: &'a Value) -> Result<Option<&'a Value>, EvalError> {
    let Value::VariantValue { tag, payload } = value else {
        return Err(EvalError::BadTransformOperands {
            reason: "expected Inputs optional field to be Optional variant",
        });
    };
    match variant_label_for_tag(dag, *tag).as_deref() {
        Some("Absent" | "None") => Ok(None),
        Some("Present" | "Some") => Ok(Some(optional_present_payload_value(payload)?)),
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected Inputs optional field to be Present or Absent",
        }),
    }
}

fn optional_present_payload_value<'a>(payload: &'a Value) -> Result<&'a Value, EvalError> {
    match payload {
        Value::RecordValue(fields) => fields
            .iter()
            .find(|field| field.label == "value")
            .map(|field| &field.value)
            .ok_or(EvalError::BadTransformOperands {
                reason: "expected Optional Present { value } payload",
            }),
        value => Ok(value),
    }
}

/// Project substrate `Node` to a non-empty runner pin (Symbol carrier, Atom identity, or `content_hash`).
fn host_pin_from_inputs_root(
    dag: &Dag,
    root: &Value,
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Result<String, EvalError> {
    let pin = match root {
        Value::LiteralValue(LiteralBits::String(s)) => s.clone(),
        node => match node_primary_symbol(dag, node) {
            Some(sym) => sym,
            None => content_hash_runner_pin(dag, node, state, strategy)?,
        },
    };
    if pin.is_empty() {
        return Err(EvalError::BadTransformOperands {
            reason: "Inputs pin Node projected to empty host runner pin",
        });
    }
    Ok(pin)
}

fn content_hash_runner_pin(
    dag: &Dag,
    node: &Value,
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Result<String, EvalError> {
    let hash_value = eval_substrate_fn1(dag, "content_hash", node.clone(), state, strategy)?;
    hash_value_to_runner_pin(dag, &hash_value)
}

fn eval_substrate_fn1(
    dag: &Dag,
    name: &str,
    arg: Value,
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Result<Value, EvalError> {
    let decl = dag
        .declaration_by_name(name)
        .ok_or(EvalError::BadTransformOperands {
            reason: "substrate fn declaration not found in eval dag",
        })?;
    crate::evaluator::eval_callable_declaration(dag, decl.id, vec![arg], state, strategy)
}

fn hash_value_to_runner_pin(dag: &Dag, hash: &Value) -> Result<String, EvalError> {
    if let Some(sym) = node_primary_symbol(dag, hash) {
        return Ok(sym);
    }
    match hash {
        Value::LiteralValue(LiteralBits::String(s)) => Ok(s.clone()),
        Value::LiteralValue(LiteralBits::Int(s)) => Ok(s.clone()),
        _ => Err(EvalError::BadTransformOperands {
            reason: "content_hash result not projectable to host runner pin",
        }),
    }
}

fn variant_label_for_tag(dag: &Dag, tag: DeclarationId) -> Option<String> {
    dag.declarations().iter().find_map(|decl| {
        let TypeConnective::Disj { variants } = &decl.connective else {
            return None;
        };
        variants
            .iter()
            .find(|variant| variant.ty == tag)
            .map(|variant| variant.label.clone())
    })
}

fn node_primary_symbol(dag: &Dag, value: &Value) -> Option<String> {
    let fields = record_fields(value)?;
    let kind = fields.iter().find(|f| f.label == "kind")?;
    connective_atom_symbol(dag, &kind.value)
}

fn connective_atom_symbol(dag: &Dag, value: &Value) -> Option<String> {
    match value {
        Value::RecordValue(fields) => fields
            .iter()
            .find(|f| f.label == "connective")
            .and_then(|f| atom_identity_literal(&f.value)),
        Value::VariantValue { tag, payload } => {
            let arm = dag.declaration(*tag).name.as_deref()?;
            if arm == "Atom" {
                atom_identity_literal(payload)
            } else {
                connective_atom_symbol(dag, payload)
            }
        }
        _ => None,
    }
}

fn atom_identity_literal(value: &Value) -> Option<String> {
    match value {
        Value::RecordValue(fields) => {
            fields
                .iter()
                .find(|f| f.label == "identity")
                .and_then(|f| match &f.value {
                    Value::LiteralValue(LiteralBits::String(s)) => Some(s.clone()),
                    _ => None,
                })
        }
        Value::VariantValue { payload, .. } => atom_identity_literal(payload),
        _ => None,
    }
}

fn record_fields(value: &Value) -> Option<&[NamedField]> {
    match value {
        Value::RecordValue(fields) => Some(fields),
        Value::VariantValue { payload, .. } => record_fields(payload),
        _ => None,
    }
}

/// Build claim-only transport pins for `run_emit_host_*` substrate rows.
///
/// Operand `Inputs` at this boundary carries only `root` (claim pin). Expected eval root is
/// modeled separately on emit-vs-eval harness paths via [`emit_host_fixture_inputs`].
fn emit_host_transport_inputs(
    dag: &Dag,
    fixture_inputs: &Value,
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Result<EmitHostTransportInputs, EvalError> {
    let claim_root = inputs_root_field(fixture_inputs)?;
    let claim_pin_source =
        inputs_optional_present_node_field(dag, fixture_inputs, "host_claim_pin")?
            .unwrap_or(claim_root);
    let claim_input_root = host_pin_from_inputs_root(dag, claim_pin_source, state, strategy)?;
    Ok(EmitHostTransportInputs { claim_input_root })
}

/// Resolve a top-level v4 std carrier by **source file** (not `declaration_by_name` rank).
///
/// Bootstrap `Dag::new()` also embeds `src/v3/std/dimensions.dag` `Witness<Carrier>`
/// (`Inhabits` / `Violates`). Host-run reification must use `v4.std.witness.Witness`
/// (`Holds` / `Violates`) from the emit-host import graph.
///
/// When several declarations in the same authority file share `type_name` (alias re-lowers,
/// specialization copies), pick the one with the **lowest** `DeclarationId::raw()` so lookup
/// is deterministic and stable across hermetic stubs and `compile_to_dag_modules_in_order`.
fn v4_carrier_decl_id(
    dag: &Dag,
    authority_file_suffix: &str,
    type_name: &str,
) -> Result<DeclarationId, EvalError> {
    dag.declarations()
        .iter()
        .filter(|decl| {
            decl.name.as_deref() == Some(type_name)
                && decl.span.file.ends_with(authority_file_suffix)
        })
        .max_by_key(|decl| std::cmp::Reverse(decl.id.raw()))
        .map(|decl| decl.id)
        .ok_or(EvalError::BadTransformOperands {
            reason: "v4 carrier type not found in modeled authority file",
        })
}

fn v4_carrier_variant_tag(
    dag: &Dag,
    authority_file_suffix: &str,
    type_name: &str,
    variant_name: &str,
) -> Result<DeclarationId, EvalError> {
    let decl_id = v4_carrier_decl_id(dag, authority_file_suffix, type_name)?;
    let TypeConnective::Disj { variants } = &dag.declaration(decl_id).connective else {
        return Err(EvalError::BadTransformOperands {
            reason: "v4 carrier is not a sum type",
        });
    };
    variants
        .iter()
        .find(|variant| variant.label == variant_name)
        .map(|variant| variant.ty)
        .ok_or(EvalError::BadTransformOperands {
            reason: "v4 carrier variant arm not found in modeled authority file",
        })
}

fn is_v4_std_authority_file(file: &str) -> bool {
    file.contains("src/v4/std/")
}

fn list_instantiation_for_element(
    dag: &Dag,
    element_ty: DeclarationId,
) -> Result<DeclarationId, EvalError> {
    let list_id = v4_carrier_decl_id(dag, V4_STD_COLLECTION_AUTHORITY, "List")?;
    dag.declarations()
        .iter()
        .filter(|decl| is_v4_std_authority_file(&decl.span.file))
        .find_map(|decl| {
            let TypeConnective::Instantiation {
                template,
                arguments,
            } = &decl.connective
            else {
                return None;
            };
            (*template == list_id && arguments.len() == 1 && arguments[0].value == element_ty)
                .then_some(decl.id)
        })
        .ok_or(EvalError::BadTransformOperands {
            reason: "v4 List<element> instantiation not found in modeled authority files",
        })
}

fn list_element_ty(dag: &Dag, list_ty: DeclarationId) -> Result<DeclarationId, EvalError> {
    let list_id = v4_carrier_decl_id(dag, V4_STD_COLLECTION_AUTHORITY, "List")?;
    let free_monoid_id = v4_carrier_decl_id(dag, V4_STD_ALGEBRA_AUTHORITY, "FreeMonoid").ok();
    peel_list_element_ty(dag, list_ty, list_id, free_monoid_id, 0)
}

fn peel_list_element_ty(
    dag: &Dag,
    ty: DeclarationId,
    list_id: DeclarationId,
    free_monoid_id: Option<DeclarationId>,
    depth: usize,
) -> Result<DeclarationId, EvalError> {
    if depth >= 8 {
        return Err(EvalError::BadTransformOperands {
            reason: "List<elem> alias peel depth exceeded",
        });
    }
    match &dag.declaration(ty).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if arguments.len() == 1 => {
            if *template == list_id || free_monoid_id == Some(*template) {
                Ok(arguments[0].value)
            } else {
                peel_list_element_ty(dag, *template, list_id, free_monoid_id, depth + 1)
            }
        }
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            peel_list_element_ty(dag, *next, list_id, free_monoid_id, depth + 1)
        }
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected List<elem> instantiation",
        }),
    }
}

/// `Empty` / `Cons` arm type from `v4.std.algebra.FreeMonoid` (not bootstrap `List`).
fn v4_free_monoid_variant_arm_type(dag: &Dag, variant: &str) -> Result<DeclarationId, EvalError> {
    v4_carrier_variant_tag(dag, V4_STD_ALGEBRA_AUTHORITY, "FreeMonoid", variant)
}

fn find_list_variant_tag(
    dag: &Dag,
    list_ty: DeclarationId,
    variant: &str,
) -> Result<DeclarationId, EvalError> {
    let elem_ty = list_element_ty(dag, list_ty)?;
    let arm_ty = v4_free_monoid_variant_arm_type(dag, variant)?;
    dag.declarations()
        .iter()
        .filter(|decl| is_v4_std_authority_file(&decl.span.file))
        .find_map(|decl| {
            let TypeConnective::Instantiation {
                template,
                arguments,
            } = &decl.connective
            else {
                return None;
            };
            (*template == arm_ty && arguments.len() == 1 && arguments[0].value == elem_ty)
                .then_some(decl.id)
        })
        .ok_or(EvalError::BadTransformOperands {
            reason: "v4 FreeMonoid variant constructor tag not found in modeled authority files",
        })
}

fn empty_list_value(dag: &Dag, list_ty: DeclarationId) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: find_list_variant_tag(dag, list_ty, "Empty")?,
        payload: Box::new(Value::RecordValue(vec![])),
    })
}

fn cons_list_value(
    dag: &Dag,
    list_ty: DeclarationId,
    head: Value,
    tail: Value,
) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: find_list_variant_tag(dag, list_ty, "Cons")?,
        payload: Box::new(Value::RecordValue(vec![
            NamedField {
                label: "head".to_string(),
                value: head,
            },
            NamedField {
                label: "tail".to_string(),
                value: tail,
            },
        ])),
    })
}

fn list_from_values(
    dag: &Dag,
    list_ty: DeclarationId,
    items: Vec<Value>,
) -> Result<Value, EvalError> {
    let mut list = empty_list_value(dag, list_ty)?;
    for item in items.into_iter().rev() {
        list = cons_list_value(dag, list_ty, item, list)?;
    }
    Ok(list)
}

fn bit_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let bit_ty = v4_carrier_decl_id(dag, V4_STD_MACHINE_AUTHORITY, "Bit")?;
    list_instantiation_for_element(dag, bit_ty)
}

fn byte_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let byte_ty = v4_carrier_decl_id(dag, V4_STD_MACHINE_AUTHORITY, "Byte")?;
    list_instantiation_for_element(dag, byte_ty)
}

fn string_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let string_ty = v4_carrier_decl_id(dag, V4_STD_TEXT_AUTHORITY, "String")?;
    list_instantiation_for_element(dag, string_ty)
}

fn byte_value(dag: &Dag, byte: u8) -> Result<Value, EvalError> {
    let bits: Vec<Value> = (0..8)
        .map(|shift| Value::LiteralValue(LiteralBits::Bool((byte >> (7 - shift)) & 1 != 0)))
        .collect();
    let bits_list = list_from_values(dag, bit_list_ty(dag)?, bits)?;
    Ok(Value::RecordValue(vec![NamedField {
        label: "bits".to_string(),
        value: bits_list,
    }]))
}

fn byte_string_value(dag: &Dag, bytes: &[u8]) -> Result<Value, EvalError> {
    let elems: Result<Vec<Value>, EvalError> =
        bytes.iter().copied().map(|b| byte_value(dag, b)).collect();
    list_from_values(dag, byte_list_ty(dag)?, elems?)
}

fn string_list_value(dag: &Dag, lines: &[String]) -> Result<Value, EvalError> {
    let elems: Vec<Value> = lines
        .iter()
        .cloned()
        .map(|line| Value::LiteralValue(LiteralBits::String(line)))
        .collect();
    list_from_values(dag, string_list_ty(dag)?, elems)
}

fn build_log_value(dag: &Dag, lines: &[String]) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![NamedField {
        label: "lines".to_string(),
        value: string_list_value(dag, lines)?,
    }]))
}

fn diagnostics_none_variant(dag: &Dag) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: v4_carrier_variant_tag(dag, V4_STD_DIAGNOSTIC_AUTHORITY, "Diagnostics", "None")?,
        payload: Box::new(Value::RecordValue(Vec::new())),
    })
}

fn accepted_variant(dag: &Dag, value: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: v4_carrier_variant_tag(dag, V4_STD_DIAGNOSTIC_AUTHORITY, "Outcome", "Accepted")?,
        payload: Box::new(Value::RecordValue(vec![
            NamedField {
                label: "value".to_string(),
                value,
            },
            NamedField {
                label: "diagnostics".to_string(),
                value: diagnostics_none_variant(dag)?,
            },
        ])),
    })
}

fn rejected_outcome_variant(dag: &Dag, diagnostics: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: v4_carrier_variant_tag(dag, V4_STD_DIAGNOSTIC_AUTHORITY, "Outcome", "Rejected")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "diagnostics".to_string(),
            value: diagnostics,
        }])),
    })
}

fn diagnostic_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let diagnostic_ty = v4_carrier_decl_id(dag, V4_STD_DIAGNOSTIC_AUTHORITY, "Diagnostic")?;
    list_instantiation_for_element(dag, diagnostic_ty)
}

fn non_empty_diagnostics_singleton(dag: &Dag, diagnostic: Value) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![
        NamedField {
            label: "head".to_string(),
            value: diagnostic,
        },
        NamedField {
            label: "tail".to_string(),
            value: empty_list_value(dag, diagnostic_list_ty(dag)?)?,
        },
    ]))
}

fn correction_unavailable_variant(dag: &Dag) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: v4_carrier_variant_tag(
            dag,
            V4_STD_DIAGNOSTIC_AUTHORITY,
            "Correction",
            "Unavailable",
        )?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "reason".to_string(),
            value: Value::VariantValue {
                tag: v4_carrier_variant_tag(
                    dag,
                    V4_STD_DIAGNOSTIC_AUTHORITY,
                    "NoCorrectionReason",
                    "ExternalContractUnknown",
                )?,
                payload: Box::new(Value::RecordValue(vec![])),
            },
        }])),
    })
}

fn port_locus_value(dag: &Dag, port: &str) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: v4_carrier_variant_tag(dag, V4_STD_DIAGNOSTIC_AUTHORITY, "Locus", "PortLocus")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "anchor".to_string(),
            value: Value::RecordValue(vec![NamedField {
                label: "at".to_string(),
                value: Value::LiteralValue(LiteralBits::String(port.to_string())),
            }]),
        }])),
    })
}

fn emit_host_port_diagnostic(dag: &Dag, reason: &str) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![
        NamedField {
            label: "reason".to_string(),
            value: Value::LiteralValue(LiteralBits::String(reason.to_string())),
        },
        NamedField {
            label: "at".to_string(),
            value: port_locus_value(dag, "emit_host_transport_port")?,
        },
        NamedField {
            label: "correction".to_string(),
            value: correction_unavailable_variant(dag)?,
        },
    ]))
}

fn host_logical_failure_diagnostic(
    dag: &Dag,
    failure: &HostLogicalFailure,
) -> Result<Value, EvalError> {
    let reason = match failure {
        HostLogicalFailure::TimedOut { .. } => "emit_host_exit_timed_out",
        HostLogicalFailure::NoExitStatus { .. } => "emit_host_exit_no_status",
        HostLogicalFailure::ExitedNonzero { .. } => "emit_host_exit_nonzero",
    };
    emit_host_port_diagnostic(dag, reason)
}

fn host_setup_failure_reason(failure: &HostSetupFailure) -> &'static str {
    match failure {
        HostSetupFailure::SpawnFailed { .. } => "emit_host_setup_spawn_failed",
        HostSetupFailure::StdoutPipeMissing { .. } => "emit_host_setup_stdout_pipe_missing",
        HostSetupFailure::StderrPipeMissing { .. } => "emit_host_setup_stderr_pipe_missing",
        HostSetupFailure::TryWaitFailed { .. } => "emit_host_setup_try_wait_failed",
        HostSetupFailure::StreamReadFailed { .. } => "emit_host_setup_stream_read_failed",
        HostSetupFailure::WorkDirCreateFailed { .. } => "emit_host_setup_work_dir_create_failed",
        HostSetupFailure::ManifestWriteFailed { .. } => "emit_host_setup_manifest_write_failed",
        HostSetupFailure::SourceWriteFailed { .. } => "emit_host_setup_source_write_failed",
        HostSetupFailure::EmptyClaimInputRoot => "emit_host_setup_empty_claim_input_root",
        HostSetupFailure::EmptyExpectedEvalRoot => "emit_host_setup_empty_expected_eval_root",
    }
}

fn host_setup_failure_diagnostic(
    dag: &Dag,
    failure: &HostSetupFailure,
) -> Result<Value, EvalError> {
    emit_host_port_diagnostic(dag, host_setup_failure_reason(failure))
}

fn witness_holds_variant(dag: &Dag, value: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: v4_carrier_variant_tag(dag, V4_STD_WITNESS_AUTHORITY, "Witness", "Holds")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "value".to_string(),
            value,
        }])),
    })
}

fn witness_violates_variant(dag: &Dag, diagnostic: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: v4_carrier_variant_tag(dag, V4_STD_WITNESS_AUTHORITY, "Witness", "Violates")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "diagnostic".to_string(),
            value: diagnostic,
        }])),
    })
}

fn host_exit_value(dag: &Dag, exit: &HostExit) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![NamedField {
        label: "outcome".to_string(),
        value: host_exit_outcome_value(dag, exit)?,
    }]))
}

fn host_exit_outcome_value(dag: &Dag, exit: &HostExit) -> Result<Value, EvalError> {
    match &exit.outcome {
        HostExitOutcome::Accepted(ExitWitness::Holds(ok)) => accepted_variant(
            dag,
            witness_holds_variant(
                dag,
                Value::RecordValue(vec![NamedField {
                    label: "code".to_string(),
                    value: Value::LiteralValue(LiteralBits::Int(ok.code.to_string())),
                }]),
            )?,
        ),
        HostExitOutcome::Accepted(ExitWitness::Violates(failure)) => accepted_variant(
            dag,
            witness_violates_variant(dag, host_logical_failure_diagnostic(dag, failure)?)?,
        ),
        HostExitOutcome::Rejected(setup) => rejected_outcome_variant(
            dag,
            non_empty_diagnostics_singleton(dag, host_setup_failure_diagnostic(dag, setup)?)?,
        ),
    }
}

/// Mirrors `host_logical_run_from_exit` in `v4.std.host_run`.
fn host_logical_run_outcome_value(
    dag: &Dag,
    exit: &HostExit,
    stdout_bytes: &[u8],
) -> Result<Value, EvalError> {
    match &exit.outcome {
        HostExitOutcome::Accepted(ExitWitness::Holds(_)) => {
            let stdout = Value::RecordValue(vec![NamedField {
                label: "bytes".to_string(),
                value: byte_string_value(dag, stdout_bytes)?,
            }]);
            accepted_variant(
                dag,
                Value::RecordValue(vec![NamedField {
                    label: "stdout".to_string(),
                    value: stdout,
                }]),
            )
        }
        HostExitOutcome::Accepted(ExitWitness::Violates(failure)) => rejected_outcome_variant(
            dag,
            non_empty_diagnostics_singleton(dag, host_logical_failure_diagnostic(dag, failure)?)?,
        ),
        HostExitOutcome::Rejected(setup) => rejected_outcome_variant(
            dag,
            non_empty_diagnostics_singleton(dag, host_setup_failure_diagnostic(dag, setup)?)?,
        ),
    }
}

fn emit_host_receipt_value(
    dag: &Dag,
    target: &Value,
    receipt: EmitHostRunReceipt,
) -> Result<Value, EvalError> {
    let exit = host_exit_outcome_value(dag, &receipt.exit)?;
    let logical_run = host_logical_run_outcome_value(dag, &receipt.exit, &receipt.stdout_bytes)?;
    Ok(Value::RecordValue(vec![
        NamedField {
            label: "target".to_string(),
            value: target.clone(),
        },
        NamedField {
            label: "source_text".to_string(),
            value: Value::LiteralValue(LiteralBits::String(receipt.source_text)),
        },
        NamedField {
            label: "exit".to_string(),
            value: Value::RecordValue(vec![NamedField {
                label: "outcome".to_string(),
                value: exit,
            }]),
        },
        NamedField {
            label: "logical_run".to_string(),
            value: logical_run,
        },
        NamedField {
            label: "stderr_bytes".to_string(),
            value: byte_string_value(dag, &receipt.stderr_bytes)?,
        },
        NamedField {
            label: "build_log".to_string(),
            value: Value::RecordValue(vec![NamedField {
                label: "lines".to_string(),
                value: string_list_value(dag, &receipt.build_log.lines)?,
            }]),
        },
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{
        ArrowBody, AtomPayload, BindNodeId, Dag, Declaration, DeclarationId, Field,
        TemplateArgument, TypeConnective,
    };
    use crate::diagnostics::SourceSpan;
    use crate::evaluator::{EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder};
    use emit_host_runner::HostSetupFailure;

    fn optional_variant(dag: &Dag, name: &str, payload: Value) -> Value {
        let tag = dag
            .declarations()
            .iter()
            .find_map(|decl| {
                let TypeConnective::Disj { variants } = &decl.connective else {
                    return None;
                };
                variants
                    .iter()
                    .find(|variant| variant.label == name)
                    .map(|variant| variant.ty)
            })
            .expect(name);
        Value::VariantValue {
            tag,
            payload: Box::new(payload),
        }
    }

    fn optional_present_value(dag: &Dag, value: Value) -> Value {
        optional_variant(dag, "Some", value)
    }

    fn span(file: &str) -> SourceSpan {
        SourceSpan::new(file, 0, 1)
    }

    fn atom_decl(dag: &mut Dag, name: &str, file: &str) -> DeclarationId {
        let id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id,
            name: Some(name.to_string()),
            connective: TypeConnective::Atom(AtomPayload::TypeParam(name.to_string())),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span(file),
        });
        id
    }

    fn run_emit_host_decl(dag: &mut Dag, name: &str, file: &str) -> DeclarationId {
        let target = atom_decl(dag, "TargetModel", file);
        let source = atom_decl(dag, "TargetSource", file);
        let inputs = atom_decl(dag, "Inputs", file);
        let output = atom_decl(dag, "OutcomeEmitHostRunReceipt", file);
        let body_value = dag.push_value(LiteralBits::Bool(false), span(file));
        let bind = dag.push_bind(
            "transport_not_wired_body",
            body_value,
            Vec::new(),
            span(file),
        );
        let id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id,
            name: Some(name.to_string()),
            connective: TypeConnective::Arrow {
                inputs: vec![target, source, inputs],
                output,
                body: ArrowBody::UserDefined(
                    BindNodeId::from_bind_node(dag, bind).expect("bind body"),
                ),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span(file),
        });
        id
    }

    fn empty_eval_state() -> EvalStateStack<Value> {
        EvalStateStack::with_root_frame(EvalFrame::empty())
    }

    fn applicative_strategy() -> EvalStrategy {
        EvalStrategy::ApplicativeOrder {
            input_order: InputEvaluationOrder::LeftFirst,
        }
    }

    const V4_DIAGNOSTIC_PATH: &str = "src/v4/std/diagnostic.dag";
    const V4_WITNESS_PATH: &str = "src/v4/std/witness.dag";
    const V4_COLLECTION_PATH: &str = "src/v4/std/collection.dag";
    const V4_ALGEBRA_PATH: &str = "src/v4/std/algebra.dag";
    const V4_MACHINE_PATH: &str = "src/v4/std/machine.dag";
    const V4_TEXT_PATH: &str = "src/v4/std/text.dag";

    fn push_named_decl(
        dag: &mut Dag,
        name: &str,
        connective: TypeConnective,
        file: &str,
    ) -> DeclarationId {
        let id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id,
            name: Some(name.to_string()),
            connective,
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span(file),
        });
        id
    }

    fn push_anonymous_decl(dag: &mut Dag, connective: TypeConnective, file: &str) -> DeclarationId {
        let id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id,
            name: None,
            connective,
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span(file),
        });
        id
    }

    fn push_disj_type(
        dag: &mut Dag,
        name: &str,
        file: &str,
        variants: &[(&str, DeclarationId)],
    ) -> DeclarationId {
        push_named_decl(
            dag,
            name,
            TypeConnective::Disj {
                variants: variants
                    .iter()
                    .map(|(label, ty)| Field {
                        label: (*label).to_string(),
                        ty: *ty,
                    })
                    .collect(),
            },
            file,
        )
    }

    fn push_conj_type(
        dag: &mut Dag,
        name: &str,
        file: &str,
        fields: &[(&str, DeclarationId)],
    ) -> DeclarationId {
        push_named_decl(
            dag,
            name,
            TypeConnective::Conj {
                children: fields
                    .iter()
                    .map(|(label, ty)| Field {
                        label: (*label).to_string(),
                        ty: *ty,
                    })
                    .collect(),
            },
            file,
        )
    }

    fn push_list_instantiation(
        dag: &mut Dag,
        list_template: DeclarationId,
        elem: DeclarationId,
        file: &str,
    ) -> DeclarationId {
        let id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id,
            name: None,
            connective: TypeConnective::Instantiation {
                template: list_template,
                arguments: vec![TemplateArgument {
                    parameter: list_template,
                    value: elem,
                }],
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: span(file),
        });
        id
    }

    fn wire_free_monoid_variant_constructors(
        dag: &mut Dag,
        free_monoid: DeclarationId,
        elem: DeclarationId,
        file: &str,
    ) {
        let TypeConnective::Disj { variants } = dag.declaration(free_monoid).connective.clone()
        else {
            panic!("FreeMonoid must be a sum type");
        };
        for label in ["Empty", "Cons"] {
            let arm_ty = variants
                .iter()
                .find(|variant| variant.label == label)
                .expect(label)
                .ty;
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Instantiation {
                    template: arm_ty,
                    arguments: vec![TemplateArgument {
                        parameter: arm_ty,
                        value: elem,
                    }],
                },
                type_params: Vec::new(),
                phantom_params: Vec::new(),
                meta_tag: None,
                specialization_parent: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                nominal_opacity: None,
                span: span(file),
            });
        }
    }

    /// Hermetic v4 std carrier stubs (authority-qualified by `span.file` suffix).
    ///
    /// Avoids `compile_to_dag_modules_in_order` on `diagnostic.dag` fn bodies (R3 Gap 9) while
    /// exercising the same `v4_carrier_*` lookup paths used by python eval dispatch reification.
    fn dag_with_hermetic_v4_emit_host_carriers() -> Dag {
        let mut dag = Dag::new();
        let bit_ty = push_named_decl(
            &mut dag,
            "Bit",
            TypeConnective::Atom(AtomPayload::TypeParam("Bit".to_string())),
            V4_MACHINE_PATH,
        );
        let symbol_ty = push_named_decl(
            &mut dag,
            "Symbol",
            TypeConnective::Atom(AtomPayload::TypeParam("Symbol".to_string())),
            V4_DIAGNOSTIC_PATH,
        );
        let node_ty = push_named_decl(
            &mut dag,
            "Node",
            TypeConnective::Atom(AtomPayload::TypeParam("Node".to_string())),
            V4_DIAGNOSTIC_PATH,
        );
        let _diagnostic_ty = push_conj_type(
            &mut dag,
            "Diagnostic",
            V4_DIAGNOSTIC_PATH,
            &[
                ("reason", symbol_ty),
                ("at", node_ty),
                ("correction", node_ty),
            ],
        );
        let external_contract_unknown = push_anonymous_decl(
            &mut dag,
            TypeConnective::Atom(AtomPayload::TypeParam(
                "ExternalContractUnknown".to_string(),
            )),
            V4_DIAGNOSTIC_PATH,
        );
        let _no_correction_reason = push_disj_type(
            &mut dag,
            "NoCorrectionReason",
            V4_DIAGNOSTIC_PATH,
            &[("ExternalContractUnknown", external_contract_unknown)],
        );
        let _correction = push_disj_type(
            &mut dag,
            "Correction",
            V4_DIAGNOSTIC_PATH,
            &[("Unavailable", external_contract_unknown)],
        );
        let _locus = push_disj_type(
            &mut dag,
            "Locus",
            V4_DIAGNOSTIC_PATH,
            &[("PortLocus", external_contract_unknown)],
        );
        let _diagnostics = push_disj_type(
            &mut dag,
            "Diagnostics",
            V4_DIAGNOSTIC_PATH,
            &[("None", external_contract_unknown)],
        );
        let _outcome = push_disj_type(
            &mut dag,
            "Outcome",
            V4_DIAGNOSTIC_PATH,
            &[
                ("Accepted", external_contract_unknown),
                ("Rejected", external_contract_unknown),
            ],
        );
        let holds_payload = push_conj_type(
            &mut dag,
            "WitnessHolds",
            V4_WITNESS_PATH,
            &[("value", external_contract_unknown)],
        );
        let violates_payload = push_conj_type(
            &mut dag,
            "WitnessViolates",
            V4_WITNESS_PATH,
            &[("diagnostic", external_contract_unknown)],
        );
        push_disj_type(
            &mut dag,
            "Witness",
            V4_WITNESS_PATH,
            &[("Holds", holds_payload), ("Violates", violates_payload)],
        );
        let byte_ty = push_named_decl(
            &mut dag,
            "Byte",
            TypeConnective::Atom(AtomPayload::TypeParam("Byte".to_string())),
            V4_MACHINE_PATH,
        );
        let string_ty = push_named_decl(
            &mut dag,
            "String",
            TypeConnective::Atom(AtomPayload::TypeParam("String".to_string())),
            V4_TEXT_PATH,
        );
        let empty_arm = push_anonymous_decl(
            &mut dag,
            TypeConnective::Atom(AtomPayload::TypeParam("FreeMonoidEmpty".to_string())),
            V4_ALGEBRA_PATH,
        );
        let elem_param = push_anonymous_decl(
            &mut dag,
            TypeConnective::Atom(AtomPayload::TypeParam("T".to_string())),
            V4_ALGEBRA_PATH,
        );
        let tail_param = push_anonymous_decl(
            &mut dag,
            TypeConnective::Atom(AtomPayload::TypeParam("FreeMonoidTail".to_string())),
            V4_ALGEBRA_PATH,
        );
        let cons_payload = push_conj_type(
            &mut dag,
            "FreeMonoidCons",
            V4_ALGEBRA_PATH,
            &[("head", elem_param), ("tail", tail_param)],
        );
        let free_monoid = push_disj_type(
            &mut dag,
            "FreeMonoid",
            V4_ALGEBRA_PATH,
            &[("Empty", empty_arm), ("Cons", cons_payload)],
        );
        let list_template = push_named_decl(
            &mut dag,
            "List",
            TypeConnective::Atom(AtomPayload::TypeParam("List".to_string())),
            V4_COLLECTION_PATH,
        );
        let diagnostic_ty = dag
            .declarations()
            .iter()
            .find(|decl| {
                decl.name.as_deref() == Some("Diagnostic")
                    && decl.span.file.ends_with(V4_DIAGNOSTIC_PATH)
            })
            .expect("Diagnostic stub")
            .id;
        for elem in [bit_ty, byte_ty, string_ty, diagnostic_ty] {
            push_list_instantiation(&mut dag, list_template, elem, V4_COLLECTION_PATH);
            wire_free_monoid_variant_constructors(&mut dag, free_monoid, elem, V4_ALGEBRA_PATH);
        }
        dag
    }

    fn dag_with_v4_emit_host_carrier_authorities() -> Dag {
        dag_with_hermetic_v4_emit_host_carriers()
    }

    #[test]
    fn emit_host_fixture_inputs_projects_distinct_claim_and_expected_roots() {
        let dag = Dag::new();
        let inputs = Value::RecordValue(vec![
            NamedField {
                label: "root".to_string(),
                value: Value::LiteralValue(LiteralBits::String("claim_pin".to_string())),
            },
            NamedField {
                label: "expected_eval_root".to_string(),
                value: optional_present_value(
                    &dag,
                    Value::LiteralValue(LiteralBits::String("expected_eval_pin".to_string())),
                ),
            },
        ]);
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        let pins =
            emit_host_fixture_inputs(&dag, &inputs, &mut state, &strategy).expect("Inputs pins");
        assert_eq!(pins.claim_input_root, "claim_pin");
        assert_eq!(pins.expected_eval_root, "expected_eval_pin");
    }

    #[test]
    fn emit_host_fixture_inputs_uses_present_host_claim_pin() {
        let dag = Dag::new();
        let inputs = Value::RecordValue(vec![
            NamedField {
                label: "root".to_string(),
                value: Value::LiteralValue(LiteralBits::String("eval_root_pin".to_string())),
            },
            NamedField {
                label: "expected_eval_root".to_string(),
                value: optional_present_value(
                    &dag,
                    Value::LiteralValue(LiteralBits::String("expected_eval_pin".to_string())),
                ),
            },
            NamedField {
                label: "host_claim_pin".to_string(),
                value: optional_present_value(
                    &dag,
                    Value::LiteralValue(LiteralBits::String("host_claim_pin".to_string())),
                ),
            },
        ]);
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        let pins =
            emit_host_fixture_inputs(&dag, &inputs, &mut state, &strategy).expect("Inputs pins");
        assert_eq!(pins.claim_input_root, "host_claim_pin");
        assert_eq!(pins.expected_eval_root, "expected_eval_pin");
    }

    #[test]
    fn v4_witness_variant_resolution_uses_witness_dag_not_dimensions() {
        let dag = dag_with_v4_emit_host_carrier_authorities();
        assert!(
            v4_carrier_variant_tag(&dag, V4_STD_WITNESS_AUTHORITY, "Witness", "Holds").is_ok(),
            "v4.std.witness.Witness must expose Holds"
        );
        assert!(
            v4_carrier_variant_tag(&dag, V4_STD_WITNESS_AUTHORITY, "Witness", "Inhabits").is_err(),
            "dimensions.dag Witness must not satisfy Holds lookup"
        );
    }

    #[test]
    fn try_dispatch_emit_host_python_ignores_unrelated_callable() {
        let mut dag = Dag::empty();
        let unrelated_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: unrelated_id,
            name: Some("unrelated_callable".to_string()),
            connective: TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(
                "probe".to_string(),
            )),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("unrelated.dag", 0, 1),
        });
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        assert!(
            try_dispatch_emit_host_python(&dag, unrelated_id, &[], &mut state, &strategy).is_none(),
            "unrelated callable must not intercept eval dispatch"
        );
        assert!(
            try_dispatch_emit_host_python(
                &dag,
                DeclarationId::test_raw(999),
                &[],
                &mut state,
                &strategy
            )
            .is_none(),
            "missing declaration must not intercept eval dispatch"
        );
    }

    #[test]
    fn emit_host_transport_inputs_carries_claim_only_at_transport_boundary() {
        let dag = Dag::new();
        let inputs = Value::RecordValue(vec![NamedField {
            label: "root".to_string(),
            value: Value::LiteralValue(LiteralBits::String("claim_only".to_string())),
        }]);
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        let pins =
            emit_host_transport_inputs(&dag, &inputs, &mut state, &strategy).expect("claim pin");
        assert_eq!(pins.claim_input_root, "claim_only");
        assert!(
            emit_host_runner::validate_emit_host_transport_inputs(&pins).is_ok(),
            "transport validation must accept claim-only pins"
        );
    }

    #[test]
    fn emit_host_fixture_inputs_rejects_missing_root_field() {
        let inputs = Value::RecordValue(vec![NamedField {
            label: "not_root".to_string(),
            value: Value::LiteralValue(LiteralBits::String("x".to_string())),
        }]);
        let dag = Dag::new();
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        let err = emit_host_transport_inputs(&dag, &inputs, &mut state, &strategy).unwrap_err();
        assert!(matches!(
            err,
            EvalError::BadTransformOperands {
                reason: "expected Inputs.root field"
            }
        ));
    }

    #[test]
    fn emit_host_fixture_inputs_rejects_non_record_operand() {
        let dag = Dag::new();
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        let err = emit_host_transport_inputs(
            &dag,
            &Value::LiteralValue(LiteralBits::String("not_inputs".to_string())),
            &mut state,
            &strategy,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            EvalError::BadTransformOperands {
                reason: "expected Inputs record"
            }
        ));
    }

    /// Five-byte MVP-2 stdout (same contract as `v4_emit_host_harness_test`).
    const EVAL_DISPATCH_PYTHON_FIXTURE: &str =
        "import sys\nsys.stdout.buffer.write(b'\\x00' * 5)\n";

    fn run_emit_host_python_decl(dag: &mut Dag) -> DeclarationId {
        let callee_decl = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: callee_decl,
            name: Some("run_emit_host_python".to_string()),
            connective: TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(
                "run_emit_host_python".to_string(),
            )),
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,
            value_body: None,
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("src/v4/compiler/emit_host.dag", 207, 216),
        });
        callee_decl
    }

    fn eval_dispatch_operands() -> [Value; 3] {
        [
            Value::LiteralValue(LiteralBits::String("eval_target_stub".to_string())),
            Value::LiteralValue(LiteralBits::String(
                EVAL_DISPATCH_PYTHON_FIXTURE.to_string(),
            )),
            Value::RecordValue(vec![NamedField {
                label: "root".to_string(),
                value: Value::LiteralValue(LiteralBits::String("eval_claim_root".to_string())),
            }]),
        ]
    }

    #[test]
    fn try_dispatch_emit_host_python_reifies_accepted_receipt_for_pass_fixture() {
        let mut dag = dag_with_hermetic_v4_emit_host_carriers();
        let callee_decl = run_emit_host_python_decl(&mut dag);
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        let operands = eval_dispatch_operands();
        let result =
            try_dispatch_emit_host_python(&dag, callee_decl, &operands, &mut state, &strategy)
                .expect("python dispatch must claim emit_host.dag row")
                .expect("python dispatch must succeed for pass fixture");
        let Value::VariantValue { tag, payload } = &result else {
            panic!("expected Outcome variant, got {result:?}");
        };
        assert_eq!(
            v4_carrier_variant_tag(&dag, V4_STD_DIAGNOSTIC_AUTHORITY, "Outcome", "Accepted")
                .expect("Outcome::Accepted"),
            *tag
        );
        let Value::RecordValue(outcome_fields) = &**payload else {
            panic!("expected Accepted payload record");
        };
        let Value::RecordValue(receipt_fields) = &outcome_fields
            .iter()
            .find(|field| field.label == "value")
            .expect("Accepted.value")
            .value
        else {
            panic!("expected EmitHostRunReceipt record");
        };
        let source = receipt_fields
            .iter()
            .find(|field| field.label == "source_text")
            .expect("receipt.source_text")
            .value
            .clone();
        assert_eq!(
            source,
            Value::LiteralValue(LiteralBits::String(
                EVAL_DISPATCH_PYTHON_FIXTURE.to_string()
            ))
        );
    }

    #[test]
    fn run_emit_host_python_eval_dispatch_via_evaluator_callable_transform() {
        use crate::dag::TransformTarget;
        use crate::evaluator::{
            eval_node, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder,
        };

        let mut dag = dag_with_hermetic_v4_emit_host_carriers();
        let callee_decl = run_emit_host_python_decl(&mut dag);
        let span = SourceSpan::new("emit_host_eval_dispatch_test.v3", 0, 1);
        let target_port = dag.alloc_port(None);
        let source_port = dag.alloc_port(None);
        let inputs_port = dag.alloc_port(None);
        let call_output = dag.push_transform(
            TransformTarget::Callable(callee_decl),
            vec![target_port, source_port, inputs_port],
            span.clone(),
        );
        let entry = dag
            .port(call_output)
            .produced_by
            .expect("callable transform must produce output port");
        let operands = eval_dispatch_operands();
        let frame = EvalFrame::from_bindings([
            (target_port, operands[0].clone()),
            (source_port, operands[1].clone()),
            (inputs_port, operands[2].clone()),
        ])
        .expect("eval operand bindings");
        let mut state = EvalStateStack::with_root_frame(frame);
        let strategy = EvalStrategy::ApplicativeOrder {
            input_order: InputEvaluationOrder::LeftFirst,
        };

        let via_eval = eval_node(&dag, entry, &mut state, &strategy).expect("eval dispatch");
        assert_eq!(
            state.frames_outer_to_inner().len(),
            1,
            "eval frame must pop"
        );

        let Value::VariantValue { tag, payload } = &via_eval else {
            panic!("expected Outcome variant, got {via_eval:?}");
        };
        assert_eq!(
            v4_carrier_variant_tag(&dag, V4_STD_DIAGNOSTIC_AUTHORITY, "Outcome", "Accepted")
                .expect("v4.std.diagnostic.Outcome::Accepted"),
            *tag,
            "evaluator Callable dispatch must reify Accepted host receipt"
        );
        let Value::RecordValue(outcome_fields) = &**payload else {
            panic!("expected Accepted payload record");
        };
        let Value::RecordValue(receipt_fields) = &outcome_fields
            .iter()
            .find(|field| field.label == "value")
            .expect("Accepted.value")
            .value
        else {
            panic!("expected EmitHostRunReceipt record");
        };
        let source = receipt_fields
            .iter()
            .find(|field| field.label == "source_text")
            .expect("receipt.source_text")
            .value
            .clone();
        assert_eq!(
            source,
            Value::LiteralValue(LiteralBits::String(
                EVAL_DISPATCH_PYTHON_FIXTURE.to_string()
            )),
            "eval dispatch must invoke emit_host_runner with the source operand"
        );
    }

    #[test]
    fn emit_host_fixture_inputs_rejects_malformed_host_claim_pin() {
        let dag = Dag::new();
        let inputs = Value::RecordValue(vec![
            NamedField {
                label: "root".to_string(),
                value: Value::LiteralValue(LiteralBits::String("eval_root_pin".to_string())),
            },
            NamedField {
                label: "expected_eval_root".to_string(),
                value: optional_present_value(
                    &dag,
                    Value::LiteralValue(LiteralBits::String("expected_eval_pin".to_string())),
                ),
            },
            NamedField {
                label: "host_claim_pin".to_string(),
                value: Value::LiteralValue(LiteralBits::String("not_optional".to_string())),
            },
        ]);
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();
        let err = emit_host_fixture_inputs(&dag, &inputs, &mut state, &strategy)
            .expect_err("malformed host_claim_pin must fail closed");
        assert_eq!(
            err,
            EvalError::BadTransformOperands {
                reason: "expected Inputs optional field to be Optional variant"
            }
        );
    }

    #[test]
    fn host_setup_failure_reason_maps_variant() {
        assert_eq!(
            host_setup_failure_reason(&HostSetupFailure::EmptyClaimInputRoot),
            "emit_host_setup_empty_claim_input_root"
        );
    }

    #[test]
    fn go_eval_dispatch_claims_only_emit_host_authority_decl() {
        let mut dag = Dag::new();
        let go_decl = run_emit_host_decl(
            &mut dag,
            "run_emit_host_go",
            "src/v4/compiler/emit_host.dag",
        );
        let lookalike_decl =
            run_emit_host_decl(&mut dag, "run_emit_host_go", "src/v4/compiler/other.dag");
        let rust_decl = run_emit_host_decl(
            &mut dag,
            "run_emit_host_rust",
            "src/v4/compiler/emit_host.dag",
        );
        let mut state = empty_eval_state();
        let strategy = applicative_strategy();

        let claimed =
            try_dispatch_emit_host_go(&dag, go_decl, &[], &mut state, &strategy).expect("claimed");
        assert_eq!(
            claimed,
            Err(EvalError::TransformArityMismatch {
                expected: 3,
                got: 0
            }),
            "the Go hook must claim the substrate run_emit_host_go declaration before user-body eval"
        );
        assert!(
            try_dispatch_emit_host_go(&dag, lookalike_decl, &[], &mut state, &strategy).is_none(),
            "same-name declarations outside emit_host.dag must not be intercepted"
        );
        assert!(
            try_dispatch_emit_host_go(&dag, rust_decl, &[], &mut state, &strategy).is_none(),
            "run_emit_host_rust must stay on the Rust dispatch hook"
        );
    }
}
