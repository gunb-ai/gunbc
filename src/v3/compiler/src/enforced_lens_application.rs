//! Enforced lens applications (`EnforcedApplication` from `lens_application.dag`).
//!
//! Gate #91 (`enforce_violation_routing_landed`): budget-violation diagnostics are
//! routed through the authored `diagnostic_severity` field — today the single
//! substrate variant `DiagnosticSeverity::Error` maps to [`Diagnostic::ParseError`]
//! (compile-fail / fail-closed), per `lens_application.dag`, design §3, and
//! INVARIANTS C-8.
//!
//! Future extra `DiagnosticSeverity` constructors stay fail-closed at this consumer until
//! an intentional substrate + routing co-update assigns their `Diagnostic::*` shape.
//!
//! Gate #92 (`complexity_violation_compile_error_demonstrated`): when a program
//! authors `EnforcedApplication<ComplexitySummary, AsymptoticClass>` referencing
//! `complexity_enforceable`, infer checks the named section using the **same**
//! `LensEnforcement` projection + violation relation as `complexity.dag`
//! (`complexity_enforcement_project` / `complexity_enforcement_violates`, surfaced
//! through `complexity_lens_generated`) and attaches a compile diagnostic on violation.
//!
//! Gate #58 (`apply_lens_self_application_demonstrated`): when a program authors
//! `EnforcedApplication<TimingMeasurement, TimingBudget>` referencing `timing_enforceable`
//! (`v3.std.timing_lens`), infer reads the **lowered** `TimingMeasurement` from the
//! `DeclarationScope` subject's structural `measurement` field **only after** verifying the
//! subject row's **nominal** `data …: RowTy = …` type declares `measurement: TimingMeasurement`
//! on `RowTy` (API-level contract; avoids unrelated records that merely spell a `measurement`
//! label). The same usage ceiling as `timing_enforcement_project` / `timing_enforcement_violates`
//! in `timing_lens.dag` applies: non-`Observed` reports are enforced via a **variant-shaped**
//! fault path (never compared as ordinary wall-clock `Nat` against the fault sentinel), while
//! `Observed` wall-clock nanoseconds use the same strict `>` edge as gate #94 against
//! `TimingBudget.max`.

use std::collections::HashMap;

use crate::dag::{
    literal_decimal_i64, positive_descent_count, ArrowBody, AsymptoticClass, AtomPayload,
    Behavior, Dag, DeclarationId, FieldValue, LiteralBits, Lookup, PortId, PositiveDescentAmount,
    TypeConnective, ValueBody,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::lens_cost::{
    complexity_enforcement_project, complexity_enforcement_violates, complexity_of,
};

fn diagnostic_severity_substrate_disj(dag: &Dag) -> Option<DeclarationId> {
    dag.declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("DiagnosticSeverity")
                && d.span.file.ends_with("lens_application.dag")
        })
        .map(|d| d.id)
}

// Fail-closed when `lens_application.dag` does not expose the expected `DiagnosticSeverity`
// substrate row — factored so unit tests can pin the diagnostic without staging a full
// complexity-enforcement DAG (`Dag::new()` omits `complexity_enforceable`).
fn attach_missing_diagnostic_severity_substrate_diagnostic(
    dag: &mut Dag,
    enforced_template: DeclarationId,
) {
    dag.attach_diagnostic(Diagnostic::ParseError {
        message: "lens enforcement: could not resolve substrate `DiagnosticSeverity` from \
                  `lens_application.dag` (modeled authority missing; fail-closed)"
            .to_string(),
        span: dag.declaration(enforced_template).span.clone(),
        fixes: Vec::new(),
    });
}

/// Reads `timing_enforcement_fault_sentinel_count` from the lowered `timing_lens.dag` declaration
/// body (nullary `fn` → bind → [`ValueNode`]), matching substrate authority instead of duplicating
/// the decimal literal in Rust.
fn timing_enforcement_fault_sentinel_ns_from_substrate(dag: &Dag) -> Option<u64> {
    let decl = dag.declarations().iter().find(|d| {
        d.name.as_deref() == Some("timing_enforcement_fault_sentinel_count")
            && d.span.file.ends_with("timing_lens.dag")
    })?;
    let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &decl.connective
    else {
        return None;
    };
    let bind = (*bind_id).bind_opt(dag)?;
    let producer = dag.resolve_producer_opt(&bind.value)?;
    let Behavior::Value(vn) = producer else {
        return None;
    };
    let LiteralBits::Int(s) = &vn.data else {
        return None;
    };
    s.parse().ok()
}

