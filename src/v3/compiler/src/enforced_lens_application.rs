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
//! `EnforcedApplication<TimingMeasurement, TimingBudget, TimingEnforcementProjected>` referencing `timing_enforceable`
//! (`v3.std.timing_lens`), infer reads the **lowered** `TimingMeasurement` from the
//! `DeclarationScope` subject's structural `measurement` field **only after** verifying the
//! subject row's **nominal** `data …: RowTy = …` type declares `measurement: TimingMeasurement`
//! on `RowTy` (API-level contract; avoids unrelated records that merely spell a `measurement`
//! label). The same usage ceiling as `timing_enforcement_project` / `timing_enforcement_violates`
//! in `timing_lens.dag` applies: non-`Observed` reports are enforced via a **variant-shaped**
//! fault path (never compared as ordinary wall-clock `Nat` against the fault sentinel), while
//! `Observed` wall-clock nanoseconds use the same strict `>` edge as gate #94 against
//! `TimingBudget.max`.
//!
//! Gate #95 (`opt_in_iteration_parallelism_via_lens_application_demonstrated`): when a program
//! authors `EnforcedApplication<ParallelismMode, ParallelismMode, ParallelismMode>` referencing
//! `parallelism_enforceable` (`lenses.parallelism`) with a [`SectionRef::NodeScope`] section, infer
//! observes **cross-iteration parallel emission** eligibility via authored
//! [`crate::lens_parallelism::parallelism_iteration_observed_mode`] (`parallelism.dag`) on the scope's
//! `NodeId` — identical to lens `parallelism_lens.read`'s substrate surface and the staged
//! `loop_iteration_parallel_emission_indicator`. Budget violations delegate to authored
//! `parallelism_enforcement_violates` via [`crate::lens_declaration_apply::apply_lens_declaration`];
//! host code does not remap raw indicator integers independently.

use std::collections::HashMap;

use crate::dag::{
    literal_decimal_i64, positive_descent_count, ArrowBody, AsymptoticClass, Behavior, Dag,
    DeclarationId, FieldValue, LiteralBits, Lookup, NodeId, PortId, PositiveDescentAmount,
    TypeConnective, ValueBody,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::lens_cost::{
    complexity_enforcement_project, complexity_enforcement_violates, complexity_of,
};
use crate::lens_declaration_apply::{apply_lens_declaration, LensApplyError};

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
        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
            "EnforcedLensApplicationDiagnostic",
        ),
    });
}

fn timing_lens_gate_58_retirement_correction() -> crate::diagnostics::Correction {
    crate::diagnostics::Correction::deferred(
        "Gate #58 timing enforcement does not yet retain a source span for a safe timing witness rewrite",
        "R3 Gap 9 row #106 timing-lens diagnostic roundtrip",
        "thread timing measurement/budget source spans through gate #58 enforcement and replace this scaffold diagnostic with a generated/data-backed LiveCorrection",
    )
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
    children
        .iter()
        .any(|f| f.label == "measurement" && peel_type_declaration_head(dag, f.ty) == tm_head)
}

/// Lowered `TimingMeasurement` usage for timing `EnforcedApplication` enforcement.
///
/// Separates non-evidence variants from `Observed` wall-clock nanoseconds so a legitimate
/// `Observed` duration equal to the substrate fault sentinel `Nat` cannot be conflated with
/// `Unobserved` / `Ambiguous` / `Stale` (INVARIANTS P2 / modeling-discipline Practice 2).
///
/// Practice 4 (`docs/modeling-discipline.md`, coproduct checkpoint): **GREEN (terminal)** — mirrors
/// `timing_enforcement_project` / `timing_enforcement_violates` in `timing_lens.dag` on the lowered
/// `TimingMeasurement` witness (substrate `LensEnforcement.violates` now takes `TimingMeasurement`
/// × `TimingBudget` per `lens_application.dag`; gate #58 / codex 10994).
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

