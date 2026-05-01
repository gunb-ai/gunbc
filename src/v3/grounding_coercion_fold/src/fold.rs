//! Structural fold entry — coercion = emission (incremental body).
//!
//! ## Design authority (`docs/design-emission-model.md`)
//!
//! Worked examples in that doc are behavioral targets. **Examples 1, 2, 5, 6, and 8** are
//! implemented for the [`LanguageSpecProjection::ScratchIntExamples`](crate::types::LanguageSpecProjection::ScratchIntExamples)
//! checkpoint path only; other examples and `Undeclared` remain
//! [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).
//!
//! **Call-site (ScratchIntExamples):** While the scratch body ignores `dag` / lifetime inputs and
//! fixes [`BindingId`](v3_grounding_lifetime::BindingId)`(0)`, production wiring must not treat
//! returned map keys as evidence of real program bindings (#1133 / #1286).

use std::collections::BTreeMap;

use v3_compiler::dag::Dag;
use v3_grounding_lifetime::{BindingId, LifetimeAnalysisReport};

use crate::diagnostic::EmissionDiagnostic;
use crate::types::{IntScratchExample, LanguageSpecProjection, TargetInhabitance};

fn fold_design_doc_example_1_unrefined_int() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Err(EmissionDiagnostic::UnderRefined {
        unspecified_axis: "bound".to_string(),
    })
}

fn fold_design_doc_example_2_semiring_u32() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Ok(TargetInhabitance::RustU32)
}

fn fold_design_doc_example_5_ambiguous_algebra() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Err(EmissionDiagnostic::UnderRefined {
        unspecified_axis: "algebra".to_string(),
    })
}

fn fold_design_doc_example_6_no_inhabitant() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Err(EmissionDiagnostic::NoInhabitant)
}

fn fold_design_doc_example_8_rust() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Ok(TargetInhabitance::RustI32)
}

fn fold_design_doc_example_8_python() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Ok(TargetInhabitance::PythonInt)
}

fn fold_design_doc_example_8_go() -> Result<TargetInhabitance, EmissionDiagnostic> {
    Ok(TargetInhabitance::GoInt32)
}

/// Structural fold: program + lifetime analysis + LanguageSpec projection →
/// per-binding target inhabitances, **or** a single typed diagnostic.
///
/// - [`LanguageSpecProjection::Undeclared`](crate::types::LanguageSpecProjection::Undeclared): fail-closed
///   [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).
/// - [`LanguageSpecProjection::ScratchIntExamples`](crate::types::LanguageSpecProjection::ScratchIntExamples): runs
///   design-doc Int Examples 1, 2, 5, 6, and 8 for a single synthetic binding [`BindingId`](v3_grounding_lifetime::BindingId)`(0)`.
///   **Checkpoint:** ignores `_dag` by design; on the scratch path, `lifetime_facts` must be
///   empty in debug builds until this body reads real facts. Do not widen this arm to multiple
///   bindings or real program facts without landing the declared projection / dissolution path
///   first (#1133 / #1286).
pub fn fold_program_to_target(
    _dag: &Dag,
    lifetime_facts: &LifetimeAnalysisReport,
    language_spec: &LanguageSpecProjection,
) -> Result<BTreeMap<BindingId, TargetInhabitance>, EmissionDiagnostic> {
    match language_spec {
        LanguageSpecProjection::Undeclared => {
            let _ = lifetime_facts;
            Err(EmissionDiagnostic::FoldNotImplemented)
        }
        LanguageSpecProjection::ScratchIntExamples(example) => {
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
                    fold_design_doc_example_2_semiring_u32()?
                }
                IntScratchExample::DesignDocExample5AmbiguousAlgebra => {
                    fold_design_doc_example_5_ambiguous_algebra()?
                }
                IntScratchExample::DesignDocExample6NoInhabitant => {
                    fold_design_doc_example_6_no_inhabitant()?
                }
                IntScratchExample::DesignDocExample8Rust => fold_design_doc_example_8_rust()?,
                IntScratchExample::DesignDocExample8Python => fold_design_doc_example_8_python()?,
                IntScratchExample::DesignDocExample8Go => fold_design_doc_example_8_go()?,
            };
            Ok(BTreeMap::from([(binding, inhabitance)]))
        }
    }
}