fn timing_measurement_sum_type_decl_id(dag: &Dag) -> Option<DeclarationId> {
    dag.declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("TimingMeasurement")
                && d.span.file.ends_with("timing_lens.dag")
        })
        .map(|d| d.id)
}

fn peel_type_declaration_head(dag: &Dag, mut id: DeclarationId) -> DeclarationId {
    const MAX: usize = 64;
    for _ in 0..MAX {
        match &dag.declaration(id).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                id = *template;
            }
            TypeConnective::Atom(ap) => {
                let Some(next) = ap.resolved_id() else {
                    break;
                };
                id = next;
            }
            _ => break,
        }
    }
    id
}

/// Returns the nominal `RowTy` declaration id for a lowered `data row: RowTy = …` value row.
fn data_row_nominal_type_decl_id(dag: &Dag, value_decl_id: DeclarationId) -> Option<DeclarationId> {
    match &dag.declaration(value_decl_id).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } if arguments.is_empty() => Some(*template),
        _ => None,
    }
}

/// True when `RowTy` (after alias peel) is a conj that declares `measurement` with type peeling
/// to the substrate `TimingMeasurement` sum at `tm_disj`.
fn timing_section_row_type_declares_measurement_tm(
    dag: &Dag,
    section_value_decl_id: DeclarationId,
    tm_disj: DeclarationId,
) -> bool {
    let Some(row_ty) = data_row_nominal_type_decl_id(dag, section_value_decl_id) else {
        return false;
    };
    let ty_head = peel_type_declaration_head(dag, row_ty);
    let TypeConnective::Conj { children } = &dag.declaration(ty_head).connective else {
        return false;
    };
    let tm_head = peel_type_declaration_head(dag, tm_disj);
    children.iter().any(|f| {
        f.label == "measurement" && peel_type_declaration_head(dag, f.ty) == tm_head
    })
}

/// Lowered `TimingMeasurement` usage for timing `EnforcedApplication` enforcement.
///
/// Separates non-evidence variants from `Observed` wall-clock nanoseconds so a legitimate
/// `Observed` duration equal to the substrate fault sentinel `Nat` cannot be conflated with
/// `Unobserved` / `Ambiguous` / `Stale` (INVARIANTS P2 / modeling-discipline Practice 2).
///
/// Practice 4 (`docs/modeling-discipline.md`, coproduct checkpoint): **GREEN (terminal)** for this
/// PR — internal host projection mirroring `timing_enforcement_project` / `timing_lens.dag` for
/// executable `EnforcedApplication` checks only. Ledger: substrate `LensEnforcement` still carries
/// `TimingBudget` until the named `.dag` fault-budget carrier dissolves the sentinel encoding (see
/// `timing_lens.dag` gate #58 block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimingMeasurementEnforcementUsage {
    NonObservedFault,
    ObservedWallNs(u64),
}

/// Interprets a lowered `TimingMeasurement` value the same way `timing_enforcement_project` does
/// on the substrate (`timing_lens.dag`): `Observed` → wall-clock nanoseconds; other report variants
/// classify as [`TimingMeasurementEnforcementUsage::NonObservedFault`] (not as a raw `Nat`).
fn timing_measurement_enforcement_usage(
    dag: &Dag,
    tm_disj: DeclarationId,
    value: &FieldValue,
) -> Option<TimingMeasurementEnforcementUsage> {
    let TypeConnective::Disj { variants } = &dag.declaration(tm_disj).connective else {
        return None;
    };
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    let variant = variants.iter().find(|v| v.ty == *constructor)?;
    match variant.label.as_str() {
        "Observed" => observed_timing_payload_max_ns(payload)
            .map(TimingMeasurementEnforcementUsage::ObservedWallNs),
        "Unobserved" | "Ambiguous" | "Stale" => {
            Some(TimingMeasurementEnforcementUsage::NonObservedFault)
        }
        _ => None,
    }
}

