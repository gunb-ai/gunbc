//! E6-G1.a — evaluator-owned substrate `Dag` reification for [`crate::evaluator::evaluate_body`].
//!
//! User `data … : Dag = …` literals remain class-5 blocked; this module bridges compiled
//! [`crate::dag::Dag`] graphs into the five-variant [`crate::evaluator::Value`] carrier without
//! routing the API through [`crate::lens_apply::apply_lens_declaration`] (compatibility seam only).
//!
//! **Program scope:** [`reify_compiled_dag_as_substrate_value`] names whole-`Dag` authority: it
//! reflects **every** behavior in `program.nodes()` in graph order — no `source_file` filter and
//! no inheritance of [`crate::lens_apply::reflect_program_dag_nodes_in_file`] narrowing.
//!
//! **Variant payload fields:** [`eval_constructor_variant_payload_fields`] is the public
//! reification-facing entry to the same [`crate::lower`] query used by the body evaluator when
//! lowering `FieldValue::Variant` into [`Value::VariantValue`].
//!
//! **Declaration references:** `runtime.dag` locks [`Value`] to five variants and does not carry
//! `DeclarationId` as a first-class inhabitant. [`field_value_to_eval_value`] therefore encodes
//! [`FieldValue::Reference`] as [`LiteralBits::Int`] holding [`DeclarationId::raw`] (sign-extended
//! to `i64`). This is an explicit host-bridge contract for reflected substrate facts (e.g.
//! [`TransformTarget::Callable`], [`BranchPattern::ResolvedVariant`]); user programs must not rely
//! on mathematical `Int` semantics for those handles until the substrate value model grows a typed
//! declaration reference.

use crate::dag::{Dag, DeclarationId, FieldValue, LiteralBits};
use crate::evaluator::{NamedField, Value};
use crate::lens_apply::{empty_substrate_list_value, reflect_program_nodes_whole_dag, LensApplyError};

/// Resolves variant-constructor payload field labels for eager reification into
/// [`Value::VariantValue`], delegating to [`crate::lower::eval_constructor_variant_payload_fields`].
pub fn eval_constructor_variant_payload_fields(
    dag: &Dag,
    callee_decl: DeclarationId,
) -> Option<Vec<(String, DeclarationId)>> {
    crate::lower::eval_constructor_variant_payload_fields(dag, callee_decl)
}

/// Convert structural [`FieldValue`] (from whole-dag reflection) into [`Value`].
///
/// See the module note on [`FieldValue::Reference`] encoding.
pub fn field_value_to_eval_value(dag: &Dag, fv: &FieldValue) -> Result<Value, LensApplyError> {
    match fv {
        FieldValue::Literal(bits) => Ok(Value::LiteralValue(bits.clone())),
        FieldValue::Reference(id) => Ok(Value::LiteralValue(LiteralBits::Int(id.raw() as i64))),
        FieldValue::List(_) => Err(LensApplyError::SubstrateReflect(
            "FieldValue::List is not reified; reflection uses Cons spines",
        )),
        FieldValue::Map(_) => Err(LensApplyError::SubstrateReflect(
            "FieldValue::Map is not reified into evaluator Value (E6-G1.a)",
        )),
        FieldValue::Record(rec) => {
            let mut fields = Vec::with_capacity(rec.len());
            for (label, child) in rec {
                fields.push(NamedField {
                    label: label.clone(),
                    value: field_value_to_eval_value(dag, child)?,
                });
            }
            Ok(Value::RecordValue(fields))
        }
        FieldValue::Variant {
            constructor,
            payload,
        } => {
            let field_defs = eval_constructor_variant_payload_fields(dag, *constructor)
                .ok_or(LensApplyError::SubstrateReflect(
                    "variant constructor payload fields missing",
                ))?;
            if field_defs.len() != payload.len() {
                return Err(LensApplyError::SubstrateReflect(
                    "variant payload arity mismatch for eval reification",
                ));
            }
            let converted: Vec<Value> = payload
                .iter()
                .map(|p| field_value_to_eval_value(dag, p))
                .collect::<Result<_, _>>()?;
            let inner = if field_defs.len() == 1 && field_defs[0].0 == "_0" {
                converted.into_iter().next().expect("length checked")
            } else {
                Value::RecordValue(
                    field_defs
                        .into_iter()
                        .zip(converted)
                        .map(|((label, _), value)| NamedField { label, value })
                        .collect(),
                )
            };
            Ok(Value::VariantValue {
                tag: *constructor,
                payload: Box::new(inner),
            })
        }
    }
}

/// Build a substrate-typed `Dag` [`Value`] carrying **all** behaviors from `program.nodes()`.
///
/// `declarations`, `ports`, and `clusters` are empty lists (class-5 blocks structural `Dag` data
/// literals; those fields are not required by the G1.a static fold ratchet).
pub fn reify_compiled_dag_as_substrate_value(program: &Dag) -> Result<Value, LensApplyError> {
    let empty_fv = empty_substrate_list_value(program)?;
    let empty_val = field_value_to_eval_value(program, &empty_fv)?;
    let nodes_fv = reflect_program_nodes_whole_dag(program)?;
    let nodes_val = field_value_to_eval_value(program, &nodes_fv)?;
    Ok(Value::RecordValue(vec![
        NamedField {
            label: "declarations".to_string(),
            value: empty_val.clone(),
        },
        NamedField {
            label: "nodes".to_string(),
            value: nodes_val,
        },
        NamedField {
            label: "ports".to_string(),
            value: empty_val.clone(),
        },
        NamedField {
            label: "clusters".to_string(),
            value: empty_val,
        },
    ]))
}
