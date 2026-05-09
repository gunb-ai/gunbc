//! Structural fold entry — coercion = emission (incremental body).
//!
//! ## Design authority (`docs/design-emission-model.md`)
//!
//! Worked examples in that doc are behavioral targets for the executable
//! [`LanguageSpecProjection::DeclaredIntegerIntents`](crate::types::LanguageSpecProjection::DeclaredIntegerIntents)
//! path: program intent rows plus declared `TargetIntegerTypeInhabitance` facts in `dag`
//! drive selection. [`LanguageSpecProjection::Undeclared`](crate::types::LanguageSpecProjection::Undeclared)
//! stays [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).
//!
//! **`DeclaredIntegerIntents`:** before selection, the fold counts meta-tagged
//! `TargetIntegerTypeInhabitance` declarations (**INVARIANTS.md E-6**). After a unique structural
//! match, **`type_realization`** is accepted only through **`selected_target_after_row_type_realization_gate`**
//! (substrate **`TypeRealization`** meta-tag + **`language`/`target`** vs row **`language`/`kernel_integer`**);
//! mismatches surface `UnderRefined` **`target_integer_type_realization`**. Row `bound:` values use
//! [`TargetIntegerInhabitanceBound`](../../std/emit_model.dag) — including **unit
//! `PlatformDependentFact`** for platform-sized targets. [`match_bound`] pairs program
//! [`BoundDeclarationView`] with those facts: kind-only **positive match** for
//! `(PlatformDependent, PlatformDependentFact)`; **DiffersKind** when static vs platform-kind
//! disagree.

use std::collections::BTreeMap;

use v3_compiler::dag::{
    Dag, Declaration, DeclarationId, FieldValue, Interval, IntervalWidth, LiteralBits,
    PositiveIntervalWidth, TypeConnective, ValueBody,
};
use v3_grounding_lifetime::{BindingId, LifetimeAnalysisReport};

use crate::diagnostic::EmissionDiagnostic;
use crate::types::{
    IntegerBoundProjection, IntegerTargetIntent, LanguageSpecProjection, SelectedTargetInhabitance,
};

