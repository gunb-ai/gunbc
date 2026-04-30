//! Structural fold entry — coercion = emission (incremental body).
//!
//! ## Design authority (`docs/design-emission-model.md`)
//!
//! Worked **Examples 1–7** in that doc are behavioral targets. **Examples 1–2** are
//! implemented for the [`LanguageSpecProjection::ScratchIntExamples`](crate::types::LanguageSpecProjection::ScratchIntExamples)
//! checkpoint path only; other examples and `Undeclared` remain
//! [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).

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

/// Structural fold: program + lifetime analysis + LanguageSpec projection →
/// per-binding target inhabitances, **or** a single typed diagnostic.
///
/// - [`LanguageSpecProjection::Undeclared`](crate::types::LanguageSpecProjection::Undeclared): fail-closed
///   [`EmissionDiagnostic::FoldNotImplemented`](crate::diagnostic::EmissionDiagnostic::FoldNotImplemented).
/// - [`LanguageSpecProjection::ScratchIntExamples`](crate::types::LanguageSpecProjection::ScratchIntExamples): runs
///   design-doc Examples 1–2 for a single synthetic binding [`BindingId`](v3_grounding_lifetime::BindingId)`(0)`.
///   **Checkpoint:** ignores `_dag` and `_lifetime_facts` by design; do not widen this arm to
///   multiple bindings or real program facts without landing the declared projection / dissolution
///   path first (#1133 / #1286).
pub fn fold_program_to_target(
    _dag: &Dag,
    _lifetime_facts: &LifetimeAnalysisReport,
    language_spec: &LanguageSpecProjection,
) -> Result<BTreeMap<BindingId, TargetInhabitance>, EmissionDiagnostic> {
    match language_spec {
        LanguageSpecProjection::Undeclared => Err(EmissionDiagnostic::FoldNotImplemented),
        LanguageSpecProjection::ScratchIntExamples(example) => {
            let binding = BindingId(0);
            let inhabitance = match example {
                IntScratchExample::DesignDocExample1UnrefinedInt => {
                    fold_design_doc_example_1_unrefined_int()?
                }
                IntScratchExample::DesignDocExample2BoundedU32 => {
                    fold_design_doc_example_2_semiring_u32()?
                }
            };
            Ok(BTreeMap::from([(binding, inhabitance)]))
        }
    }
}