/// Same reflexive edge as `timing_enforcement_violates` in `timing_lens.dag` on the lowered witness,
/// using the same `>` edge as substrate `Observed` enforcement (no `TimingBudget` sentinel equality
/// on the observed path).
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

/// Local bookkeeping parallel to authored [`crate::lens_parallelism::ParallelismMode`] for
/// substrate `apply_lens_declaration` witnesses (DSL sum ↔ [`FieldValue::Variant`]).
///
/// Observations must come only from [`crate::lens_parallelism::parallelism_iteration_observed_mode`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParallelismIterationBudget {
    OptInIndependent,
    Sequential,
}

fn parallelism_iteration_budget_from_substrate(
    observed: crate::lens_parallelism::ParallelismMode,
) -> ParallelismIterationBudget {
    match observed {
        crate::lens_parallelism::ParallelismMode::OptInIndependent => {
            ParallelismIterationBudget::OptInIndependent
        }
        crate::lens_parallelism::ParallelismMode::Sequential => {
            ParallelismIterationBudget::Sequential
        }
    }
}

fn parallelism_mode_disj_decl_id(dag: &Dag) -> Option<DeclarationId> {
    dag.declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("ParallelismMode") && d.span.file.ends_with("parallelism.dag")
        })
        .map(|d| d.id)
}

fn parallelism_enforcement_violates_decl_id(dag: &Dag) -> Option<DeclarationId> {
    dag.declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("parallelism_enforcement_violates")
                && d.span.file.ends_with("parallelism.dag")
        })
        .map(|d| d.id)
}

fn parallelism_iteration_budget_as_variant_field_value(
    dag: &Dag,
    pm_disj: DeclarationId,
    budget: ParallelismIterationBudget,
) -> Option<FieldValue> {
    let TypeConnective::Disj { variants } = &dag.declaration(pm_disj).connective else {
        return None;
    };
    let label = match budget {
        ParallelismIterationBudget::OptInIndependent => "OptInIndependent",
        ParallelismIterationBudget::Sequential => "Sequential",
    };
    let ctor = variants.iter().find(|v| v.label == label)?.ty;
    Some(FieldValue::Variant {
        constructor: ctor,
        payload: Vec::new(),
    })
}

fn parallelism_enforcement_violates_via_substrate(
    dag: &Dag,
    pm_disj: DeclarationId,
    observed: ParallelismIterationBudget,
    declared: ParallelismIterationBudget,
) -> Result<bool, LensApplyError> {
    let lens_id = parallelism_enforcement_violates_decl_id(dag).ok_or(
        LensApplyError::SubstrateReflect("parallelism_enforcement_violates declaration"),
    )?;
    let observed_fv = parallelism_iteration_budget_as_variant_field_value(dag, pm_disj, observed)
        .ok_or(LensApplyError::SubstrateReflect(
        "ParallelismMode observed variant",
    ))?;
    let declared_fv = parallelism_iteration_budget_as_variant_field_value(dag, pm_disj, declared)
        .ok_or(LensApplyError::SubstrateReflect(
        "ParallelismMode declared variant",
    ))?;
    let out = apply_lens_declaration(dag, None, lens_id, &[observed_fv, declared_fv])?;
    match out {
        FieldValue::Literal(LiteralBits::Bool(b)) => Ok(b),
        _ => Err(LensApplyError::TypeMismatch(
            "parallelism_enforcement_violates Bool output",
        )),
    }
}

fn section_ref_node_scope_constructor_id(
    dag: &Dag,
    section_ref_disj: DeclarationId,
) -> Option<DeclarationId> {
    let TypeConnective::Disj { variants } = &dag.declaration(section_ref_disj).connective else {
        return None;
    };
    variants
        .iter()
        .find(|v| v.label == "NodeScope")
        .map(|v| v.ty)
}

