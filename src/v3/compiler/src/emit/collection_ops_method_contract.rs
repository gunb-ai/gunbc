//! `CollectionOps.*_contract` fields cross `DeclarationRef` boundaries. Emit
//! consumers must fail closed on the **algebra identity** (P1): each referenced
//! declaration must be a `MethodTemplateContract` data binding whose
//! `dag_method.decl` resolves to the expected registry `*_method` row.

use crate::dag::{Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};

/// Fail closed unless `contract_decl` is a `MethodTemplateContract` instance
/// whose `dag_method.decl` resolves to `expected_method_decl`.
pub(crate) fn require_method_template_contract_dag_method(
    dag: &Dag,
    contract_decl: DeclarationId,
    collection_ops_field: &'static str,
    expected_method_decl: DeclarationId,
) -> Result<(), &'static str> {
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
    if method_id != expected_method_decl {
        return Err(match collection_ops_field {
            "fold_contract" => "CollectionOps.fold_contract must target fold_method",
            "concat_contract" => "CollectionOps.concat_contract must target concat_method",
            "length_contract" => "CollectionOps.length_contract must target length_method",
            "is_empty_contract" => {
                "CollectionOps.is_empty_contract must target is_empty_method (FreeMonoid<T>.is_empty -> Bool)"
            }
            _ => "CollectionOps contract dag_method does not match expected registry method",
        });
    }

    Ok(())
}