/// Same reflexive edge as `timing_enforcement_violates` in `timing_lens.dag`, without collapsing
/// [`TimingMeasurementEnforcementUsage::NonObservedFault`] into a sentinel `Nat` comparison on the
/// `Observed` path.
pub(crate) fn timing_enforcement_violates(
    declared_max_ns: u64,
    usage: TimingMeasurementEnforcementUsage,
) -> bool {
    match usage {
        TimingMeasurementEnforcementUsage::NonObservedFault => true,
        TimingMeasurementEnforcementUsage::ObservedWallNs(ns) => ns > declared_max_ns,
    }
}

fn observed_timing_payload_max_ns(payload: &[FieldValue]) -> Option<u64> {
    let FieldValue::Record(parts) = payload.first()? else {
        return None;
    };
    if let Some((_, v)) = parts.iter().find(|(label, _)| label == "duration") {
        return field_value_nat_magnitude(v);
    }
    field_value_nat_magnitude(&FieldValue::Record(parts.clone()))
}

fn field_value_nat_magnitude(value: &FieldValue) -> Option<u64> {
    match value {
        FieldValue::Literal(LiteralBits::Int(s)) => s.parse().ok(),
        FieldValue::Record(parts) => parts
            .iter()
            .find(|(label, _)| label == "count")
            .and_then(|(_, v)| field_value_nat_magnitude(v)),
        _ => None,
    }
}

fn field_value_timing_budget_max_ns(value: &FieldValue) -> Option<u64> {
    let FieldValue::Record(parts) = value else {
        return None;
    };
    let max = parts
        .iter()
        .find(|(label, _)| label == "max")
        .map(|(_, v)| v)?;
    field_value_nat_magnitude(max)
}

fn is_timing_enforceable_lens_value(
    value: &FieldValue,
    timing_enforceable_id: Option<DeclarationId>,
    timing_lens_id: Option<DeclarationId>,
    timing_enforcement_id: Option<DeclarationId>,
) -> bool {
    if let (Some(bundle), FieldValue::Reference(id)) = (timing_enforceable_id, value) {
        if *id == bundle {
            return true;
        }
    }
    let (Some(lens_id), Some(enforcement_id)) = (timing_lens_id, timing_enforcement_id) else {
        return false;
    };
    let FieldValue::Record(parts) = value else {
        return false;
    };
    let mut lens_ref: Option<DeclarationId> = None;
    let mut enforcement_ref: Option<DeclarationId> = None;
    for (label, field) in parts {
        match (label.as_str(), field) {
            ("lens", FieldValue::Reference(r)) => lens_ref = Some(*r),
            ("enforcement", FieldValue::Reference(r)) => enforcement_ref = Some(*r),
            _ => {}
        }
    }
    lens_ref == Some(lens_id) && enforcement_ref == Some(enforcement_id)
}