/// Same-PR consumer for `TargetIntegerTypeInhabitance` spec rows (`emit_model.dag`, **E-6**).
///
/// Counts declarations meta-tagged with the template. Coercion-Fold requires this count
/// before declared projection runs so deleting or failing to lower inhabitance `data` breaks CI.
pub const MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundMatch {
    Matches,
    DiffersExact,
    DiffersKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoundDeclarationView {
    StaticBound(Interval<i64>),
    /// No production construction site yet — program `PlatformDependent` extraction is #1133/#1286.
    /// `match_bound` + unit tests exercise this variant.
    #[allow(dead_code)]
    PlatformDependent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TargetIntegerInhabitanceBoundView {
    BoundUnspecified,
    StaticBoundFact(Interval<i64>),
    /// Target row mirrors `BoundDeclaration::PlatformDependent` (emit_model `PlatformDependentFact`).
    PlatformDependentFact,
}

struct TargetIntegerTypeInhabitanceRow {
    language: DeclarationId,
    kernel_integer: DeclarationId,
    algebra: DeclarationId,
    bound: TargetIntegerInhabitanceBoundView,
    type_realization: DeclarationId,
}

struct ProgramIntegerIntent {
    target_language: DeclarationId,
    kernel_integer: DeclarationId,
    algebra: DeclarationId,
    bound: BoundDeclarationView,
    type_realization: Option<DeclarationId>,
}

fn declared_target_integer_type_inhabitance_row_count(dag: &Dag) -> usize {
    let Some(meta) = dag.declaration_by_name("TargetIntegerTypeInhabitance") else {
        return 0;
    };
    dag.declarations()
        .iter()
        .filter(|decl| decl.meta_tag == Some(meta.id))
        .count()
}

fn match_bound(
    program: &BoundDeclarationView,
    target: &TargetIntegerInhabitanceBoundView,
) -> BoundMatch {
    match (program, target) {
        (
            BoundDeclarationView::StaticBound(_),
            TargetIntegerInhabitanceBoundView::StaticBoundFact(Interval::Unbounded),
        ) => BoundMatch::Matches,
        (
            BoundDeclarationView::StaticBound(program_interval),
            TargetIntegerInhabitanceBoundView::StaticBoundFact(target_interval),
        ) => {
            if program_interval == target_interval {
                BoundMatch::Matches
            } else {
                BoundMatch::DiffersExact
            }
        }
        (
            BoundDeclarationView::StaticBound(_),
            TargetIntegerInhabitanceBoundView::BoundUnspecified,
        ) => BoundMatch::DiffersKind,
        (
            BoundDeclarationView::StaticBound(_),
            TargetIntegerInhabitanceBoundView::PlatformDependentFact,
        ) => BoundMatch::DiffersKind,
        (
            BoundDeclarationView::PlatformDependent,
            TargetIntegerInhabitanceBoundView::PlatformDependentFact,
        ) => BoundMatch::Matches,
        (BoundDeclarationView::PlatformDependent, _) => BoundMatch::DiffersKind,
    }
}

fn exact_static_bound_match(
    program: &BoundDeclarationView,
    target: &TargetIntegerInhabitanceBoundView,
) -> bool {
    matches!(
        (program, target),
        (
            BoundDeclarationView::StaticBound(program_interval),
            TargetIntegerInhabitanceBoundView::StaticBoundFact(target_interval),
        ) if program_interval == target_interval
    )
}

fn field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find_map(|(field_label, value)| (field_label == label).then_some(value))
}

fn reference_field(fields: &[(String, FieldValue)], label: &str) -> Option<DeclarationId> {
    match field(fields, label)? {
        FieldValue::Reference(id) => Some(*id),
        _ => None,
    }
}

fn int_literal_field(fields: &[(String, FieldValue)], label: &str) -> Option<i64> {
    match field(fields, label)? {
        FieldValue::Literal(LiteralBits::Int(value)) => Some(*value),
        _ => None,
    }
}

fn variant_label(dag: &Dag, constructor: DeclarationId) -> Option<&str> {
    dag.declarations().iter().find_map(|decl| {
        if let TypeConnective::Disj { variants } = &decl.connective {
            variants
                .iter()
                .find_map(|variant| (variant.ty == constructor).then_some(variant.label.as_str()))
        } else {
            None
        }
    })
}

fn parse_positive_interval_width(dag: &Dag, value: &FieldValue) -> Option<PositiveIntervalWidth> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    match variant_label(dag, *constructor)? {
        "OneUnit" => Some(PositiveIntervalWidth::OneUnit),
        "AdditionalUnit" => {
            let [previous] = payload.as_slice() else {
                return None;
            };
            Some(PositiveIntervalWidth::AdditionalUnit {
                previous: Box::new(parse_positive_interval_width(dag, previous)?),
            })
        }
        "UnitCount" => {
            if let [FieldValue::Literal(LiteralBits::Int(units))] = payload.as_slice() {
                return Some(PositiveIntervalWidth::UnitCount { units: *units });
            }
            if let [FieldValue::Record(fields)] = payload.as_slice() {
                return Some(PositiveIntervalWidth::UnitCount {
                    units: int_literal_field(fields, "units")?,
                });
            };
            None
        }
        _ => None,
    }
}

fn parse_interval_width(dag: &Dag, value: &FieldValue) -> Option<IntervalWidth> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    match variant_label(dag, *constructor)? {
        "ZeroWidth" => {
            if payload.is_empty() {
                Some(IntervalWidth::ZeroWidth)
            } else {
                None
            }
        }
        "PositiveWidth" => {
            let [width] = payload.as_slice() else {
                return None;
            };
            Some(IntervalWidth::PositiveWidth(parse_positive_interval_width(
                dag, width,
            )?))
        }
        _ => None,
    }
}

fn parse_int_interval(dag: &Dag, value: &FieldValue) -> Option<Interval<i64>> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    match variant_label(dag, *constructor)? {
        "Unbounded" => {
            if payload.is_empty() {
                Some(Interval::Unbounded)
            } else {
                None
            }
        }
        "BoundedInterval" => {
            if let [FieldValue::Literal(LiteralBits::Int(lower)), width] = payload.as_slice() {
                return Some(Interval::BoundedInterval {
                    lower: *lower,
                    width: parse_interval_width(dag, width)?,
                });
            }
            if let [FieldValue::Record(fields)] = payload.as_slice() {
                return Some(Interval::BoundedInterval {
                    lower: int_literal_field(fields, "lower")?,
                    width: parse_interval_width(dag, field(fields, "width")?)?,
                });
            };
            None
        }
        _ => None,
    }
}