fn field_value_parallelism_iteration_budget(
    dag: &Dag,
    disj: DeclarationId,
    value: &FieldValue,
) -> Option<ParallelismIterationBudget> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    if !payload.is_empty() {
        return None;
    }
    let TypeConnective::Disj { variants } = &dag.declaration(disj).connective else {
        return None;
    };
    let label = variants
        .iter()
        .find(|v| v.ty == *constructor)?
        .label
        .as_str();
    match label {
        "OptInIndependent" => Some(ParallelismIterationBudget::OptInIndependent),
        "Sequential" => Some(ParallelismIterationBudget::Sequential),
        _ => None,
    }
}

fn resolve_node_scope_section(
    _dag: &Dag,
    section: &FieldValue,
    node_scope_conj: DeclarationId,
) -> Option<(DeclarationId, NodeId)> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = section
    else {
        return None;
    };
    if *constructor != node_scope_conj {
        return None;
    }
    let parts: &[(String, FieldValue)] = match payload.as_slice() {
        [FieldValue::Record(rec)] => rec.as_slice(),
        _ => return None,
    };
    let mut decl_ref: Option<DeclarationId> = None;
    let mut node_idx: Option<u32> = None;
    for (k, v) in parts {
        if k == "declaration" {
            if let FieldValue::Reference(id) = v {
                decl_ref = Some(*id);
            }
        } else if k == "node" {
            if let FieldValue::Literal(LiteralBits::Int(s)) = v {
                node_idx = s.parse().ok();
            }
        }
    }
    Some((decl_ref?, NodeId::from_table_index(node_idx?)))
}

/// Returns `true` when an authored **`ParallelismMode::OptInIndependent`** budget violates the
/// observed iteration-parallelism emission contract from authored
/// `parallelism_iteration_observed_mode` (Lane‑2 `loop_iteration_parallel_emission_indicator`:
/// indicator `1` ⇒ `OptInIndependent`; otherwise `Sequential`).
///
/// **Gate #95 interim surface** (see `docs/design-lens-application-surface.md` §4.4): pairs with
/// [`check_enforced_lens_applications`] for full `EnforcedApplication`/`NodeScope` routing.
///
/// Requires `dag` to carry lowered `parallelism.dag` (`parallelism_enforcement_violates`);
/// observes via [`crate::lens_parallelism::parallelism_iteration_observed_mode`]. Delegates violation
/// semantics to that substrate via [`apply_lens_declaration`].
///
/// Returns [`Err`] when `parallelism.dag` is absent from this [`Dag`] or substrate evaluation fails
/// ([`LensApplyError`]) — no panics on arbitrary caller-supplied graphs (CODING.md § Hidden panic surface).
pub fn parallelism_iteration_opt_in_enforcement_violates(
    dag: &Dag,
    workflow_root: NodeId,
) -> Result<bool, LensApplyError> {
    let pm_disj = parallelism_mode_disj_decl_id(dag).ok_or(LensApplyError::SubstrateReflect(
        "ParallelismMode from parallelism.dag",
    ))?;
    let observed = parallelism_iteration_budget_from_substrate(
        crate::lens_parallelism::parallelism_iteration_observed_mode(dag, workflow_root),
    );
    parallelism_enforcement_violates_via_substrate(
        dag,
        pm_disj,
        observed,
        ParallelismIterationBudget::OptInIndependent,
    )
}

