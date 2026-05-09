//! Enforced lens applications (`EnforcedApplication` from `lens_application.dag`).
//!
//! Gate #92 (`complexity_violation_compile_error_demonstrated`): when a program
//! authors `EnforcedApplication<ComplexitySummary, AsymptoticClass>` referencing
//! `complexity_enforceable`, infer checks the named section using the **same**
//! `LensEnforcement` projection + violation relation as `complexity.dag`
//! (`complexity_enforcement_project` / `complexity_enforcement_violates`, surfaced
//! through `complexity_lens_generated`) and attaches a compile diagnostic on violation.

use std::collections::HashMap;

use crate::dag::{
    positive_descent_count, ArrowBody, AsymptoticClass, Dag, DeclarationId, FieldValue,
    LiteralBits, Lookup, PortId, PositiveDescentAmount, TypeConnective, ValueBody,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::lens_cost::{
    complexity_enforcement_project, complexity_enforcement_violates, complexity_of,
};

/// Fail-closed check for landed complexity enforcement applications.
pub(crate) fn check_enforced_lens_applications(dag: &mut Dag) {
    let Some(enforced_template) = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("EnforcedApplication")
                && d.span.file.ends_with("lens_application.dag")
        })
        .map(|d| d.id)
    else {
        return;
    };
    let Some(complexity_enforceable_id) = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("complexity_enforceable")
                && d.span.file.ends_with("complexity.dag")
        })
        .map(|d| d.id)
    else {
        return;
    };
    let Some(asymptotic_disj) = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("AsymptoticClass") && d.span.file.ends_with("algebra.dag")
        })
        .map(|d| d.id)
    else {
        return;
    };
    let Some(section_ref_disj) = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("SectionRef") && d.span.file.ends_with("lens_application.dag")
        })
        .map(|d| d.id)
    else {
        return;
    };
    let Some(declaration_scope_conj) = declaration_scope_payload_conj(dag, section_ref_disj) else {
        return;
    };
    let positive_descent_disj = dag
        .declaration_by_name("PositiveDescentAmount")
        .map(|d| d.id);

    let mut violations: Vec<Diagnostic> = Vec::new();

    for decl in dag.declarations() {
        let Some(body) = decl.value_body.as_ref() else {
            continue;
        };
        let ValueBody::Structural { fields } = body else {
            continue;
        };
        let Some(meta_id) = decl.meta_tag else {
            continue;
        };
        if !matches_enforced_application_instantiation(dag, meta_id, enforced_template) {
            continue;
        }
        let fm = field_map(fields);
        let enforceable = match fm.get("enforceable_lens") {
            Some(FieldValue::Reference(id)) => *id,
            _ => continue,
        };
        if enforceable != complexity_enforceable_id {
            continue;
        }
        let section = match fm.get("section") {
            Some(v) => *v,
            _ => continue,
        };
        let Some(fn_decl) =
            resolve_declaration_scope_fn(dag, section, section_ref_disj, declaration_scope_conj)
        else {
            violations.push(Diagnostic::ParseError {
                message: "lens enforcement: could not resolve function declaration for section"
                    .to_string(),
                span: decl.span.clone(),
                fixes: Vec::new(),
            });
            continue;
        };
        let Some(port) = fn_result_port(dag, fn_decl) else {
            violations.push(Diagnostic::ParseError {
                message: "lens enforcement: could not resolve function result port for section"
                    .to_string(),
                span: decl.span.clone(),
                fixes: Vec::new(),
            });
            continue;
        };
        let budget_val = match fm.get("budget") {
            Some(v) => *v,
            _ => continue,
        };
        let Some(budget_class) =
            field_value_asymptotic_class(dag, asymptotic_disj, positive_descent_disj, budget_val)
        else {
            violations.push(Diagnostic::ParseError {
                message: "lens enforcement: could not read complexity budget `AsymptoticClass` \
                          (ill-formed `ClassPolynomial` budgets must use Peano `degree` ≥ 3; \
                          use `ClassLinear` / `ClassQuadratic` for sub-cubic tiers)"
                    .to_string(),
                span: decl.span.clone(),
                fixes: Vec::new(),
            });
            continue;
        };
        let span = match fm.get("span") {
            Some(FieldValue::Record(span_fields)) => record_as_source_span(span_fields),
            _ => None,
        }
        .unwrap_or_else(|| decl.span.clone());

        let summary = match complexity_of(dag, &port) {
            Lookup::Hit(s) => s,
            Lookup::Miss => {
                violations.push(Diagnostic::ParseError {
                    message: "lens enforcement: complexity lens returned Miss for section — \
                              cannot enforce budget"
                        .to_string(),
                    span: span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            }
        };
        let observed = complexity_enforcement_project(&summary);
        if !complexity_enforcement_violates(&summary, &budget_class, &observed) {
            continue;
        }
        violations.push(Diagnostic::ParseError {
            message: format!(
                "lens enforcement violation: complexity budget {budget_class:?} exceeded \
                 by observed class {observed:?} (declared `EnforcedApplication` at {})",
                decl.name.as_deref().unwrap_or("?")
            ),
            span,
            fixes: Vec::new(),
        });
    }

    for diagnostic in violations {
        dag.attach_diagnostic(diagnostic);
    }
}

fn field_map(fields: &[(String, FieldValue)]) -> HashMap<&str, &FieldValue> {
    fields.iter().map(|(k, v)| (k.as_str(), v)).collect()
}

fn matches_enforced_application_instantiation(
    dag: &Dag,
    mut meta: DeclarationId,
    enforced_template: DeclarationId,
) -> bool {
    for _ in 0..16 {
        match &dag.declaration(meta).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                if *template == enforced_template && arguments.len() == 2 {
                    return true;
                }
                if arguments.is_empty() {
                    meta = *template;
                    continue;
                }
                return false;
            }
            _ => return false,
        }
    }
    false
}