fn parse_target_integer_inhabitance_bound(
    dag: &Dag,
    value: &FieldValue,
) -> Option<TargetIntegerInhabitanceBoundView> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return None;
    };
    match variant_label(dag, *constructor)? {
        "BoundUnspecified" => {
            if payload.is_empty() {
                Some(TargetIntegerInhabitanceBoundView::BoundUnspecified)
            } else {
                None
            }
        }
        "StaticBoundFact" => {
            let [interval] = payload.as_slice() else {
                return None;
            };
            Some(TargetIntegerInhabitanceBoundView::StaticBoundFact(
                parse_int_interval(dag, interval)?,
            ))
        }
        "PlatformDependentFact" => {
            if payload.is_empty() {
                Some(TargetIntegerInhabitanceBoundView::PlatformDependentFact)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Fail-closed `TypeRealization` payload inspection + **`SelectedTargetInhabitance`** mint (**single
/// step** — proof and construction coincide). See **E-6** / inlined checks below.
///
/// **`row.type_realization`** names substrate **`data …: TypeRealization`** (**`meta_tag ==
/// type_realization_meta()`**). Structural **`language` / `target`** must equal **`row.language`**
/// / **`row.kernel_integer`** (**`emit_model.dag`** shape).
fn selected_target_after_row_type_realization_gate(
    dag: &Dag,
    row: &TargetIntegerTypeInhabitanceRow,
) -> Result<SelectedTargetInhabitance, EmissionDiagnostic> {
    let type_real_meta =
        dag.type_realization_meta()
            .ok_or_else(|| EmissionDiagnostic::UnderRefined {
                unspecified_axis: "TypeRealization_meta".to_string(),
            })?;

    let realization = row.type_realization;
    let Some(decl) = dag.declaration_opt(&realization) else {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_integer_type_realization".to_string(),
        });
    };

    if decl.meta_tag != Some(type_real_meta) {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_integer_type_realization".to_string(),
        });
    }

    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_integer_type_realization".to_string(),
        });
    };

    let Some(rez_language) = reference_field(fields, "language") else {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_integer_type_realization".to_string(),
        });
    };

    let Some(rez_target) = reference_field(fields, "target") else {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_integer_type_realization".to_string(),
        });
    };

    if rez_language != row.language || rez_target != row.kernel_integer {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_integer_type_realization".to_string(),
        });
    }

    Ok(SelectedTargetInhabitance::from_validated_type_realization(
        realization,
    ))
}

fn parse_target_integer_type_inhabitance_row(
    dag: &Dag,
    decl: &Declaration,
) -> Option<TargetIntegerTypeInhabitanceRow> {
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        return None;
    };
    Some(TargetIntegerTypeInhabitanceRow {
        language: reference_field(fields, "language")?,
        kernel_integer: reference_field(fields, "kernel_integer")?,
        algebra: reference_field(fields, "algebra")?,
        bound: parse_target_integer_inhabitance_bound(dag, field(fields, "bound")?)?,
        type_realization: reference_field(fields, "type_realization")?,
    })
}

fn select_declared_inhabitance(
    dag: &Dag,
    intent: &ProgramIntegerIntent,
) -> Result<SelectedTargetInhabitance, EmissionDiagnostic> {
    let Some(meta) = dag.declaration_by_name("TargetIntegerTypeInhabitance") else {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "declared_TargetIntegerTypeInhabitance_rows".to_string(),
        });
    };

    let matches: Vec<TargetIntegerTypeInhabitanceRow> = dag
        .declarations()
        .iter()
        .filter(|decl| decl.meta_tag == Some(meta.id))
        .filter_map(|decl| parse_target_integer_type_inhabitance_row(dag, decl))
        .filter(|row| row.language == intent.target_language)
        .filter(|row| row.kernel_integer == intent.kernel_integer)
        .filter(|row| row.algebra == intent.algebra)
        .filter(|row| {
            intent
                .type_realization
                .is_none_or(|type_realization| row.type_realization == type_realization)
        })
        .filter(|row| match_bound(&intent.bound, &row.bound) == BoundMatch::Matches)
        .collect();
    let exact_matches: Vec<&TargetIntegerTypeInhabitanceRow> = matches
        .iter()
        .filter(|row| exact_static_bound_match(&intent.bound, &row.bound))
        .collect();

    let selected = match (exact_matches.as_slice(), matches.as_slice()) {
        ([selected], _) => *selected,
        ([], [selected]) => selected,
        ([], []) => return Err(EmissionDiagnostic::NoInhabitant),
        _ => {
            return Err(EmissionDiagnostic::UnderRefined {
                unspecified_axis: "target_integer_inhabitance".to_string(),
            });
        }
    };

    selected_target_after_row_type_realization_gate(dag, selected)
}

