//! Structural fold entry — coercion = emission (incremental body).
//!
//! ## Design authority (`docs/design-emission-model.md`)
//!
//! Worked examples in that doc are behavioral targets. **Examples 1, 2, 5, 6, and 8** are
//! implemented for the [`LanguageSpecProjection::ScratchIntExamples`](crate::types::LanguageSpecProjection::ScratchIntExamples)
//! checkpoint path only; other examples and `Undeclared` remain
//! [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).
//!
//! **Call-site (`ScratchIntExamples`):** counts declared `TargetIntegerTypeInhabitance` rows in `dag`
//! (**INVARIANTS.md E-6** witness) before applying scratch outcomes. Examples 2 and 8 now
//! select by full declared integer-row facts; the scratch driver still fixes
//! [`BindingId`](v3_grounding_lifetime::BindingId)`(0)` only because real program-bound
//! and algebra-intent extraction is still Slice C scope (#1133 / #1286).

use std::collections::BTreeMap;

use v3_compiler::dag::{
    Dag, Declaration, DeclarationId, FieldValue, Interval, IntervalWidth, LiteralBits,
    PositiveIntervalWidth, TypeConnective, ValueBody,
};
use v3_grounding_lifetime::{BindingId, LifetimeAnalysisReport};

use crate::diagnostic::EmissionDiagnostic;
use crate::types::{IntScratchExample, LanguageSpecProjection, TargetInhabitance};

/// Same-PR consumer for `TargetIntegerTypeInhabitance` spec rows (`emit_model.dag`, **E-6**).
///
/// Counts declarations meta-tagged with the template. Coercion-Fold requires this count
/// before scratch examples run so deleting or failing to lower inhabitance `data` breaks CI.
const MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundMatch {
    Matches,
    DiffersExact,
    DiffersKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoundDeclarationView {
    StaticBound(Interval<i64>),
    #[allow(dead_code)]
    PlatformDependent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TargetIntegerInhabitanceBoundView {
    BoundUnspecified,
    StaticBoundFact(Interval<i64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScratchTargetLanguage {
    Rust,
    Python,
    Go,
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
    type_realization: DeclarationId,
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

fn design_doc_example_8_program_bound() -> BoundDeclarationView {
    BoundDeclarationView::StaticBound(Interval::BoundedInterval {
        lower: -2_147_483_648,
        width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
            units: 4_294_967_295,
        }),
    })
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
        _ => None,
    }
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

fn target_language_id(dag: &Dag, target: ScratchTargetLanguage) -> Option<DeclarationId> {
    match target {
        ScratchTargetLanguage::Rust => dag.rust_language_spec(),
        ScratchTargetLanguage::Python => dag.python_language_spec(),
        ScratchTargetLanguage::Go => dag.go_language_spec(),
    }
}

fn select_example_8_declared_inhabitance(
    dag: &Dag,
    target: ScratchTargetLanguage,
    program_bound: &BoundDeclarationView,
) -> Result<TargetInhabitance, EmissionDiagnostic> {
    let intent = example_8_program_intent(dag, target, program_bound.clone())?;
    select_declared_inhabitance(dag, &intent)
}

fn select_declared_inhabitance(
    dag: &Dag,
    intent: &ProgramIntegerIntent,
) -> Result<TargetInhabitance, EmissionDiagnostic> {
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
        .filter(|row| row.type_realization == intent.type_realization)
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

    target_inhabitance_from_type_realization(dag, selected.type_realization).ok_or_else(|| {
        EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_integer_type_realization".to_string(),
        }
    })
}

fn target_inhabitance_from_type_realization(
    dag: &Dag,
    realization: DeclarationId,
) -> Option<TargetInhabitance> {
    match dag.declaration(realization).name.as_deref()? {
        "rust_i32" => Some(TargetInhabitance::RustI32),
        "python_int" => Some(TargetInhabitance::PythonInt),
        "go_int32" => Some(TargetInhabitance::GoInt32),
        "rust_u32" => Some(TargetInhabitance::RustU32),
        _ => None,
    }
}

fn declaration_id_by_name(
    dag: &Dag,
    name: &str,
    axis: &str,
) -> Result<DeclarationId, EmissionDiagnostic> {
    dag.declaration_by_name(name)
        .map(|decl| decl.id)
        .ok_or_else(|| EmissionDiagnostic::UnderRefined {
            unspecified_axis: axis.to_string(),
        })
}

fn example_8_program_intent(
    dag: &Dag,
    target: ScratchTargetLanguage,
    bound: BoundDeclarationView,
) -> Result<ProgramIntegerIntent, EmissionDiagnostic> {
    let target_language =
        target_language_id(dag, target).ok_or_else(|| EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_language".to_string(),
        })?;
    let (kernel_integer_name, algebra_name, type_realization_name) = match target {
        ScratchTargetLanguage::Rust => ("Int32", "Int32", "rust_i32"),
        ScratchTargetLanguage::Python => ("Int", "Int", "python_int"),
        ScratchTargetLanguage::Go => ("Int32", "Int32", "go_int32"),
    };
    Ok(ProgramIntegerIntent {
        target_language,
        kernel_integer: declaration_id_by_name(dag, kernel_integer_name, "kernel_integer")?,
        algebra: declaration_id_by_name(dag, algebra_name, "algebra")?,
        bound,
        type_realization: declaration_id_by_name(
            dag,
            type_realization_name,
            "target_integer_type_realization",
        )?,
    })
}

fn fold_design_doc_example_1_unrefined_int() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Err(EmissionDiagnostic::UnderRefined {
        unspecified_axis: "bound".to_string(),
    })
}

