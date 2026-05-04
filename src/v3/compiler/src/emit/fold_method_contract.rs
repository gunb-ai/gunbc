//! `CollectionOps.fold_contract` crosses a `DeclarationRef` boundary. Emit
//! consumers must fail closed on the **algebra identity** (P1): the referenced
//! declaration must be a `MethodTemplateContract` data binding whose
//! `dag_method` targets `fold_method`, not merely a structurally similar record.

use crate::dag::{Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};

/// Fail closed unless `contract_decl` is a `MethodTemplateContract` instance
/// whose `dag_method.decl` resolves to the registry `fold_method` binding.
pub(crate) fn require_fold_method_template_contract(
    dag: &Dag,
    contract_decl: DeclarationId,
) -> Result<(), &'static str> {
    let mtc_type_id = dag
        .method_template_contract_decl()
        .ok_or("internal: MethodTemplateContract type missing from dag")?;

    let fold_method_id = dag
        .fold_method_decl()
        .ok_or("internal: fold_method registry entry missing from dag")?;

    let decl = dag.declaration(contract_decl);
    let template = match &decl.connective {
        TypeConnective::Instantiation { template, .. } => *template,
        _ => {
            return Err(
                "CollectionOps.fold_contract must reference a MethodTemplateContract data binding",
            );
        }
    };
    if template != mtc_type_id {
        return Err("CollectionOps.fold_contract must instantiate MethodTemplateContract");
    }

    let ValueBody::Structural { fields } = decl
        .value_body
        .as_ref()
        .ok_or("fold_contract referenced declaration must carry a structural value body")?
    else {
        return Err(
            "fold_contract referenced declaration must be structural MethodTemplateContract data",
        );
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
    if method_id != fold_method_id {
        return Err("CollectionOps.fold_contract must target fold_method");
    }

    Ok(())
}