fn program_integer_intent_from_projection(
    projection: &IntegerTargetIntent,
) -> Result<ProgramIntegerIntent, EmissionDiagnostic> {
    let bound = match &projection.bound {
        IntegerBoundProjection::Static(interval) => {
            BoundDeclarationView::StaticBound(interval.clone())
        }
        IntegerBoundProjection::PlatformDependent => BoundDeclarationView::PlatformDependent,
    };
    Ok(ProgramIntegerIntent {
        target_language: projection.target_language,
        kernel_integer: projection.kernel_integer,
        algebra: projection.algebra,
        bound,
        type_realization: None,
    })
}

fn fold_declared_integer_intents(
    dag: &Dag,
    intents: &BTreeMap<BindingId, IntegerTargetIntent>,
) -> Result<BTreeMap<BindingId, SelectedTargetInhabitance>, EmissionDiagnostic> {
    let declared_rows = declared_target_integer_type_inhabitance_row_count(dag);
    if declared_rows < MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS {
        return Err(EmissionDiagnostic::UnderRefined {
            unspecified_axis: "declared_TargetIntegerTypeInhabitance_rows".to_string(),
        });
    }

    let mut out = BTreeMap::new();
    for (&binding, projection) in intents {
        let intent = program_integer_intent_from_projection(projection)?;
        out.insert(binding, select_declared_inhabitance(dag, &intent)?);
    }
    Ok(out)
}

/// Structural fold: program + lifetime analysis + LanguageSpec projection →
/// per-binding selected target inhabitances, **or** a single typed diagnostic.
///
/// - [`LanguageSpecProjection::Undeclared`](crate::types::LanguageSpecProjection::Undeclared): fail-closed
///   [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).
/// - [`LanguageSpecProjection::DeclaredIntegerIntents`](crate::types::LanguageSpecProjection::DeclaredIntegerIntents):
///   for each binding, selects a unique `TargetIntegerTypeInhabitance` row and returns its
///   `type_realization` identity.
pub fn fold_program_to_target(
    dag: &Dag,
    lifetime_facts: &LifetimeAnalysisReport,
    language_spec: &LanguageSpecProjection,
) -> Result<BTreeMap<BindingId, SelectedTargetInhabitance>, EmissionDiagnostic> {
    match language_spec {
        LanguageSpecProjection::Undeclared => {
            let _ = lifetime_facts;
            Err(EmissionDiagnostic::FoldNotImplemented)
        }
        LanguageSpecProjection::DeclaredIntegerIntents(intents) => {
            let _ = lifetime_facts;
            fold_declared_integer_intents(dag, intents)
        }
    }
}

#[cfg(test)]
mod match_bound_tests {
    use super::*;
    use v3_compiler::dag::{Interval, IntervalWidth, PositiveIntervalWidth};

    fn i32_declared_interval() -> Interval<i64> {
        Interval::BoundedInterval {
            lower: -2_147_483_648,
            width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
                units: 4_294_967_295,
            }),
        }
    }

    #[test]
    fn platform_dependent_program_matches_platform_dependent_fact_target() {
        assert_eq!(
            match_bound(
                &BoundDeclarationView::PlatformDependent,
                &TargetIntegerInhabitanceBoundView::PlatformDependentFact,
            ),
            BoundMatch::Matches
        );
    }

    #[test]
    fn platform_dependent_program_does_not_match_static_bound_fact_target() {
        assert_eq!(
            match_bound(
                &BoundDeclarationView::PlatformDependent,
                &TargetIntegerInhabitanceBoundView::StaticBoundFact(i32_declared_interval()),
            ),
            BoundMatch::DiffersKind
        );
    }

    #[test]
    fn platform_dependent_program_does_not_match_bound_unspecified_target() {
        assert_eq!(
            match_bound(
                &BoundDeclarationView::PlatformDependent,
                &TargetIntegerInhabitanceBoundView::BoundUnspecified,
            ),
            BoundMatch::DiffersKind
        );
    }

    #[test]
    fn static_program_does_not_match_platform_dependent_fact_target() {
        assert_eq!(
            match_bound(
                &BoundDeclarationView::StaticBound(i32_declared_interval()),
                &TargetIntegerInhabitanceBoundView::PlatformDependentFact,
            ),
            BoundMatch::DiffersKind
        );
    }
}

