//! T-22 substrate eval dispatch: `run_emit_host_rust` → `tools/emit_host_runner`.
//!
//! **Modeled authority:** `src/v4/compiler/emit_host.dag` + `v4.std.host_run` — host receipt
//! assembly (`emit_host_receipt_from_source`, `host_logical_run_from_exit`) is evaluated via
//! [`evaluator::eval_callable_declaration`], not re-encoded here. This module only maps runner
//! facts → substrate operand carriers and invokes the eval hook.
//!
//! **P5 receipt:** `EXPECTED_HAND_AUTHORED_NON_TEST` row in `sg0_census_test.rs`; lane
//! `T-PB-B` / `pb_rust_tests_outside_residual_zero`; dissolution: delete when substrate eval
//! owns host dispatch without this intercept (`emit_host_bridge.rs` retires with harness).

use crate::dag::{Dag, DeclarationId, LiteralBits, TypeConnective};
use crate::evaluator::{
    EvalError, EvalStateStack, EvalStrategy, NamedField, Value,
};
use emit_host_runner::{
    ExitWitness, HostExit, HostExitOutcome, HostLogicalFailure, HostSetupFailure,
};

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
    let inputs = match emit_host_fixture_inputs(dag, &operands[2]) {
        Ok(inputs) => inputs,
        Err(err) => return Some(Err(err)),
    };
    let work_dir = emit_host_runner::default_work_dir(&format!(
        "gunbc_eval_emit_host_rust_{}",
        std::process::id()
    ));
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

fn is_run_emit_host_rust_decl(dag: &Dag, callee_decl: DeclarationId) -> bool {
    let callee = dag.declaration(callee_decl);
    callee.name.as_deref() == Some("run_emit_host_rust")
        && matches!(callee.connective, TypeConnective::Arrow { .. })
}

fn find_fn_decl(dag: &Dag, name: &str) -> Result<DeclarationId, EvalError> {
    dag.declarations()
        .iter()
        .find(|decl| decl.name.as_deref() == Some(name))
        .map(|decl| decl.id)
        .ok_or(EvalError::BadTransformOperands {
            reason: "substrate fn declaration not found in eval dag",
        })
}

fn run_emit_host_setup_rejected(
    dag: &Dag,
    setup: &HostSetupFailure,
) -> Result<Value, EvalError> {
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

fn emit_host_fixture_inputs(
    dag: &Dag,
    value: &Value,
) -> Result<emit_host_runner::EmitHostFixtureInputs, EvalError> {
    let root = inputs_root_field(value)?;
    let pin = host_pin_from_inputs_root(dag, root)?;
    Ok(emit_host_runner::EmitHostFixtureInputs {
        claim_input_root: pin.clone(),
        // Substrate passes only `claim_input_root` in `Inputs.root` for `run_emit_host` today
        // (`emit_host.dag` `run_test_claim_emit_vs_eval_for_claim`); runner validates both pins.
        expected_eval_root: pin,
    })
}

fn inputs_root_field<'a>(inputs: &'a Value) -> Result<&'a Value, EvalError> {
    match inputs {
        Value::RecordValue(fields) => fields
            .iter()
            .find(|field| field.label == "root")
            .map(|field| &field.value)
            .ok_or(EvalError::BadTransformOperands {
                reason: "expected Inputs.root field",
            }),
        _ => Err(EvalError::BadTransformOperands {
            reason: "expected Inputs record",
        }),
    }
}