fn design_doc_example_2_program_bound() -> BoundDeclarationView {
    BoundDeclarationView::StaticBound(Interval::BoundedInterval {
        lower: 0,
        width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
            units: 4_294_967_295,
        }),
    })
}

fn fold_design_doc_example_2_semiring_u32(
    dag: &Dag,
) -> Result<TargetInhabitance, EmissionDiagnostic> {
    let target_language =
        dag.rust_language_spec()
            .ok_or_else(|| EmissionDiagnostic::UnderRefined {
                unspecified_axis: "target_language".to_string(),
            })?;
    let intent = ProgramIntegerIntent {
        target_language,
        kernel_integer: declaration_id_by_name(dag, "UInt32", "kernel_integer")?,
        algebra: declaration_id_by_name(dag, "UInt32", "algebra")?,
        bound: design_doc_example_2_program_bound(),
        type_realization: declaration_id_by_name(
            dag,
            "rust_u32",
            "target_integer_type_realization",
        )?,
    };
    select_declared_inhabitance(dag, &intent)
}

fn fold_design_doc_example_5_ambiguous_algebra() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Err(EmissionDiagnostic::UnderRefined {
        unspecified_axis: "algebra".to_string(),
    })
}

fn fold_design_doc_example_6_no_inhabitant() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Err(EmissionDiagnostic::NoInhabitant)
}

fn fold_design_doc_example_8_rust(dag: &Dag) -> Result<TargetInhabitance, EmissionDiagnostic> {
    select_example_8_declared_inhabitance(
        dag,
        ScratchTargetLanguage::Rust,
        &design_doc_example_8_program_bound(),
    )
}

fn fold_design_doc_example_8_python(dag: &Dag) -> Result<TargetInhabitance, EmissionDiagnostic> {
    select_example_8_declared_inhabitance(
        dag,
        ScratchTargetLanguage::Python,
        &design_doc_example_8_program_bound(),
    )
}

fn fold_design_doc_example_8_go(dag: &Dag) -> Result<TargetInhabitance, EmissionDiagnostic> {
    select_example_8_declared_inhabitance(
        dag,
        ScratchTargetLanguage::Go,
        &design_doc_example_8_program_bound(),
    )
}

#[cfg(test)]
pub(crate) fn fold_design_doc_example_8_for_testing(
    dag: &Dag,
    target: IntScratchExample,
    program_bound: Interval<i64>,
) -> Result<TargetInhabitance, EmissionDiagnostic> {
    let target = match target {
        IntScratchExample::DesignDocExample8Rust => ScratchTargetLanguage::Rust,
        IntScratchExample::DesignDocExample8Python => ScratchTargetLanguage::Python,
        IntScratchExample::DesignDocExample8Go => ScratchTargetLanguage::Go,
        _ => {
            return Err(EmissionDiagnostic::UnderRefined {
                unspecified_axis: "example_8_target".to_string(),
            });
        }
    };
    select_example_8_declared_inhabitance(
        dag,
        target,
        &BoundDeclarationView::StaticBound(program_bound),
    )
}