#[cfg(test)]
mod type_realization_gate_tests {
    use super::*;
    use crate::diagnostic::EmissionDiagnostic;
    use v3_compiler::dag::Dag;

    fn with_bootstrap_stack<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("spawn bootstrap stack test thread")
            .join()
            .expect("bootstrap stack test thread panicked")
    }

    fn declaration_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("missing declaration `{name}`"))
            .id
    }

    fn u32_fixture_row_for_rust(dag: &Dag) -> TargetIntegerTypeInhabitanceRow {
        TargetIntegerTypeInhabitanceRow {
            language: dag.rust_language_spec().expect("Rust LanguageSpec"),
            kernel_integer: declaration_id_by_name(dag, "UInt32"),
            algebra: declaration_id_by_name(dag, "UInt32"),
            bound: TargetIntegerInhabitanceBoundView::BoundUnspecified,
            type_realization: declaration_id_by_name(dag, "rust_u32"),
        }
    }

    #[test]
    fn substrate_type_realization_aligns_language_and_kernel_for_fixture_row() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let row = u32_fixture_row_for_rust(&dag);
            assert!(selected_target_after_row_type_realization_gate(&dag, &row).is_ok());
        });
    }

    #[test]
    fn mismatched_language_on_type_realization_fails_gate() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let rust_u32 = declaration_id_by_name(&dag, "rust_u32");
            let kernel_u32 = declaration_id_by_name(&dag, "UInt32");
            let python_lang = dag.python_language_spec().expect("Python LanguageSpec");

            assert_eq!(
                selected_target_after_row_type_realization_gate(
                    &dag,
                    &TargetIntegerTypeInhabitanceRow {
                        language: python_lang,
                        kernel_integer: kernel_u32,
                        algebra: kernel_u32,
                        bound: TargetIntegerInhabitanceBoundView::BoundUnspecified,
                        type_realization: rust_u32,
                    },
                ),
                Err(EmissionDiagnostic::UnderRefined {
                    unspecified_axis: "target_integer_type_realization".to_string(),
                })
            );
        });
    }

    #[test]
    fn mismatched_kernel_integer_vs_realization_target_fails_gate() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let rust_language = dag.rust_language_spec().expect("Rust LanguageSpec");
            let rust_u32 = declaration_id_by_name(&dag, "rust_u32");
            let wrong_kernel = declaration_id_by_name(&dag, "Int32");

            assert_eq!(
                selected_target_after_row_type_realization_gate(
                    &dag,
                    &TargetIntegerTypeInhabitanceRow {
                        language: rust_language,
                        kernel_integer: wrong_kernel,
                        algebra: wrong_kernel,
                        bound: TargetIntegerInhabitanceBoundView::BoundUnspecified,
                        type_realization: rust_u32,
                    },
                ),
                Err(EmissionDiagnostic::UnderRefined {
                    unspecified_axis: "target_integer_type_realization".to_string(),
                })
            );
        });
    }

    #[test]
    fn kernel_declaration_used_as_payload_fails_meta_gate() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let kernel_u32 = declaration_id_by_name(&dag, "UInt32");
            let row = TargetIntegerTypeInhabitanceRow {
                language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                kernel_integer: kernel_u32,
                algebra: kernel_u32,
                bound: TargetIntegerInhabitanceBoundView::BoundUnspecified,
                type_realization: kernel_u32,
            };

            assert_eq!(
                selected_target_after_row_type_realization_gate(&dag, &row),
                Err(EmissionDiagnostic::UnderRefined {
                    unspecified_axis: "target_integer_type_realization".to_string(),
                })
            );
        });
    }

    #[test]
    fn unknown_declaration_id_used_as_payload_fails_gate() {
        with_bootstrap_stack(|| {
            let dag = Dag::new();
            let bogus = DeclarationId::declaration_id_raw_for_testing(u32::MAX);
            assert_eq!(
                selected_target_after_row_type_realization_gate(
                    &dag,
                    &TargetIntegerTypeInhabitanceRow {
                        language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                        kernel_integer: declaration_id_by_name(&dag, "UInt32"),
                        algebra: declaration_id_by_name(&dag, "UInt32"),
                        bound: TargetIntegerInhabitanceBoundView::BoundUnspecified,
                        type_realization: bogus,
                    },
                ),
                Err(EmissionDiagnostic::UnderRefined {
                    unspecified_axis: "target_integer_type_realization".to_string(),
                })
            );
        });
    }
}