/// Fail-closed check for landed complexity, timing, and iteration-opt-in parallelism enforcement
/// applications (`EnforcedApplication` rows).
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
    let Some(node_scope_conj) = section_ref_node_scope_constructor_id(dag, section_ref_disj) else {
        return;
    };
    let parallelism_enforceable_id = dag
        .declarations()
        .iter()
        .find(|d| {
            d.name.as_deref() == Some("parallelism_enforceable")
                && d.span.file.ends_with("parallelism.dag")
        })
        .map(|d| d.id);
    let parallelism_mode_disj = parallelism_mode_disj_decl_id(dag);
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
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
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
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
                    });
                    continue;
                };
                let Some(port) = fn_result_port(dag, fn_decl) else {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: could not resolve function result port for section"
                                .to_string(),
                        span: decl.span.clone(),
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
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
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
                    });
                    continue;
                };

                let summary = match complexity_of(dag, &port) {
                    Lookup::Hit(s) => s,
                    Lookup::Miss => {
                        violations.push(Diagnostic::ParseError {
                            message:
                                "lens enforcement: complexity lens returned Miss for section — \
                                  cannot enforce budget"
                                    .to_string(),
                            span: span.clone(),
                            correction:
                                crate::diagnostics::Correction::deferred_for_diagnostic_class(
                                    "EnforcedLensApplicationDiagnostic",
                                ),
                        });
                        continue;
                    }
                };
                if !complexity_enforcement_violates(&summary, &budget_class) {
                    continue;
                }
                let observed = complexity_enforcement_project(&summary);
                let severity_val = match fm.get("diagnostic_severity") {
                    Some(v) => *v,
                    _ => {
                        violations.push(Diagnostic::ParseError {
                            message:
                                "lens enforcement: `EnforcedApplication` missing `diagnostic_severity`"
                                    .to_string(),
                            span: decl.span.clone(),
                            correction: crate::diagnostics::Correction::deferred_for_diagnostic_class("EnforcedLensApplicationDiagnostic"),
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
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                        "EnforcedLensApplicationDiagnostic",
                    ),
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
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                        "EnforcedLensApplicationDiagnostic",
                    ),
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
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                        "EnforcedLensApplicationDiagnostic",
                    ),
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
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                        "EnforcedLensApplicationDiagnostic",
                    ),
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
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class("EnforcedLensApplicationDiagnostic"),
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
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class("EnforcedLensApplicationDiagnostic"),
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
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                        "EnforcedLensApplicationDiagnostic",
                    ),
                });
                continue;
            };
            let Some(declared_max_ns) = field_value_timing_budget_max_ns(budget_val) else {
                violations.push(Diagnostic::ParseError {
                    message: "lens enforcement: could not read timing budget `TimingBudget.max` \
                              nanoseconds (expected `TimingBudget { max: Nanoseconds { count } }`)"
                        .to_string(),
                    span: decl.span.clone(),
                    correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                        "EnforcedLensApplicationDiagnostic",
                    ),
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
                            correction: crate::diagnostics::Correction::deferred_for_diagnostic_class("EnforcedLensApplicationDiagnostic"),
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
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
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
                span.clone(),
            ));
        }

        if let (Some(peid), FieldValue::Reference(pe_ref)) =
            (parallelism_enforceable_id, enforceable_lens)
        {
            if *pe_ref == peid {
                let Some(pm_disj) = parallelism_mode_disj else {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: could not resolve substrate `ParallelismMode` from \
                             `parallelism.dag` (modeled authority missing; fail-closed)"
                                .to_string(),
                        span: decl.span.clone(),
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
                    });
                    continue;
                };
                let Some((fn_decl, subject_node)) =
                    resolve_node_scope_section(dag, section, node_scope_conj)
                else {
                    violations.push(Diagnostic::ParseError {
                        message: "lens enforcement: parallelism `EnforcedApplication` requires a \
                                  `SectionRef::NodeScope { declaration, node }` section (see \
                                  docs/design-lens-application-surface.md §4.4)"
                            .to_string(),
                        span: decl.span.clone(),
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
                    });
                    continue;
                };
                if !matches!(
                    &dag.declaration(fn_decl).connective,
                    TypeConnective::Arrow { .. }
                ) {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: parallelism `NodeScope.declaration` must name a \
                                  function (`Arrow` declaration)"
                                .to_string(),
                        span: decl.span.clone(),
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
                    });
                    continue;
                };
                if !crate::dag::node_scope_subject_within_arrow_declaration(
                    dag,
                    fn_decl,
                    subject_node,
                ) {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: parallelism `SectionRef::NodeScope.node` must name a \
                                   substrate node in the authored `declaration` body's lowered graph \
                                   (declaring function bind or descendant reachable from its result \
                                   subgraph); malformed `(declaration, node)` pairs fail closed instead \
                                   of reading `loop_iteration_parallel_emission_indicator` out-of-scope"
                                .to_string(),
                        span: decl.span.clone(),
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
                    });
                    continue;
                }
                let Some(budget_host) =
                    field_value_parallelism_iteration_budget(dag, pm_disj, budget_val)
                else {
                    violations.push(Diagnostic::ParseError {
                        message:
                            "lens enforcement: could not read parallelism budget `ParallelismMode` \
                                  (expected nullary `OptInIndependent` or `Sequential`)"
                                .to_string(),
                        span: decl.span.clone(),
                        correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                            "EnforcedLensApplicationDiagnostic",
                        ),
                    });
                    continue;
                };
                let indicator =
                    crate::loop_iteration_parallel_emission_indicator(dag, subject_node);
                let observed_host = parallelism_iteration_budget_from_substrate(
                    crate::lens_parallelism::parallelism_iteration_observed_mode(dag, subject_node),
                );
                let violates = match parallelism_enforcement_violates_via_substrate(
                    dag,
                    pm_disj,
                    observed_host,
                    budget_host,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        violations.push(Diagnostic::ParseError {
                            message: format!(
                                "lens enforcement: could not evaluate `parallelism_enforcement_violates` \
                                 from `parallelism.dag` (fail-closed): {e:?}"
                            ),
                            span: decl.span.clone(),
                            correction:
                                crate::diagnostics::Correction::deferred_for_diagnostic_class(
                                    "EnforcedLensApplicationDiagnostic",
                                ),
                        });
                        continue;
                    }
                };
                if !violates {
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
                            correction:
                                crate::diagnostics::Correction::deferred_for_diagnostic_class(
                                    "EnforcedLensApplicationDiagnostic",
                                ),
                        });
                        continue;
                    }
                };
                let violation_message = format!(
                    "lens enforcement violation: parallelism iteration `ParallelismMode` budget {budget_host:?} inconsistent \
                     with `loop_iteration_parallel_emission_indicator={indicator}` observation (declared `EnforcedApplication` \
                     at {})",
                    decl.name.as_deref().unwrap_or("?")
                );
                violations.push(enforced_violation_diagnostic(
                    dag,
                    diagnostic_severity_disj,
                    severity_val,
                    violation_message,
                    span.clone(),
                ));
            }
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
            correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                "EnforcedLensApplicationDiagnostic",
            ),
        };
    };
    let Some(error_ctor) = variants.iter().find(|v| v.label == "Error").map(|v| v.ty) else {
        return Diagnostic::ParseError {
            message: "lens enforcement: internal error (DiagnosticSeverity lacks `Error` variant)"
                .to_string(),
            span,
            correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                "EnforcedLensApplicationDiagnostic",
            ),
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
            correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                "EnforcedLensApplicationDiagnostic",
            ),
        };
    };
    if *constructor != error_ctor {
        return Diagnostic::ParseError {
            message: "lens enforcement: `diagnostic_severity` on `EnforcedApplication` must be \
                      `Error` (INVARIANTS C-8; fail-closed discipline)"
                .to_string(),
            span,
            correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                "EnforcedLensApplicationDiagnostic",
            ),
        };
    }
    // `lens_application.dag`: `type DiagnosticSeverity = Error` — the lone variant is nullary.
    if !payload.is_empty() {
        return Diagnostic::ParseError {
            message: "lens enforcement: `DiagnosticSeverity::Error` must be nullary (INVARIANTS \
                      P1; malformed variant payload)"
                .to_string(),
            span,
            correction: crate::diagnostics::Correction::deferred_for_diagnostic_class(
                "EnforcedLensApplicationDiagnostic",
            ),
        };
    }
    Diagnostic::ParseError {
        message: violation_message,
        span,
        correction: timing_lens_gate_58_retirement_correction(),
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
                if *template == enforced_template && arguments.len() == 3 {
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
            Diagnostic::ParseError {
                span, correction, ..
            } => {
                assert_eq!(&span, expected_span);
                assert!(matches!(
                    correction,
                    crate::diagnostics::Correction::DeferredCorrection { .. }
                ));
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

    #[test]
    fn enforce_violation_routing_landed_routes_error_severity_to_parse_error() {
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
        let expected_span = SourceSpan::new("gate_91.dag", 10, 20);
        let diag = enforced_violation_diagnostic(
            &dag,
            ds_disj,
            &FieldValue::Variant {
                constructor: error_ctor,
                payload: Vec::new(),
            },
            "lens enforcement violation: gate #91".to_string(),
            expected_span.clone(),
        );
        match diag {
            Diagnostic::ParseError { message, span, .. } => {
                assert_eq!(span, expected_span);
                assert_eq!(message, "lens enforcement violation: gate #91");
            }
            other => panic!("expected ParseError, got {other:?}"),
        }
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

#[cfg(test)]
mod gate_95_parallelism_iteration_enforcement_tests {
    use super::*;
    use crate::compile_to_dag;
    use crate::dag::{Declaration, TemplateArgument, TypeConnective, ValueBody};
    use crate::diagnostics::Diagnostic;

    fn push_parallelism_iteration_enforced_declaration(
        dag: &mut Dag,
        witness_name: &str,
        fn_decl: DeclarationId,
        subject: NodeId,
    ) {
        let enforced_template = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("EnforcedApplication")
                    && d.span.file.ends_with("lens_application.dag")
            })
            .expect("EnforcedApplication")
            .id;
        let pm_disj = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("ParallelismMode")
                    && d.span.file.ends_with("parallelism.dag")
            })
            .expect("ParallelismMode")
            .id;
        let parallelism_enforceable = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("parallelism_enforceable")
                    && d.span.file.ends_with("parallelism.dag")
            })
            .expect("parallelism_enforceable")
            .id;
        let section_ref_disj = dag
            .declarations()
            .iter()
            .find(|d| {
                d.name.as_deref() == Some("SectionRef")
                    && d.span.file.ends_with("lens_application.dag")
            })
            .expect("SectionRef")
            .id;
        let node_scope_ctor =
            section_ref_node_scope_constructor_id(dag, section_ref_disj).expect("NodeScope");
        let TypeConnective::Disj {
            variants: pm_variants,
        } = &dag.declaration(pm_disj).connective
        else {
            panic!("ParallelismMode sum");
        };
        let optin_ctor = pm_variants
            .iter()
            .find(|v| v.label == "OptInIndependent")
            .expect("OptInIndependent ctor")
            .ty;
        let ds_disj =
            diagnostic_severity_substrate_disj(dag).expect("DiagnosticSeverity substrate");
        let TypeConnective::Disj {
            variants: ds_variants,
        } = &dag.declaration(ds_disj).connective
        else {
            panic!("DiagnosticSeverity sum");
        };
        let error_ctor = ds_variants
            .iter()
            .find(|v| v.label == "Error")
            .expect("DiagnosticSeverity.Error")
            .ty;

        let template_params = dag.declaration(enforced_template).type_params.clone();
        assert_eq!(
            template_params.len(),
            3,
            "EnforcedApplication exposes Output/Budget/Projected formal parameters"
        );
        let instantiation_arguments: Vec<TemplateArgument> = template_params
            .into_iter()
            .map(|parameter| TemplateArgument {
                parameter,
                value: pm_disj,
            })
            .collect();

        let new_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: new_id,
            name: Some(witness_name.to_string()),
            connective: TypeConnective::Instantiation {
                template: enforced_template,
                arguments: instantiation_arguments,
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: Some(new_id),
            specialization_parent: None,
            inhabits: None,
            value_body: Some(ValueBody::Structural {
                fields: vec![
                    (
                        "enforceable_lens".to_string(),
                        FieldValue::Reference(parallelism_enforceable),
                    ),
                    (
                        "section".to_string(),
                        FieldValue::Variant {
                            constructor: node_scope_ctor,
                            payload: vec![FieldValue::Record(vec![
                                ("declaration".to_string(), FieldValue::Reference(fn_decl)),
                                (
                                    "node".to_string(),
                                    FieldValue::Literal(LiteralBits::Int(
                                        subject.raw().to_string(),
                                    )),
                                ),
                            ])],
                        },
                    ),
                    (
                        "budget".to_string(),
                        FieldValue::Variant {
                            constructor: optin_ctor,
                            payload: Vec::new(),
                        },
                    ),
                    (
                        "diagnostic_severity".to_string(),
                        FieldValue::Variant {
                            constructor: error_ctor,
                            payload: Vec::new(),
                        },
                    ),
                    (
                        "span".to_string(),
                        FieldValue::Record(vec![
                            (
                                "file".to_string(),
                                FieldValue::Literal(LiteralBits::String(
                                    "gate95_injected.dag".to_string(),
                                )),
                            ),
                            (
                                "start".to_string(),
                                FieldValue::Literal(LiteralBits::Int("0".to_string())),
                            ),
                            (
                                "end".to_string(),
                                FieldValue::Literal(LiteralBits::Int("1".to_string())),
                            ),
                        ]),
                    ),
                ],
            }),
            refinement: None,
            nominal_opacity: None,
            span: SourceSpan::new("gate95_injected.dag", 0, 1),
        });
    }

    #[test]
    fn opt_in_budget_is_clean_when_lane2_reads_parallelizable_indicator() {
        let mut dag = compile_to_dag(
            "// gunbc::r3_free_consequences::lane2_loop_witness: read_only\n\
             import lenses.parallelism { parallelism_enforceable }\n\
             fn gate95_demo_fn() -> Int = 0\n",
            "gate95_iteration_parallelism_clean.v3",
        )
        .expect("compile");
        let subject = dag.workflow_lane2_subject().expect("workflow shell bind");
        let fn_decl = dag
            .declaration_by_name("gate95_demo_fn")
            .expect("fn decl")
            .id;
        let indicator = crate::loop_iteration_parallel_emission_indicator(&dag, subject);
        assert_eq!(indicator, 1);
        assert!(!parallelism_iteration_opt_in_enforcement_violates(&dag, subject).unwrap());

        push_parallelism_iteration_enforced_declaration(
            &mut dag,
            "gate95_iteration_parallelism_clean_witness",
            fn_decl,
            subject,
        );
        check_enforced_lens_applications(&mut dag);
        assert!(
            dag.diagnostics().is_empty(),
            "unexpected diagnostics: {:?}",
            dag.diagnostics()
        );
    }

    #[test]
    fn opt_in_budget_violates_when_lane2_reads_sequential_indicator() {
        let mut dag = compile_to_dag(
            "// gunbc::r3_free_consequences::lane2_loop_witness: upsert_dependent\n\
             import lenses.parallelism { parallelism_enforceable }\n\
             fn gate95_demo_fn() -> Int = 0\n",
            "gate95_iteration_parallelism_violation.v3",
        )
        .expect("compile");
        let subject = dag.workflow_lane2_subject().expect("workflow shell bind");
        let fn_decl = dag
            .declaration_by_name("gate95_demo_fn")
            .expect("fn decl")
            .id;
        let indicator = crate::loop_iteration_parallel_emission_indicator(&dag, subject);
        assert_eq!(indicator, 0);
        assert!(parallelism_iteration_opt_in_enforcement_violates(&dag, subject).unwrap());

        push_parallelism_iteration_enforced_declaration(
            &mut dag,
            "gate95_iteration_parallelism_violation_witness",
            fn_decl,
            subject,
        );
        check_enforced_lens_applications(&mut dag);
        assert_eq!(
            dag.diagnostics().len(),
            1,
            "expected iteration-opt-in parallelism violation diagnostic; got {:?}",
            dag.diagnostics()
        );
        let (_, diagnostic) = dag.diagnostics().iter().next().expect("diagnostic");
        let Diagnostic::ParseError { message, .. } = diagnostic else {
            panic!("expected ParseError violation, got {diagnostic:?}");
        };
        assert!(
            message.contains("loop_iteration_parallel_emission_indicator=0"),
            "message must name the sequential observation; got {message:?}"
        );
    }

    #[test]
    fn opt_in_rejects_out_of_range_node_scope_subject() {
        let mut dag = compile_to_dag(
            "// gunbc::r3_free_consequences::lane2_loop_witness: read_only\n\
             import lenses.parallelism { parallelism_enforceable }\n\
             fn gate95_demo_fn() -> Int = 0\n",
            "gate95_oob_node_idx.v3",
        )
        .expect("compile");
        let fn_decl = dag
            .declaration_by_name("gate95_demo_fn")
            .expect("fn decl")
            .id;
        let bogus_subject = NodeId::from_table_index(dag.nodes().len() as u32);
        assert!(
            dag.node_opt(&bogus_subject).is_none(),
            "fixture must use one-past-last node index as nonexistent NodeId witness"
        );
        push_parallelism_iteration_enforced_declaration(
            &mut dag,
            "gate95_oob_witness",
            fn_decl,
            bogus_subject,
        );
        check_enforced_lens_applications(&mut dag);
        assert_eq!(dag.diagnostics().len(), 1, "{:?}", dag.diagnostics());
        let (_, diagnostic) = dag.diagnostics().iter().next().expect("diagnostic");
        let Diagnostic::ParseError { message, .. } = diagnostic else {
            panic!("expected ParseError coupling failure; got {diagnostic:?}");
        };
        assert!(
            message.contains("fail closed") && message.contains("NodeScope"),
            "unexpected message: {message:?}"
        );
    }

    #[test]
    fn opt_in_rejects_node_scope_foreign_to_declaration() {
        let mut dag = compile_to_dag(
            "// gunbc::r3_free_consequences::lane2_loop_witness: read_only\n\
             import lenses.parallelism { parallelism_enforceable }\n\
             fn gate95_early() -> Int = 0\n\
             fn gate95_last() -> Int = 1\n",
            "gate95_foreign_subject.v3",
        )
        .expect("compile");
        let fn_early = dag.declaration_by_name("gate95_early").expect("early").id;
        let alien_subject = dag.workflow_lane2_subject().expect("last fn bind shell");
        assert!(
            !crate::dag::node_scope_subject_within_arrow_declaration(&dag, fn_early, alien_subject),
            "fixture ordering invariant: alien subject must be outside gate95_early's lowered body \
             (workflow shell must not alias the early Arrow root)"
        );
        push_parallelism_iteration_enforced_declaration(
            &mut dag,
            "gate95_foreign_subject_witness",
            fn_early,
            alien_subject,
        );
        check_enforced_lens_applications(&mut dag);
        assert_eq!(dag.diagnostics().len(), 1, "{:?}", dag.diagnostics());
        let (_, diagnostic) = dag.diagnostics().iter().next().expect("diagnostic");
        let Diagnostic::ParseError { message, .. } = diagnostic else {
            panic!("expected boundary ParseError for decoupled subject; got {diagnostic:?}");
        };
        assert!(
            message.contains("lowered graph")
                && message.contains("loop_iteration_parallel_emission_indicator"),
            "unexpected coupling message: {message:?}"
        );
    }
}