#[cfg(test)]
pub(crate) fn select_program_integer_intent_for_testing(
    dag: &Dag,
    target: IntScratchExample,
    kernel_integer_name: &str,
    algebra_name: &str,
    type_realization_name: &str,
    program_bound: Interval<i64>,
) -> Result<TargetInhabitance, EmissionDiagnostic> {
    let target = match target {
        IntScratchExample::DesignDocExample8Rust => ScratchTargetLanguage::Rust,
        IntScratchExample::DesignDocExample8Python => ScratchTargetLanguage::Python,
        IntScratchExample::DesignDocExample8Go => ScratchTargetLanguage::Go,
        IntScratchExample::DesignDocExample2BoundedU32 => ScratchTargetLanguage::Rust,
        _ => {
            return Err(EmissionDiagnostic::UnderRefined {
                unspecified_axis: "integer_intent_target".to_string(),
            });
        }
    };
    let target_language =
        target_language_id(dag, target).ok_or_else(|| EmissionDiagnostic::UnderRefined {
            unspecified_axis: "target_language".to_string(),
        })?;
    let intent = ProgramIntegerIntent {
        target_language,
        kernel_integer: declaration_id_by_name(dag, kernel_integer_name, "kernel_integer")?,
        algebra: declaration_id_by_name(dag, algebra_name, "algebra")?,
        bound: BoundDeclarationView::StaticBound(program_bound),
        type_realization: declaration_id_by_name(
            dag,
            type_realization_name,
            "target_integer_type_realization",
        )?,
    };
    select_declared_inhabitance(dag, &intent)
}

/// Structural fold: program + lifetime analysis + LanguageSpec projection →
/// per-binding target inhabitances, **or** a single typed diagnostic.
///
/// - [`LanguageSpecProjection::Undeclared`](crate::types::LanguageSpecProjection::Undeclared): fail-closed
///   [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).
/// - [`LanguageSpecProjection::ScratchIntExamples`](crate::types::LanguageSpecProjection::ScratchIntExamples): runs
///   design-doc Int Examples 1, 2, 5, 6, and 8 for a single synthetic binding [`BindingId`](v3_grounding_lifetime::BindingId)`(0)`
///   after verifying the bootstrap `dag` carries at least eight `TargetIntegerTypeInhabitance` meta-tagged rows (**E-6**).
///   Examples 2 and 8 consume those rows structurally; Examples 1, 5, and 6 remain
///   blocked on program-bound/algebra-intent extraction. **Checkpoint:** on the scratch path,
///   `lifetime_facts` must be empty in debug builds. Do not widen this arm to multiple
///   bindings without landing the declared projection / dissolution path first (#1133 / #1286).
pub fn fold_program_to_target(
    dag: &Dag,
    lifetime_facts: &LifetimeAnalysisReport,
    language_spec: &LanguageSpecProjection,
) -> Result<BTreeMap<BindingId, TargetInhabitance>, EmissionDiagnostic> {
    match language_spec {
        LanguageSpecProjection::Undeclared => {
            let _ = lifetime_facts;
            Err(EmissionDiagnostic::FoldNotImplemented)
        }
        LanguageSpecProjection::ScratchIntExamples(example) => {
            let declared_rows = declared_target_integer_type_inhabitance_row_count(dag);
            if declared_rows < MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS {
                return Err(EmissionDiagnostic::UnderRefined {
                    unspecified_axis: "declared_TargetIntegerTypeInhabitance_rows".to_string(),
                });
            }
            debug_assert!(
                lifetime_facts.is_empty(),
                "ScratchIntExamples checkpoint: pass an empty LifetimeAnalysisReport until this body reads facts (#1133 / #1286)"
            );
            let binding = BindingId(0);
            let inhabitance = match example {
                IntScratchExample::DesignDocExample1UnrefinedInt => {
                    fold_design_doc_example_1_unrefined_int()?
                }
                IntScratchExample::DesignDocExample2BoundedU32 => {
                    fold_design_doc_example_2_semiring_u32(dag)?
                }
                IntScratchExample::DesignDocExample5AmbiguousAlgebra => {
                    fold_design_doc_example_5_ambiguous_algebra()?
                }
                IntScratchExample::DesignDocExample6NoInhabitant => {
                    fold_design_doc_example_6_no_inhabitant()?
                }
                IntScratchExample::DesignDocExample8Rust => fold_design_doc_example_8_rust(dag)?,
                IntScratchExample::DesignDocExample8Python => {
                    fold_design_doc_example_8_python(dag)?
                }
                IntScratchExample::DesignDocExample8Go => fold_design_doc_example_8_go(dag)?,
            };
            Ok(BTreeMap::from([(binding, inhabitance)]))
        }
    }
}
