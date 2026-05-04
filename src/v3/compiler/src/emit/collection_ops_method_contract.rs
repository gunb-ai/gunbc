//! `CollectionOps.*_contract` fields cross `DeclarationRef` boundaries. Emit
//! consumers must fail closed on the **algebra identity** (P1): each referenced
//! declaration must be a `MethodTemplateContract` data binding whose
//! `dag_method.decl` resolves to the expected registry `*_method` row.

use super::method_emit_template_variant_label;
use crate::dag::{Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MethodTemplateContractEmitTemplate {
    SingleTemplate(String),
    HigherOrderTemplates {
        inline_template: String,
        fn_ref_template: String,
    },
}

/// Fail closed unless `contract_decl` is a `MethodTemplateContract` instance
/// whose `dag_method.decl` resolves to `expected_method_decl`.
pub(crate) fn require_method_template_contract_dag_method(
    dag: &Dag,
    contract_decl: DeclarationId,
    collection_ops_field: &'static str,
    expected_method_decl: DeclarationId,
) -> Result<(), &'static str> {
    let fields = method_template_contract_fields(dag, contract_decl)?;
    require_method_template_contract_fields_dag_method(
        fields,
        collection_ops_field,
        expected_method_decl,
    )
}

pub(crate) fn method_template_contract_decl_emit_template(
    dag: &Dag,
    contract_decl: DeclarationId,
    collection_ops_field: &'static str,
    expected_method_decl: DeclarationId,
) -> Result<MethodTemplateContractEmitTemplate, &'static str> {
    let fields = method_template_contract_fields(dag, contract_decl)?;
    require_method_template_contract_fields_dag_method(
        fields,
        collection_ops_field,
        expected_method_decl,
    )?;
    method_template_contract_fields_emit_template(dag, fields)
}

pub(crate) fn method_template_contract_list_emit_template_for_method(
    dag: &Dag,
    list_decl_name: &'static str,
    collection_ops_field: &'static str,
    expected_method_decl: DeclarationId,
) -> Result<MethodTemplateContractEmitTemplate, &'static str> {
    let decl = dag
        .declaration_by_name(list_decl_name)
        .ok_or("MethodTemplateContract list declaration missing from dag")?;
    let Some(ValueBody::List(entries)) = decl.value_body.as_ref() else {
        return Err("MethodTemplateContract list declaration must carry a list value body");
    };

    for entry in entries {
        let FieldValue::Record(fields) = entry else {
            return Err("MethodTemplateContract list entries must be structural records");
        };
        if method_template_contract_fields_dag_method(fields)? == expected_method_decl {
            require_method_template_contract_fields_dag_method(
                fields,
                collection_ops_field,
                expected_method_decl,
            )?;
            return method_template_contract_fields_emit_template(dag, fields);
        }
    }

    Err("MethodTemplateContract list missing expected dag_method row")
}

fn method_template_contract_fields(
    dag: &Dag,
    contract_decl: DeclarationId,
) -> Result<&[(String, FieldValue)], &'static str> {
    let mtc_type_id = dag
        .declaration_by_name("MethodTemplateContract")
        .ok_or("internal: MethodTemplateContract type missing from dag")?
        .id;

    let decl = dag.declaration(contract_decl);
    let template = match &decl.connective {
        TypeConnective::Instantiation { template, .. } => *template,
        _ => {
            return Err(
                "CollectionOps contract must reference a MethodTemplateContract data binding",
            );
        }
    };
    if template != mtc_type_id {
        return Err("CollectionOps contract must instantiate MethodTemplateContract");
    }

    let Some(body) = decl.value_body.as_ref() else {
        return Err(
            "CollectionOps contract referenced declaration must carry a structural value body",
        );
    };
    let fields = match body {
        ValueBody::Structural { fields } => fields,
        _ => {
            return Err(
                "CollectionOps contract referenced declaration must be structural MethodTemplateContract data",
            );
        }
    };
    Ok(fields)
}

fn require_method_template_contract_fields_dag_method(
    fields: &[(String, FieldValue)],
    collection_ops_field: &'static str,
    expected_method_decl: DeclarationId,
) -> Result<(), &'static str> {
    let method_id = method_template_contract_fields_dag_method(fields)?;
    if method_id != expected_method_decl {
        return Err(match collection_ops_field {
            "fold_contract" => "CollectionOps.fold_contract must target fold_method",
            "concat_contract" => "CollectionOps.concat_contract must target concat_method",
            "length_contract" => "CollectionOps.length_contract must target length_method",
            "filter_contract" => "CollectionOps.filter_contract must target filter_method",
            "flat_map_contract" => "CollectionOps.flat_map_contract must target flat_map_method",
            "any_contract" => "CollectionOps.any_contract must target any_method",
            "all_contract" => "CollectionOps.all_contract must target all_method",
            "is_empty_contract" => {
                "CollectionOps.is_empty_contract must target is_empty_method (FreeMonoid<T>.is_empty -> Bool)"
            }
            _ => "CollectionOps contract dag_method does not match expected registry method",
        });
    }

    Ok(())
}

fn method_template_contract_fields_dag_method(
    fields: &[(String, FieldValue)],
) -> Result<DeclarationId, &'static str> {
    let dag_method = fields
        .iter()
        .find(|(l, _)| l == "dag_method")
        .map(|(_, v)| v)
        .ok_or("MethodTemplateContract missing dag_method field")?;

    let FieldValue::Record(method_ref_fields) = dag_method else {
        return Err("MethodTemplateContract.dag_method must be a MethodRef record");
    };

    let method_id = match method_ref_fields.as_slice() {
        [(label, FieldValue::Reference(id))] if label == "decl" => *id,
        _ => {
            return Err(
                "MethodTemplateContract.dag_method must be exactly `{ decl: DeclarationRef }` (MethodRef)",
            );
        }
    };
    Ok(method_id)
}

fn method_template_contract_fields_emit_template(
    dag: &Dag,
    fields: &[(String, FieldValue)],
) -> Result<MethodTemplateContractEmitTemplate, &'static str> {
    let emit_value = fields
        .iter()
        .find(|(label, _)| label == "emit_template")
        .map(|(_, v)| v)
        .ok_or("MethodTemplateContract missing emit_template field")?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = emit_value
    else {
        return Err("MethodTemplateContract.emit_template must be a sum variant");
    };
    let ctor_name = method_emit_template_variant_label(dag, *constructor).ok_or(
        "MethodTemplateContract.emit_template variant not found under MethodEmitTemplate disj",
    )?;
    match ctor_name {
        "SingleTemplate" => {
            let [FieldValue::Literal(LiteralBits::String(template))] = payload.as_slice() else {
                return Err("SingleTemplate must carry exactly one string template payload");
            };
            Ok(MethodTemplateContractEmitTemplate::SingleTemplate(
                template.replace("%Q", "\""),
            ))
        }
        "HigherOrderTemplates" => {
            let [FieldValue::Literal(LiteralBits::String(inline_template)), FieldValue::Literal(LiteralBits::String(fn_ref_template))] =
                payload.as_slice()
            else {
                return Err(
                    "HigherOrderTemplates must carry inline_template and fn_ref_template strings",
                );
            };
            Ok(MethodTemplateContractEmitTemplate::HigherOrderTemplates {
                inline_template: inline_template.replace("%Q", "\""),
                fn_ref_template: fn_ref_template.replace("%Q", "\""),
            })
        }
        _ => Err("MethodTemplateContract.emit_template uses an unknown MethodEmitTemplate variant"),
    }
}