fn declaration_scope_payload_conj(
    dag: &Dag,
    section_ref_disj: DeclarationId,
) -> Option<DeclarationId> {
    let TypeConnective::Disj { variants } = &dag.declaration(section_ref_disj).connective else {
        return None;
    };
    let v = variants.iter().find(|f| f.label == "DeclarationScope")?;
    Some(v.ty)
}

fn resolve_declaration_scope_fn(
    dag: &Dag,
    section: &FieldValue,
    section_ref_disj: DeclarationId,
    declaration_scope_conj: DeclarationId,
) -> Option<DeclarationId> {
    let _ = section_ref_disj;
    let FieldValue::Variant {
        constructor,
        payload,
    } = section
    else {
        return None;
    };
    if *constructor != declaration_scope_conj {
        return None;
    }
    let id = match payload.as_slice() {
        [FieldValue::Reference(id)] => *id,
        [FieldValue::Record(parts)] => {
            let mut decl_id: Option<DeclarationId> = None;
            for (label, value) in parts {
                if label == "declaration" {
                    if let FieldValue::Reference(id) = value {
                        decl_id = Some(*id);
                    }
                }
            }
            decl_id?
        }
        _ => return None,
    };
    matches!(dag.declaration(id).connective, TypeConnective::Arrow { .. }).then_some(id)
}

fn fn_result_port(dag: &Dag, fn_decl: DeclarationId) -> Option<PortId> {
    let decl = dag.declaration(fn_decl);
    let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_node),
        ..
    } = &decl.connective
    else {
        return None;
    };
    let bind = bind_node.bind(dag);
    Some(bind.result_port())
}

fn record_as_source_span(fields: &[(String, FieldValue)]) -> Option<SourceSpan> {
    let mut file: Option<String> = None;
    let mut byte_start: Option<u32> = None;
    let mut byte_end: Option<u32> = None;
    for (label, value) in fields {
        match (label.as_str(), value) {
            ("file", FieldValue::Literal(LiteralBits::String(s))) => file = Some(s.clone()),
            ("byte_start", FieldValue::Literal(LiteralBits::Int(n))) => {
                byte_start = (*n).try_into().ok();
            }
            ("byte_end", FieldValue::Literal(LiteralBits::Int(n))) => {
                byte_end = (*n).try_into().ok();
            }
            ("start", FieldValue::Literal(LiteralBits::Int(n))) => {
                byte_start = (*n).try_into().ok();
            }
            ("end", FieldValue::Literal(LiteralBits::Int(n))) => {
                byte_end = (*n).try_into().ok();
            }
            _ => {}
        }
    }
    Some(SourceSpan::new(file?, byte_start?, byte_end?))
}

