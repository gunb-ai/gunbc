use crate::dag::{AtomPayload, Dag, Declaration, DeclarationId, TypeConnective};

/// `std.error_primitives` declares canonical `Result<ok, err> = Ok { value: ok } | Err { value: err }`.
/// Emitters lower it to a target-native `Result` / `struct { Ok; Err }` carrier and must
/// not also emit a second substrate `type Result`.
///
/// Suppression keys off the **resolved structural fingerprint** of that declaration
/// (name + type-parameter identities + `Ok`/`Err` payload wiring), not `span.file`
/// suffixes — so unrelated modules named `errors.dag` cannot collide, and renaming
/// the std file alone does not silently retarget suppression.
///
/// **Policy:** the fingerprint is **global**, not std-scoped: any other declaration
/// named `Result` that matches this exact shape is also suppressed (intentional — the
/// substrate owns one canonical `Result<ok, err>` carrier; a user-defined twin with
/// the same fingerprint would not emit as a separate `type Result`).
pub(crate) fn substrate_result_type_decl_suppressed_for_emit(
    dag: &Dag,
    decl: &Declaration,
) -> bool {
    if decl.name.as_deref() != Some("Result") {
        return false;
    }
    let [ok_param, err_param] = match decl.type_params.as_slice() {
        [a, b] => [*a, *b],
        _ => return false,
    };
    let ok_decl = dag.declaration(ok_param);
    let err_decl = dag.declaration(err_param);
    let ok_param_ok = matches!(
        &ok_decl.connective,
        TypeConnective::Atom(AtomPayload::TypeParam(name)) if name == "ok"
    );
    let err_param_ok = matches!(
        &err_decl.connective,
        TypeConnective::Atom(AtomPayload::TypeParam(name)) if name == "err"
    );
    if !ok_param_ok || !err_param_ok {
        return false;
    }
    let TypeConnective::Disj { variants } = &decl.connective else {
        return false;
    };
    let Some(ok_field) = variants.iter().find(|v| v.label == "Ok") else {
        return false;
    };
    let Some(err_field) = variants.iter().find(|v| v.label == "Err") else {
        return false;
    };
    substrate_result_variant_payload_is_value_of(dag, ok_field.ty, ok_param)
        && substrate_result_variant_payload_is_value_of(dag, err_field.ty, err_param)
}

fn substrate_result_variant_payload_is_value_of(
    dag: &Dag,
    payload_ty: DeclarationId,
    type_param: DeclarationId,
) -> bool {
    let payload = dag.declaration(payload_ty);
    let TypeConnective::Conj { children } = &payload.connective else {
        return false;
    };
    children.len() == 1 && children[0].label == "value" && children[0].ty == type_param
}

/// `std.error_primitives` declares canonical `DivError = DivideByZero | Overflow`.
/// Like `Result`, emit suppression keys off the resolved structural fingerprint,
/// not the declaration source path. A declaration with this exact global shape is
/// the substrate-owned integer-division error carrier and is materialized by the
/// target division prelude when a program actually needs it.
pub(crate) fn substrate_div_error_type_decl_suppressed_for_emit(
    dag: &Dag,
    decl: &Declaration,
) -> bool {
    if decl.name.as_deref() != Some("DivError") || !decl.type_params.is_empty() {
        return false;
    }
    matches!(&decl.connective, TypeConnective::Disj { variants } if {
        variants.len() == 2
            && variants.iter().any(|variant| {
                variant.label == "DivideByZero"
                    && substrate_div_error_variant_payload_is_unit(dag, variant.ty)
            })
            && variants.iter().any(|variant| {
                variant.label == "Overflow"
                    && substrate_div_error_variant_payload_is_unit(dag, variant.ty)
            })
    })
}

fn substrate_div_error_variant_payload_is_unit(dag: &Dag, payload_ty: DeclarationId) -> bool {
    matches!(
        &dag.declaration(payload_ty).connective,
        TypeConnective::Conj { children } if children.is_empty()
    )
}
