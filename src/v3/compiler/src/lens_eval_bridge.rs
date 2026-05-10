//! Gate #5 (`lens_apply_dot_rs_retired`, gunbc#2374): bridge substrate [`FieldValue`] lens
//! carriers into [`crate::evaluator::Value`] so lens bodies can execute via the R2 evaluator /
//! PB-runtime-shaped `evaluate` semantics.
//!
//! **Merge posture:** WIP scaffolding — squash-merge only after §7.1 Row-4 green
//! (`pb_runtime_equivalent_to_evaluator_on_corpus`). Surface mismatches with substrate /
//! evaluator expectations → STOP-and-PING PB Mgr before workarounds.

use crate::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective};
use crate::evaluator::{NamedField, Value};

const DECLARATION_REF_RECORD_LABEL: &str = "__declaration_ref";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LensEvalBridgeError(pub &'static str);

pub fn field_value_to_runtime_value(
    dag: &Dag,
    fv: &FieldValue,
) -> Result<Value, LensEvalBridgeError> {
    match fv {
        FieldValue::Literal(bits) => Ok(Value::LiteralValue(bits.clone())),
        FieldValue::Reference(id) => Ok(Value::RecordValue(vec![NamedField {
            label: DECLARATION_REF_RECORD_LABEL.to_string(),
            value: Value::LiteralValue(LiteralBits::Int(i64::from(id.raw()).to_string())),
        }])),
        FieldValue::Record(fields) => {
            let mut out = Vec::with_capacity(fields.len());
            for (label, inner) in fields {
                out.push(NamedField {
                    label: label.clone(),
                    value: field_value_to_runtime_value(dag, inner)?,
                });
            }
            Ok(Value::RecordValue(out))
        }
        FieldValue::Variant {
            constructor,
            payload,
        } => {
            let inner = variant_field_payload_to_runtime_value(dag, *constructor, payload)?;
            Ok(Value::VariantValue {
                tag: *constructor,
                payload: Box::new(inner),
            })
        }
        FieldValue::List(_) | FieldValue::Map(_) => Err(LensEvalBridgeError(
            "List/Map FieldValue → Value bridge not implemented yet",
        )),
    }
}

fn variant_field_payload_to_runtime_value(
    dag: &Dag,
    constructor: DeclarationId,
    payload: &[FieldValue],
) -> Result<Value, LensEvalBridgeError> {
    match &dag.declaration(constructor).connective {
        TypeConnective::Conj { children } => {
            if children.len() != payload.len() {
                return Err(LensEvalBridgeError(
                    "variant payload arity does not match Conj fields",
                ));
            }
            if children.len() == 1 && children[0].label == "_0" {
                return field_value_to_runtime_value(dag, &payload[0]);
            }
            let mut nf = Vec::with_capacity(children.len());
            for (child, fv) in children.iter().zip(payload.iter()) {
                nf.push(NamedField {
                    label: child.label.clone(),
                    value: field_value_to_runtime_value(dag, fv)?,
                });
            }
            Ok(Value::RecordValue(nf))
        }
        _ => match payload.len() {
            0 => Ok(Value::RecordValue(vec![])),
            1 => field_value_to_runtime_value(dag, &payload[0]),
            _ => Err(LensEvalBridgeError(
                "multi-field variant payload without Conj backing",
            )),
        },
    }
}

pub fn runtime_value_to_field_value(
    dag: &Dag,
    value: &Value,
) -> Result<FieldValue, LensEvalBridgeError> {
    match value {
        Value::LiteralValue(bits) => Ok(FieldValue::Literal(bits.clone())),
        Value::RecordValue(fields) => {
            if fields.len() == 1 && fields[0].label == DECLARATION_REF_RECORD_LABEL {
                let Value::LiteralValue(LiteralBits::Int(s)) = &fields[0].value else {
                    return Err(LensEvalBridgeError("malformed declaration ref wrapper"));
                };
                let raw: u32 = s.parse().map_err(|_| LensEvalBridgeError("declaration ref int"))?;
                return Ok(FieldValue::Reference(DeclarationId::from_raw(raw)));
            }
            let mut out = Vec::with_capacity(fields.len());
            for nf in fields {
                out.push((
                    nf.label.clone(),
                    runtime_value_to_field_value(dag, &nf.value)?,
                ));
            }
            Ok(FieldValue::Record(out))
        }
        Value::VariantValue { tag, payload } => {
            let payload_fv = variant_runtime_payload_to_field_value(dag, *tag, payload.as_ref())?;
            Ok(FieldValue::Variant {
                constructor: *tag,
                payload: payload_fv,
            })
        }
        Value::NodeRef(_) | Value::CardinalityValue(_) => Err(LensEvalBridgeError(
            "NodeRef/CardinalityValue → FieldValue not supported for lens bridge yet",
        )),
    }
}

fn variant_runtime_payload_to_field_value(
    dag: &Dag,
    constructor: DeclarationId,
    payload: &Value,
) -> Result<Vec<FieldValue>, LensEvalBridgeError> {
    match &dag.declaration(constructor).connective {
        TypeConnective::Conj { children } => {
            if children.len() == 1 && children[0].label == "_0" {
                return Ok(vec![runtime_value_to_field_value(dag, payload)?]);
            }
            let Value::RecordValue(fields) = payload else {
                return Err(LensEvalBridgeError("expected RecordValue variant payload"));
            };
            if fields.len() != children.len() {
                return Err(LensEvalBridgeError(
                    "variant payload record arity does not match Conj",
                ));
            }
            let mut out = Vec::with_capacity(children.len());
            for (child, nf) in children.iter().zip(fields.iter()) {
                if child.label != nf.label {
                    return Err(LensEvalBridgeError("variant payload label mismatch"));
                }
                out.push(runtime_value_to_field_value(dag, &nf.value)?);
            }
            Ok(out)
        }
        _ => Ok(vec![runtime_value_to_field_value(dag, payload)?]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_to_dag;

    fn minimal_dag() -> Dag {
        compile_to_dag("module m\nfn f() -> Int = 0\n", "lens_eval_bridge_min.v3").expect("compile")
    }

    #[test]
    fn literal_int_roundtrips() {
        let dag = minimal_dag();
        let fv = FieldValue::Literal(LiteralBits::Int("42".to_string()));
        let v = field_value_to_runtime_value(&dag, &fv).expect("to runtime");
        let back = runtime_value_to_field_value(&dag, &v).expect("to field");
        assert_eq!(fv, back);
    }

    #[test]
    fn declaration_reference_roundtrips_via_wrapper() {
        let dag = minimal_dag();
        let id = dag.declaration_by_name("f").expect("f").id;
        let fv = FieldValue::Reference(id);
        let v = field_value_to_runtime_value(&dag, &fv).expect("to runtime");
        let back = runtime_value_to_field_value(&dag, &v).expect("to field");
        assert_eq!(fv, back);
    }
}