fn field_value_asymptotic_class(
    dag: &Dag,
    ac_disj: DeclarationId,
    pda_disj: Option<DeclarationId>,
    value: &FieldValue,
) -> Option<AsymptoticClass> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    let TypeConnective::Disj { variants } = &dag.declaration(ac_disj).connective else {
        return None;
    };
    let variant = variants.iter().find(|v| v.ty == *constructor)?;
    match variant.label.as_str() {
        "ClassConstant" => Some(AsymptoticClass::ClassConstant),
        "ClassLog" => Some(AsymptoticClass::ClassLog),
        "ClassLinear" => Some(AsymptoticClass::ClassLinear),
        "ClassLinearithmic" => Some(AsymptoticClass::ClassLinearithmic),
        "ClassQuadratic" => Some(AsymptoticClass::ClassQuadratic),
        "ClassExponential" => Some(AsymptoticClass::ClassExponential),
        "ClassUnknown" => Some(AsymptoticClass::ClassUnknown),
        "ClassPolynomial" => {
            let pda = pda_disj?;
            let degree = decode_positive_descent_amount(dag, pda, payload)?;
            // `AsymptoticClass::ClassPolynomial` in `algebra.dag` is k ≥ 3; Peano
            // counts 1–2 are `ClassLinear` / `ClassQuadratic` territory — reject as
            // an invalid budget shape (fail closed; openai-pro / P1).
            if positive_descent_count(&degree) < 3 {
                return None;
            }
            Some(AsymptoticClass::ClassPolynomial { degree })
        }
        _ => None,
    }
}

fn decode_positive_descent_amount(
    dag: &Dag,
    pda_disj: DeclarationId,
    poly_payload: &[FieldValue],
) -> Option<PositiveDescentAmount> {
    match poly_payload {
        [FieldValue::Record(parts)] => {
            let value = parts.iter().find(|(k, _)| k == "degree").map(|(_, v)| v)?;
            decode_positive_descent_variant(dag, pda_disj, value)
        }
        [single] => decode_positive_descent_variant(dag, pda_disj, single),
        _ => None,
    }
}

fn decode_positive_descent_variant(
    dag: &Dag,
    pda_disj: DeclarationId,
    value: &FieldValue,
) -> Option<PositiveDescentAmount> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    let TypeConnective::Disj { variants } = &dag.declaration(pda_disj).connective else {
        return None;
    };
    let variant = variants.iter().find(|v| v.ty == *constructor)?;
    match variant.label.as_str() {
        "OneStep" => Some(PositiveDescentAmount::OneStep),
        "AdditionalStep" => {
            let FieldValue::Record(parts) = payload.first()? else {
                return None;
            };
            let prev = parts
                .iter()
                .find(|(k, _)| k == "previous")
                .map(|(_, v)| v)?;
            let inner = decode_positive_descent_variant(dag, pda_disj, prev)?;
            Some(PositiveDescentAmount::AdditionalStep {
                previous: Box::new(inner),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod polynomial_budget_class_policy_tests {
    use crate::dag::{positive_amount_from_i64, positive_descent_count, PositiveDescentAmount};

    fn polynomial_budget_peano_is_admissible(degree: &PositiveDescentAmount) -> bool {
        positive_descent_count(degree) >= 3
    }

    #[test]
    fn class_polynomial_enforcement_budget_rejects_sub_cubic_peano_degrees() {
        assert_eq!(positive_descent_count(&PositiveDescentAmount::OneStep), 1);
        assert!(!polynomial_budget_peano_is_admissible(
            &PositiveDescentAmount::OneStep
        ));
        let deg2 = PositiveDescentAmount::AdditionalStep {
            previous: Box::new(PositiveDescentAmount::OneStep),
        };
        assert_eq!(positive_descent_count(&deg2), 2);
        assert!(!polynomial_budget_peano_is_admissible(&deg2));
        let deg3 = positive_amount_from_i64(3).expect("synthetic degree 3");
        assert!(polynomial_budget_peano_is_admissible(&deg3));
    }
}