/// Project `Inputs.root: Node` (eval carrier) to a non-empty runner pin string.
fn host_pin_from_inputs_root(dag: &Dag, root: &Value) -> Result<String, EvalError> {
    let pin = match root {
        Value::LiteralValue(LiteralBits::String(s)) => s.clone(),
        node => node_primary_symbol(dag, node)
            .unwrap_or_else(|| value_structural_digest(node, dag)),
    };
    if pin.is_empty() {
        return Err(EvalError::BadTransformOperands {
            reason: "Inputs.root projected to empty host pin",
        });
    }
    Ok(pin)
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
        Value::RecordValue(fields) => fields
            .iter()
            .find(|f| f.label == "identity")
            .and_then(|f| match &f.value {
                Value::LiteralValue(LiteralBits::String(s)) => Some(s.clone()),
                _ => None,
            }),
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

/// Deterministic structural digest for substrate `Node` values (no `Debug`).
fn value_structural_digest(value: &Value, dag: &Dag) -> String {
    match value {
        Value::LiteralValue(LiteralBits::String(s)) => format!("S:{s}"),
        Value::LiteralValue(LiteralBits::Int(i)) => format!("I:{i}"),
        Value::LiteralValue(LiteralBits::Bool(b)) => format!("B:{b}"),
        Value::RecordValue(fields) => {
            let mut parts: Vec<String> = fields
                .iter()
                .map(|f| format!("{}:{}", f.label, value_structural_digest(&f.value, dag)))
                .collect();
            parts.sort();
            format!("R{{{}}}", parts.join(","))
        }
        Value::VariantValue { tag, payload } => {
            let arm = dag
                .declaration(*tag)
                .name
                .as_deref()
                .unwrap_or("?");
            format!(
                "V:{arm}:{}",
                value_structural_digest(payload, dag)
            )
        }
        Value::NodeRef(id) => format!("N:{id:?}"),
        Value::CardinalityValue(bound) => format!("C:{bound:?}"),
    }
}

fn variant_decl_id(
    dag: &Dag,
    type_name: &str,
    variant_name: &str,
) -> Result<DeclarationId, EvalError> {
    let decl = dag
        .declaration_by_name(type_name)
        .ok_or(EvalError::BadTransformOperands {
            reason: "variant carrier type not found",
        })?;
    let TypeConnective::Disj { variants } = &decl.connective else {
        return Err(EvalError::BadTransformOperands {
            reason: "variant carrier is not a sum type",
        });
    };
    variants
        .iter()
        .find(|variant| variant.label == variant_name)
        .map(|variant| variant.ty)
        .ok_or(EvalError::BadTransformOperands {
            reason: "variant arm not found",
        })
}

fn list_instantiation_for_element(
    dag: &Dag,
    element_ty: DeclarationId,
) -> Result<DeclarationId, EvalError> {
    let list_decl = dag
        .declaration_by_name("List")
        .ok_or(EvalError::BadTransformOperands {
            reason: "List type not found",
        })?;
    dag.declarations()
        .iter()
        .find_map(|decl| {
            let TypeConnective::Instantiation {
                template,
                arguments,
            } = &decl.connective
            else {
                return None;
            };
            (*template == list_decl.id
                && arguments.len() == 1
                && arguments[0].value == element_ty)
                .then_some(decl.id)
        })
        .ok_or(EvalError::BadTransformOperands {
            reason: "List<element> instantiation not found in dag",
        })
}

fn find_list_variant_tag(
    dag: &Dag,
    list_ty: DeclarationId,
    variant: &str,
) -> Result<DeclarationId, EvalError> {
    let list_decl = dag
        .declaration_by_name("List")
        .ok_or(EvalError::BadTransformOperands {
            reason: "List type not found",
        })?;
    let arm_ty = match &list_decl.connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .find(|v| v.label == variant)
            .ok_or(EvalError::BadTransformOperands {
                reason: "List variant arm not found",
            })?
            .ty,
        _ => {
            return Err(EvalError::BadTransformOperands {
                reason: "List is not a sum type",
            });
        }
    };
    let elem_ty = match &dag.declaration(list_ty).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if *template == list_decl.id && arguments.len() == 1 => arguments[0].value,
        _ => {
            return Err(EvalError::BadTransformOperands {
                reason: "expected List<elem> instantiation",
            });
        }
    };
    dag.declarations()
        .iter()
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
            reason: "List variant constructor tag not found",
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

fn bool_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let bool_ty = dag
        .declaration_by_name("Bool")
        .ok_or(EvalError::BadTransformOperands {
            reason: "Bool type not found",
        })?
        .id;
    list_instantiation_for_element(dag, bool_ty)
}

fn byte_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let byte_ty = dag
        .declaration_by_name("Byte")
        .ok_or(EvalError::BadTransformOperands {
            reason: "Byte type not found",
        })?
        .id;
    list_instantiation_for_element(dag, byte_ty)
}

fn string_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    list_instantiation_for_element(
        dag,
        dag.declaration_by_name("String")
            .ok_or(EvalError::BadTransformOperands {
                reason: "String type not found",
            })?
            .id,
    )
}

