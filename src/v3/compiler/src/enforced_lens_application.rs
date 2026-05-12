//! Enforced lens applications (`EnforcedApplication` from `lens_application.dag`).
//!
//! Gate #91 (`enforce_violation_routing_landed`): budget-violation diagnostics are
//! routed through the authored `diagnostic_severity` field — today the single
//! substrate variant `DiagnosticSeverity::Error` maps to [`Diagnostic::ParseError`]
//! (compile-fail / fail-closed), per `lens_application.dag`, design §3, and
//! INVARIANTS C-8.
//!
//! Gate #92 (`complexity_violation_compile_error_demonstrated`): when a program
//! authors `EnforcedApplication<ComplexitySummary, AsymptoticClass>` referencing
//! `complexity_enforceable`, infer checks the named section using the **same**
//! `LensEnforcement` projection + violation relation as `complexity.dag`
//! (`complexity_enforcement_project` / `complexity_enforcement_violates`, surfaced
//! through `complexity_lens_generated`) and attaches a compile diagnostic on violation.

use std::collections::HashMap;

use crate::dag::{
    literal_decimal_i64, positive_descent_count, ArrowBody, AsymptoticClass, Dag, DeclarationId,
    FieldValue, LiteralBits, Lookup, PortId, PositiveDescentAmount, TypeConnective, ValueBody,
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
    let diagnostic_severity_disj = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("DiagnosticSeverity")
                && d.span.file.ends_with("lens_application.dag")
        })
        .map(|d| d.id);
    #[cfg(test)]
    if std::env::var_os("ENFORCED_LENS_DEBUG").is_some() {
        eprintln!(
            "check_enforced_lens: diagnostic_severity_disj = {:?}",
            diagnostic_severity_disj
        );
    }
    let Some(diagnostic_severity_disj) = diagnostic_severity_disj else {
        // Fail-closed: this pass consumes substrate `DiagnosticSeverity` from
        // `lens_application.dag`. Absence must not disable enforcement (openai-pro /
        // INVARIANTS P3, C-8).
        #[cfg(test)]
        eprintln!("enforced_lens: diagnostic severity substrate missing — attaching fail-closed diagnostic");
        dag.attach_diagnostic(Diagnostic::ParseError {
            message: "lens enforcement: could not resolve substrate `DiagnosticSeverity` from \
                      `lens_application.dag` (modeled authority missing; fail-closed)"
                .to_string(),
            span: dag.declaration(enforced_template).span.clone(),
            fixes: Vec::new(),
        });
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

        let observed = match complexity_of(dag, &port) {
            Lookup::Hit(s) => complexity_enforcement_project(&s),
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
        if !complexity_enforcement_violates(&budget_class, &observed) {
            continue;
        }
        let severity_val = match fm.get("diagnostic_severity") {
            Some(v) => *v,
            _ => {
                violations.push(Diagnostic::ParseError {
                    message:
                        "lens enforcement: `EnforcedApplication` missing `diagnostic_severity`"
                            .to_string(),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            }
        };
        let violation_message = format!(
            "lens enforcement violation: complexity budget {budget_class:?} exceeded \
             by observed class {observed:?} (declared `EnforcedApplication` at {})",
            decl.name.as_deref().unwrap_or("?")
        );
        violations.push(enforced_violation_diagnostic(
            dag,
            diagnostic_severity_disj,
            severity_val,
            violation_message,
            span,
        ));
    }

    for diagnostic in violations {
        dag.attach_diagnostic(diagnostic);
    }
}

fn field_map(fields: &[(String, FieldValue)]) -> HashMap<&str, &FieldValue> {
    fields.iter().map(|(k, v)| (k.as_str(), v)).collect()
}

/// Routes an enforce-mode **budget violation** through `EnforcedApplication.diagnostic_severity`.
///
/// Substrate policy (C-8): only `DiagnosticSeverity::Error` is a valid steady-state choice; unknown
/// constructors or malformed values fail closed with an explanatory diagnostic.
fn enforced_violation_diagnostic(
    dag: &Dag,
    diagnostic_severity_disj: DeclarationId,
    severity_val: &FieldValue,
    violation_message: String,
    span: SourceSpan,
) -> Diagnostic {
    let TypeConnective::Disj { variants } = &dag.declaration(diagnostic_severity_disj).connective
    else {
        return Diagnostic::ParseError {
            message: "lens enforcement: internal error (DiagnosticSeverity is not a sum type)"
                .to_string(),
            span,
            fixes: Vec::new(),
        };
    };
    let Some(error_ctor) = variants.iter().find(|v| v.label == "Error").map(|v| v.ty) else {
        return Diagnostic::ParseError {
            message: "lens enforcement: internal error (DiagnosticSeverity lacks `Error` variant)"
                .to_string(),
            span,
            fixes: Vec::new(),
        };
    };
    let FieldValue::Variant { constructor, .. } = severity_val else {
        return Diagnostic::ParseError {
            message: "lens enforcement: `diagnostic_severity` must be a `DiagnosticSeverity` \
                      variant value"
                .to_string(),
            span,
            fixes: Vec::new(),
        };
    };
    if *constructor != error_ctor {
        return Diagnostic::ParseError {
            message: "lens enforcement: `diagnostic_severity` on `EnforcedApplication` must be \
                      `Error` (INVARIANTS C-8; fail-closed discipline)"
                .to_string(),
            span,
            fixes: Vec::new(),
        };
    }
    Diagnostic::ParseError {
        message: violation_message,
        span,
        fixes: Vec::new(),
    }
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
                byte_start = literal_decimal_i64(n.as_str()).and_then(|v| u32::try_from(v).ok());
            }
            ("byte_end", FieldValue::Literal(LiteralBits::Int(n))) => {
                byte_end = literal_decimal_i64(n.as_str()).and_then(|v| u32::try_from(v).ok());
            }
            ("start", FieldValue::Literal(LiteralBits::Int(n))) => {
                byte_start = literal_decimal_i64(n.as_str()).and_then(|v| u32::try_from(v).ok());
            }
            ("end", FieldValue::Literal(LiteralBits::Int(n))) => {
                byte_end = literal_decimal_i64(n.as_str()).and_then(|v| u32::try_from(v).ok());
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
mod diagnostic_severity_fail_closed_tests {
    use super::*;
    use crate::dag::LiteralBits;

    #[test]
    fn attach_diagnostic_round_trips_through_diagnostic_table() {
        let mut dag = Dag::new();
        assert!(dag.diagnostics().is_empty());
        dag.attach_diagnostic(Diagnostic::ParseError {
            message: "probe".to_string(),
            span: SourceSpan::new("probe.v3", 0, 1),
            fixes: Vec::new(),
        });
        assert_eq!(dag.diagnostics().len(), 1);
    }

    #[test]
    fn check_enforced_lens_emits_diagnostic_when_diagnostic_severity_authority_unresolvable() {
        let mut dag = Dag::new();
        let Some(ds_id) = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("DiagnosticSeverity")
                    && d.span.file.ends_with("lens_application.dag")
            })
            .map(|d| d.id)
        else {
            panic!("bootstrap should declare DiagnosticSeverity in lens_application.dag");
        };
        dag.declaration_mut(ds_id).name = Some("DiagnosticSeverity__test_unresolvable".to_string());
        assert_eq!(
            dag.declaration(ds_id).name.as_deref(),
            Some("DiagnosticSeverity__test_unresolvable"),
            "rename should stick"
        );

        let section_ref_disj = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("SectionRef")
                    && d.span.file.ends_with("lens_application.dag")
            })
            .map(|d| d.id)
            .expect("SectionRef");
        let scope_ok =
            super::declaration_scope_payload_conj(&dag, section_ref_disj).is_some();
        let still_resolves_ds = dag.declarations().iter().any(|d| {
            d.name.as_deref() == Some("DiagnosticSeverity")
                && d.span.file.ends_with("lens_application.dag")
        });
        assert!(
            scope_ok,
            "declaration_scope_payload_conj should still resolve after DS rename"
        );
        assert!(
            !still_resolves_ds,
            "DiagnosticSeverity substrate row should be unfindable after rename"
        );

        let probe_ds = dag.declarations().iter().find(|d| {
            d.name.as_deref() == Some("DiagnosticSeverity")
                && d.span.file.ends_with("lens_application.dag")
        });
        assert!(
            probe_ds.is_none(),
            "probe before check: expected None, got {:?}",
            probe_ds.map(|d| (d.id, d.name.clone()))
        );

        check_enforced_lens_applications(&mut dag);
        assert!(
            dag.diagnostics().iter().any(|(_, d)| matches!(
                d,
                Diagnostic::ParseError { message, .. }
                    if message.contains("could not resolve substrate `DiagnosticSeverity`")
            )),
            "expected fail-closed diagnostic, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn enforced_violation_diagnostic_rejects_non_error_severity_constructor() {
        let dag = Dag::new();
        let Some(ds_disj) = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("DiagnosticSeverity")
                    && d.span.file.ends_with("lens_application.dag")
            })
            .map(|d| d.id)
        else {
            panic!("bootstrap should declare DiagnosticSeverity in lens_application.dag");
        };
        let wrong_ctor = dag
            .bool_runtime_variant_id(true)
            .expect("bootstrap True variant");
        let span = SourceSpan::new("t.v3", 0, 1);
        let diag = enforced_violation_diagnostic(
            &dag,
            ds_disj,
            &FieldValue::Variant {
                constructor: wrong_ctor,
                payload: Vec::new(),
            },
            "violation".to_string(),
            span.clone(),
        );
        let Diagnostic::ParseError { message, .. } = diag else {
            panic!("expected ParseError, got {diag:?}");
        };
        assert!(
            message.contains("must be `Error`"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn enforced_violation_diagnostic_rejects_non_variant_severity_value() {
        let dag = Dag::new();
        let Some(ds_disj) = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("DiagnosticSeverity")
                    && d.span.file.ends_with("lens_application.dag")
            })
            .map(|d| d.id)
        else {
            panic!("bootstrap should declare DiagnosticSeverity in lens_application.dag");
        };
        let diag = enforced_violation_diagnostic(
            &dag,
            ds_disj,
            &FieldValue::Literal(LiteralBits::Int("0".to_string())),
            "violation".to_string(),
            SourceSpan::new("t.v3", 0, 1),
        );
        let Diagnostic::ParseError { message, .. } = diag else {
            panic!("expected ParseError, got {diag:?}");
        };
        assert!(
            message.contains("must be a `DiagnosticSeverity` variant value"),
            "unexpected message: {message}"
        );
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