/// Fail-closed check for landed complexity + timing enforcement applications.
///
/// Exported at the crate root (`v3_compiler::check_enforced_lens_applications`) so integration
/// tests can re-invoke the same post-infer pass as `infer::infer` without treating declaration
/// presence as proof the probe ran (gate #58 receipt).
pub fn check_enforced_lens_applications(dag: &mut Dag) {
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
    let complexity_enforceable_id = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("complexity_enforceable")
                && d.span.file.ends_with("complexity.dag")
        })
        .map(|d| d.id);
    let timing_enforceable_id = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("timing_enforceable")
                && d.span.file.ends_with("timing_lens.dag")
        })
        .map(|d| d.id);
    let timing_lens_id = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("timing_lens") && d.span.file.ends_with("timing_lens.dag")
        })
        .map(|d| d.id);
    let timing_enforcement_id = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("timing_enforcement")
                && d.span.file.ends_with("timing_lens.dag")
        })
        .map(|d| d.id);
    let asymptotic_disj = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("AsymptoticClass") && d.span.file.ends_with("algebra.dag")
        })
        .map(|d| d.id);
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
    let Some(diagnostic_severity_disj) = diagnostic_severity_substrate_disj(dag) else {
        attach_missing_diagnostic_severity_substrate_diagnostic(dag, enforced_template);
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
        let enforceable_lens = match fm.get("enforceable_lens") {
            Some(v) => v,
            _ => continue,
        };
        let section = match fm.get("section") {
            Some(v) => *v,
            _ => continue,
        };
        let budget_val = match fm.get("budget") {
            Some(v) => *v,
            _ => continue,
        };
        let span = match fm.get("span") {
            Some(FieldValue::Record(span_fields)) => record_as_source_span(span_fields),
            _ => None,
        }
        .unwrap_or_else(|| decl.span.clone());

        if let (Some(cid), FieldValue::Reference(ref_id)) =
            (complexity_enforceable_id, enforceable_lens)
        {
            if *ref_id == cid {
                let Some(asymptotic_disj) = asymptotic_disj else {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: could not resolve substrate `AsymptoticClass` from \
                              `algebra.dag` (modeled authority missing; fail-closed)"
                                .to_string(),
                        span: decl.span.clone(),
                        fixes: Vec::new(),
                    });
                    continue;
                };
                let Some(fn_decl) = resolve_declaration_scope_fn(
                    dag,
                    section,
                    section_ref_disj,
                    declaration_scope_conj,
                ) else {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: could not resolve function declaration for section"
                                .to_string(),
                        span: decl.span.clone(),
                        fixes: Vec::new(),
                    });
                    continue;
                };
                let Some(port) = fn_result_port(dag, fn_decl) else {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: could not resolve function result port for section"
                                .to_string(),
                        span: decl.span.clone(),
                        fixes: Vec::new(),
                    });
                    continue;
                };
                let Some(budget_class) = field_value_asymptotic_class(
                    dag,
                    asymptotic_disj,
                    positive_descent_disj,
                    budget_val,
                ) else {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: could not read complexity budget `AsymptoticClass` \
                              (ill-formed `ClassPolynomial` budgets must use Peano `degree` ≥ 3; \
                              use `ClassLinear` / `ClassQuadratic` for sub-cubic tiers)"
                                .to_string(),
                        span: decl.span.clone(),
                        fixes: Vec::new(),
                    });
                    continue;
                };

                let observed = match complexity_of(dag, &port) {
                    Lookup::Hit(s) => complexity_enforcement_project(&s),
                    Lookup::Miss => {
                        violations.push(Diagnostic::ParseError {
                            message:
                                "lens enforcement: complexity lens returned Miss for section — \
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
                continue;
            }
        }

        if is_timing_enforceable_lens_value(
            enforceable_lens,
            timing_enforceable_id,
            timing_lens_id,
            timing_enforcement_id,
        ) {
            let Some(tm_disj) = timing_measurement_sum_type_decl_id(dag) else {
                violations.push(Diagnostic::ParseError {
                    message:
                        "lens enforcement: could not resolve substrate `TimingMeasurement` from \
                              `timing_lens.dag` (modeled authority missing; fail-closed)"
                            .to_string(),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            };
            let Some(section_decl_id) =
                resolve_declaration_scope_declaration_id(section, declaration_scope_conj)
            else {
                violations.push(Diagnostic::ParseError {
                    message: "lens enforcement: could not resolve `DeclarationScope` for timing \
                              `EnforcedApplication` section"
                        .to_string(),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            };
            let section_decl = dag.declaration(section_decl_id);
            if !timing_section_row_type_declares_measurement_tm(dag, section_decl_id, tm_disj) {
                violations.push(Diagnostic::ParseError {
                    message: format!(
                        "lens enforcement: timing `EnforcedApplication` section `{}` must be a \
                         lowered `data …: RowTy = …` row whose nominal `RowTy` declares \
                         `measurement: TimingMeasurement` (same substrate contract as \
                         `timing_enforcement_project`)",
                        section_decl.name.as_deref().unwrap_or("?")
                    ),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            }
            let Some(body) = section_decl.value_body.as_ref() else {
                violations.push(Diagnostic::ParseError {
                    message: format!(
                        "lens enforcement: timing section `{}` has no lowered value body",
                        section_decl.name.as_deref().unwrap_or("?")
                    ),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            };
            let ValueBody::Structural { fields } = body else {
                violations.push(Diagnostic::ParseError {
                    message: format!(
                        "lens enforcement: timing section `{}` must lower to a structural record body",
                        section_decl.name.as_deref().unwrap_or("?")
                    ),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            };
            let section_fm = field_map(fields);
            let Some(measurement) = section_fm.get("measurement") else {
                violations.push(Diagnostic::ParseError {
                    message: format!(
                        "lens enforcement: timing section `{}` is missing required `measurement` field",
                        section_decl.name.as_deref().unwrap_or("?")
                    ),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            };
            let Some(usage_projection) =
                timing_measurement_enforcement_usage(dag, tm_disj, measurement)
            else {
                violations.push(Diagnostic::ParseError {
                    message: format!(
                        "lens enforcement: could not interpret `measurement` as lowered \
                         `TimingMeasurement` for section `{}`",
                        section_decl.name.as_deref().unwrap_or("?")
                    ),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            };
            let Some(declared_max_ns) = field_value_timing_budget_max_ns(budget_val) else {
                violations.push(Diagnostic::ParseError {
                    message: "lens enforcement: could not read timing budget `TimingBudget.max` \
                              nanoseconds (expected `TimingBudget { max: Nanoseconds { count } }`)"
                        .to_string(),
                    span: decl.span.clone(),
                    fixes: Vec::new(),
                });
                continue;
            };
            if !timing_enforcement_violates(declared_max_ns, usage_projection) {
                continue;
            };
            let usage_max_ns_for_message = match usage_projection {
                TimingMeasurementEnforcementUsage::ObservedWallNs(ns) => ns,
                TimingMeasurementEnforcementUsage::NonObservedFault => {
                    let Some(s) = timing_enforcement_fault_sentinel_ns_from_substrate(dag) else {
                        violations.push(Diagnostic::ParseError {
                            message:
                                "lens enforcement: could not resolve substrate `timing_enforcement_fault_sentinel_count` \
                                 from `timing_lens.dag` (needed for fault-path diagnostic text; fail-closed)"
                                    .to_string(),
                            span: decl.span.clone(),
                            fixes: Vec::new(),
                        });
                        continue;
                    };
                    s
                }
            };
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
                "lens enforcement violation: timing budget ceiling max_ns={declared_max_ns} exceeded \
                 by projected wall-clock usage max_ns={usage_max_ns_for_message} (declared `EnforcedApplication` \
                 at {})",
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
    }

    for diagnostic in violations {
        dag.attach_diagnostic(diagnostic);
    }
}

#[doc(hidden)]
fn gate_58_raise_observed_duration_ns_in_measurement(
    value: &mut FieldValue,
    duration_ns: u64,
) -> bool {
    match value {
        FieldValue::Variant { payload, .. } => {
            let Some(FieldValue::Record(parts)) = payload.first_mut() else {
                return false;
            };
            if let Some((_, FieldValue::Record(duration_parts))) =
                parts.iter_mut().find(|(label, _)| label == "duration")
            {
                return gate_58_set_count_field_to_ns(duration_parts, duration_ns);
            }
            // PB-1 bootstrap lowering: `Observed` payload can flatten to `{ count: Nat }` without
            // a nested `duration` record (authoring uses `Observed { duration: { count } }`).
            gate_58_set_count_field_to_ns(parts, duration_ns)
        }
        _ => false,
    }
}

fn gate_58_set_count_field_to_ns(parts: &mut [(String, FieldValue)], duration_ns: u64) -> bool {
    for (dlabel, dfield) in parts.iter_mut() {
        if dlabel == "count" {
            if let FieldValue::Literal(LiteralBits::Int(s)) = dfield {
                *s = duration_ns.to_string();
                return true;
            }
            return false;
        }
    }
    false
}

/// Integration receipt helper: mutates the PB-1 gate #58 witness row in a bootstrap [`Dag`].
///
/// **Not a supported production API** — only the `v3-compiler` integration test
/// `t_gate_58_apply_lens_self_application_test` should call this (exposed so that test binary can
/// link against `v3_compiler` without `cfg(test)` coupling).
#[doc(hidden)]
pub fn gate_58_test_raise_modeled_ci_timing_measurement_duration_ns(
    dag: &mut Dag,
    duration_ns: u64,
) -> Result<(), &'static str> {
    let id = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("gate_58_modeled_ci_timing_measurement")
                && d.span.file.ends_with("t_ci_workflow_as_data_demo.dag")
        })
        .map(|d| d.id)
        .ok_or("missing gate_58_modeled_ci_timing_measurement declaration")?;
    let decl = dag.declaration_mut(id);
    let Some(ValueBody::Structural { fields }) = decl.value_body.as_mut() else {
        return Err("gate_58 witness missing structural value_body");
    };
    for (label, value) in fields.iter_mut() {
        if label == "measurement"
            && gate_58_raise_observed_duration_ns_in_measurement(value, duration_ns)
        {
            return Ok(());
        }
    }
    Err("measurement field missing or not Observed with duration.count shape")
}

/// Parses `(declared_max_ns, projected_usage_max_ns)` from a timing lens **budget-ceiling**
/// violation message produced by [`check_enforced_lens_applications`].
///
/// The template is owned next to [`timing_enforcement_violates`] / `violation_message` formatting
/// in this module (`max_ns=` appears exactly twice: declared ceiling, then projected usage).
/// Integration tests use this instead of pinning English prose (see repo `TESTING.md` anti-pattern).
#[doc(hidden)]
pub fn gate_58_test_parse_timing_budget_violation_max_ns_pair(message: &str) -> Option<(u64, u64)> {
    const NEEDLE: &str = "max_ns=";
    let mut found: Vec<u64> = Vec::with_capacity(2);
    let mut rest = message;
    while let Some(pos) = rest.find(NEEDLE) {
        let after = pos + NEEDLE.len();
        let tail = &rest[after..];
        let digit_len = tail
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .map(|c| c.len_utf8())
            .sum::<usize>();
        if digit_len == 0 {
            return None;
        }
        let n: u64 = tail[..digit_len].parse().ok()?;
        found.push(n);
        rest = &tail[digit_len..];
    }
    if found.len() == 2 {
        Some((found[0], found[1]))
    } else {
        None
    }
}

fn field_map(fields: &[(String, FieldValue)]) -> HashMap<&str, &FieldValue> {
    fields.iter().map(|(k, v)| (k.as_str(), v)).collect()
}

/// Routes an enforce-mode **budget violation** through `EnforcedApplication.diagnostic_severity`.
///
/// Substrate policy (C-8): only `DiagnosticSeverity::Error` is a valid steady-state choice; unknown
/// constructors, non-nullary `Error` payload, or malformed values fail closed with an explanatory
/// diagnostic.
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
    let FieldValue::Variant {
        constructor,
        payload,
    } = severity_val
    else {
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
    // `lens_application.dag`: `type DiagnosticSeverity = Error` — the lone variant is nullary.
    if !payload.is_empty() {
        return Diagnostic::ParseError {
            message: "lens enforcement: `DiagnosticSeverity::Error` must be nullary (INVARIANTS \
                      P1; malformed variant payload)"
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

fn resolve_declaration_scope_declaration_id(
    section: &FieldValue,
    declaration_scope_conj: DeclarationId,
) -> Option<DeclarationId> {
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
    match payload.as_slice() {
        [FieldValue::Reference(id)] => Some(*id),
        [FieldValue::Record(parts)] => {
            let mut decl_id: Option<DeclarationId> = None;
            for (label, value) in parts {
                if label == "declaration" {
                    if let FieldValue::Reference(id) = value {
                        decl_id = Some(*id);
                    }
                }
            }
            decl_id
        }
        _ => None,
    }
}

fn resolve_declaration_scope_fn(
    dag: &Dag,
    section: &FieldValue,
    section_ref_disj: DeclarationId,
    declaration_scope_conj: DeclarationId,
) -> Option<DeclarationId> {
    let _ = section_ref_disj;
    let id = resolve_declaration_scope_declaration_id(section, declaration_scope_conj)?;
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
    use crate::dag::{LiteralBits, TypeConnective};
    use crate::diagnostics::Diagnostic;

    fn assert_parse_error_at(diag: Diagnostic, expected_span: &SourceSpan) {
        match diag {
            Diagnostic::ParseError { span, fixes, .. } => {
                assert_eq!(&span, expected_span);
                assert!(fixes.is_empty());
            }
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    #[test]
    fn missing_diagnostic_severity_substrate_records_fail_closed_diagnostic() {
        let mut dag = Dag::new();
        let Some(enforced_template) = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("EnforcedApplication")
                    && d.span.file.ends_with("lens_application.dag")
            })
            .map(|d| d.id)
        else {
            panic!("bootstrap should declare EnforcedApplication in lens_application.dag");
        };
        let expected_span = dag.declaration(enforced_template).span.clone();
        let Some(ds_id) = super::diagnostic_severity_substrate_disj(&dag) else {
            panic!("bootstrap should declare DiagnosticSeverity in lens_application.dag");
        };
        dag.declaration_mut(ds_id).name = Some("DiagnosticSeverity__test_unresolvable".to_string());
        assert!(
            super::diagnostic_severity_substrate_disj(&dag).is_none(),
            "Expected DiagnosticSeverity substrate row to be unresolvable after rename"
        );
        super::attach_missing_diagnostic_severity_substrate_diagnostic(&mut dag, enforced_template);
        let mut it = dag.diagnostics().iter();
        let (_, d) = it.next().expect("fail-closed diagnostic");
        assert!(it.next().is_none(), "expected exactly one diagnostic");
        assert_parse_error_at(d.clone(), &expected_span);
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
        let expected_span = SourceSpan::new("t.v3", 0, 1);
        let diag = enforced_violation_diagnostic(
            &dag,
            ds_disj,
            &FieldValue::Variant {
                constructor: wrong_ctor,
                payload: Vec::new(),
            },
            "violation".to_string(),
            expected_span.clone(),
        );
        assert_parse_error_at(diag, &expected_span);
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
        let expected_span = SourceSpan::new("t.v3", 0, 1);
        let diag = enforced_violation_diagnostic(
            &dag,
            ds_disj,
            &FieldValue::Literal(LiteralBits::Int("0".to_string())),
            "violation".to_string(),
            expected_span.clone(),
        );
        assert_parse_error_at(diag, &expected_span);
    }

    #[test]
    fn enforced_violation_diagnostic_rejects_error_severity_with_non_nullary_payload() {
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
        let TypeConnective::Disj { variants } = &dag.declaration(ds_disj).connective else {
            panic!("bootstrap DiagnosticSeverity should be a sum type");
        };
        let error_ctor = variants
            .iter()
            .find(|v| v.label == "Error")
            .map(|v| v.ty)
            .expect("bootstrap DiagnosticSeverity should have Error variant");
        let expected_span = SourceSpan::new("t.v3", 0, 1);
        let diag = enforced_violation_diagnostic(
            &dag,
            ds_disj,
            &FieldValue::Variant {
                constructor: error_ctor,
                payload: vec![FieldValue::Literal(LiteralBits::Int("0".to_string()))],
            },
            "violation".to_string(),
            expected_span.clone(),
        );
        assert_parse_error_at(diag, &expected_span);
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

#[cfg(test)]
mod gate_58_timing_enforcement_unit_tests {
    use super::{
        gate_58_test_parse_timing_budget_violation_max_ns_pair,
        timing_enforcement_fault_sentinel_ns_from_substrate, timing_enforcement_violates,
        TimingMeasurementEnforcementUsage,
    };
    use crate::dag::Dag;

    #[test]
    fn gate_58_parse_max_ns_pair_matches_timing_budget_violation_template() {
        let msg = "lens enforcement violation: timing budget ceiling max_ns=1 exceeded \
                 by projected wall-clock usage max_ns=999 (declared `EnforcedApplication` \
                 at gate_58_enforcement_budget_violation)";
        assert_eq!(
            gate_58_test_parse_timing_budget_violation_max_ns_pair(msg),
            Some((1, 999))
        );
    }

    #[test]
    fn non_observed_fault_always_violates_under_any_finite_budget() {
        assert!(timing_enforcement_violates(
            1,
            TimingMeasurementEnforcementUsage::NonObservedFault
        ));
        assert!(timing_enforcement_violates(
            u64::MAX - 1,
            TimingMeasurementEnforcementUsage::NonObservedFault
        ));
    }

    #[test]
    fn observed_usage_strictly_exceeding_budget_violates() {
        assert!(timing_enforcement_violates(
            500,
            TimingMeasurementEnforcementUsage::ObservedWallNs(1000)
        ));
    }

    #[test]
    fn observed_usage_within_or_equal_budget_is_clean() {
        assert!(!timing_enforcement_violates(
            1000,
            TimingMeasurementEnforcementUsage::ObservedWallNs(500)
        ));
        assert!(!timing_enforcement_violates(
            1000,
            TimingMeasurementEnforcementUsage::ObservedWallNs(1000)
        ));
    }

    #[test]
    fn observed_wall_at_substrate_fault_sentinel_is_not_fault_short_circuit() {
        let dag = Dag::new();
        let s = timing_enforcement_fault_sentinel_ns_from_substrate(&dag)
            .expect("bootstrap should carry timing_lens fault sentinel");
        assert!(!timing_enforcement_violates(
            s + 1,
            TimingMeasurementEnforcementUsage::ObservedWallNs(s)
        ));
        assert!(timing_enforcement_violates(
            s - 1,
            TimingMeasurementEnforcementUsage::ObservedWallNs(s)
        ));
    }
}