fn byte_value(dag: &Dag, byte: u8) -> Result<Value, EvalError> {
    let bits: Vec<Value> = (0..8)
        .map(|shift| Value::LiteralValue(LiteralBits::Bool((byte >> (7 - shift)) & 1 != 0)))
        .collect();
    Ok(Value::RecordValue(vec![NamedField {
        label: "bits".to_string(),
        value: list_from_values(dag, bool_list_ty(dag)?, bits)?,
    }]))
}

fn byte_string_value(dag: &Dag, bytes: &[u8]) -> Result<Value, EvalError> {
    let elems: Result<Vec<Value>, EvalError> = bytes
        .iter()
        .copied()
        .map(|b| byte_value(dag, b))
        .collect();
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

fn diagnostic_list_ty(dag: &Dag) -> Result<DeclarationId, EvalError> {
    let diagnostic_ty = dag
        .declaration_by_name("Diagnostic")
        .ok_or(EvalError::BadTransformOperands {
            reason: "Diagnostic type not found",
        })?
        .id;
    list_instantiation_for_element(dag, diagnostic_ty)
}

fn diagnostics_none_variant(dag: &Dag) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Diagnostics", "None")?,
        payload: Box::new(Value::RecordValue(Vec::new())),
    })
}

fn accepted_variant(dag: &Dag, value: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Outcome", "Accepted")?,
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
        tag: variant_decl_id(dag, "Outcome", "Rejected")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "diagnostics".to_string(),
            value: diagnostics,
        }])),
    })
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
        tag: variant_decl_id(dag, "Correction", "Unavailable")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "reason".to_string(),
            value: Value::VariantValue {
                tag: variant_decl_id(dag, "NoCorrectionReason", "ExternalContractUnknown")?,
                payload: Box::new(Value::RecordValue(vec![])),
            },
        }])),
    })
}

fn port_locus_value(dag: &Dag, port: &str) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Locus", "PortLocus")?,
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

fn host_setup_failure_diagnostic(
    dag: &Dag,
    failure: &HostSetupFailure,
) -> Result<Value, EvalError> {
    emit_host_port_diagnostic(dag, host_setup_failure_reason(failure))
}

fn witness_holds_variant(dag: &Dag, value: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Witness", "Holds")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "value".to_string(),
            value,
        }])),
    })
}

fn witness_violates_variant(dag: &Dag, diagnostic: Value) -> Result<Value, EvalError> {
    Ok(Value::VariantValue {
        tag: variant_decl_id(dag, "Witness", "Violates")?,
        payload: Box::new(Value::RecordValue(vec![NamedField {
            label: "diagnostic".to_string(),
            value: diagnostic,
        }])),
    })
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

fn host_exit_value(dag: &Dag, exit: &HostExit) -> Result<Value, EvalError> {
    Ok(Value::RecordValue(vec![NamedField {
        label: "outcome".to_string(),
        value: host_exit_outcome_value(dag, exit)?,
    }]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::Dag;
    use emit_host_runner::HostSetupFailure;

    fn conj_node_value() -> Value {
        Value::RecordValue(vec![
            NamedField {
                label: "kind".to_string(),
                value: Value::RecordValue(vec![NamedField {
                    label: "connective".to_string(),
                    value: Value::RecordValue(vec![]),
                }]),
            },
            NamedField {
                label: "children".to_string(),
                value: Value::RecordValue(vec![]),
            },
        ])
    }

    #[test]
    fn emit_host_fixture_inputs_accepts_node_shaped_inputs_root() {
        let dag = Dag::new();
        let inputs = Value::RecordValue(vec![NamedField {
            label: "root".to_string(),
            value: conj_node_value(),
        }]);
        let pins = emit_host_fixture_inputs(&dag, &inputs).expect("node-shaped Inputs.root");
        assert!(!pins.claim_input_root.is_empty());
        assert_eq!(pins.claim_input_root, pins.expected_eval_root);
    }

    #[test]
    fn host_setup_failure_reason_maps_variant() {
        assert_eq!(
            host_setup_failure_reason(&HostSetupFailure::EmptyClaimInputRoot),
            "emit_host_setup_empty_claim_input_root"
        );
    }

    #[test]
    fn value_structural_digest_is_nonempty_for_node_record() {
        let dag = Dag::new();
        let digest = value_structural_digest(&conj_node_value(), &dag);
        assert!(digest.starts_with("R{"));
        assert!(digest.len() > 4);
    }
}
