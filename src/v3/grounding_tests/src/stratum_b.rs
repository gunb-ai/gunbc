//! Stratum B readiness scaffold.
//!
//! This module is intentionally a readiness/checklist surface, not production
//! routing certification. `docs/audit/grounding-tests-stratum-b-scaffold-readiness.md`
//! concludes that full Stratum B assertions are gated on declared Coercion-Fold
//! projection facts. Integer inhabitance rows now have an executable declared
//! projection path; remaining gates stay typed and visible without hardcoded
//! example drivers.

use v3_compiler::dag::{Dag, TypeConnective};
use v3_grounding_coercion_fold::MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS;

use crate::diagnostic::GroundingTestsDiagnostic;

/// Named prerequisites for production Stratum B algebra-homomorphism assertions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StratumBPrerequisite {
    BoundDeclarationCarrier,
    ProgramBoundLowering,
    IntegerInhabitanceRows,
    AlgebraIntentFacts,
    StringFamilyRows,
    LifetimeAxisVocabulary,
    SharedBindingIdentity,
}

impl StratumBPrerequisite {
    pub fn label(self) -> &'static str {
        match self {
            StratumBPrerequisite::BoundDeclarationCarrier => "BoundDeclaration carrier",
            StratumBPrerequisite::ProgramBoundLowering => "program-bound lowering",
            StratumBPrerequisite::IntegerInhabitanceRows => "integer inhabitance rows",
            StratumBPrerequisite::AlgebraIntentFacts => "algebra intent facts",
            StratumBPrerequisite::StringFamilyRows => "string-family rows",
            StratumBPrerequisite::LifetimeAxisVocabulary => "lifetime-axis vocabulary",
            StratumBPrerequisite::SharedBindingIdentity => "shared binding identity",
        }
    }
}

/// Readiness state for a prerequisite. `PendingExternal` means the audit stack names the
/// requirement, but this crate must not invent a local row schema to test it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StratumBPrerequisiteState {
    Ready,
    Missing(GroundingTestsDiagnostic),
    PendingExternal { detail: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StratumBReadinessEntry {
    pub prerequisite: StratumBPrerequisite,
    pub state: StratumBPrerequisiteState,
}

/// Current Stratum B readiness against a `Dag`.
///
/// Only the landed `BoundDeclaration` carrier is checked structurally today. The
/// remaining entries are deliberately reported as external gates until their
/// substrate authorities land; this avoids a Rust-side shadow schema.
pub fn stratum_b_readiness(dag: &Dag) -> Vec<StratumBReadinessEntry> {
    vec![
        StratumBReadinessEntry {
            prerequisite: StratumBPrerequisite::BoundDeclarationCarrier,
            state: bound_declaration_carrier_state(dag),
        },
        StratumBReadinessEntry {
            prerequisite: StratumBPrerequisite::ProgramBoundLowering,
            state: StratumBPrerequisiteState::PendingExternal {
                detail: "Int(lo..hi), Int(any), and Int(platform) must lower into BoundDeclaration facts before Stratum B can assert fold outputs",
            },
        },
        StratumBReadinessEntry {
            prerequisite: StratumBPrerequisite::IntegerInhabitanceRows,
            state: integer_inhabitance_rows_state(dag),
        },
        StratumBReadinessEntry {
            prerequisite: StratumBPrerequisite::AlgebraIntentFacts,
            state: StratumBPrerequisiteState::PendingExternal {
                detail: "program facts must distinguish Semiring vs OrderedRing or fail closed as under-refined",
            },
        },
        StratumBReadinessEntry {
            prerequisite: StratumBPrerequisite::StringFamilyRows,
            state: StratumBPrerequisiteState::PendingExternal {
                detail: "string-family target rows must declare ownership, lifetime, growability, encoding, and realization refs",
            },
        },
        StratumBReadinessEntry {
            prerequisite: StratumBPrerequisite::LifetimeAxisVocabulary,
            state: StratumBPrerequisiteState::PendingExternal {
                detail: "canonical Ownership, LifetimeScope, Growability, and Encoding substrate sums must land before lockstep ratchets",
            },
        },
        StratumBReadinessEntry {
            prerequisite: StratumBPrerequisite::SharedBindingIdentity,
            state: StratumBPrerequisiteState::PendingExternal {
                detail: "Coercion-Fold and Lifetime-Analyzer need the same binding key before String examples can consume lifetime reports",
            },
        },
    ]
}

pub fn stratum_b_missing_prerequisites(dag: &Dag) -> Vec<GroundingTestsDiagnostic> {
    stratum_b_readiness(dag)
        .into_iter()
        .filter_map(|entry| match entry.state {
            StratumBPrerequisiteState::Ready => None,
            StratumBPrerequisiteState::Missing(diagnostic) => Some(diagnostic),
            StratumBPrerequisiteState::PendingExternal { detail } => {
                Some(GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
                    prerequisite: entry.prerequisite.label(),
                    detail: detail.to_string(),
                })
            }
        })
        .collect()
}

fn bound_declaration_carrier_state(dag: &Dag) -> StratumBPrerequisiteState {
    let Some(decl) = dag.declaration_by_name("BoundDeclaration") else {
        return StratumBPrerequisiteState::Missing(
            GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
                prerequisite: StratumBPrerequisite::BoundDeclarationCarrier.label(),
                detail: "missing substrate declaration `BoundDeclaration`".to_string(),
            },
        );
    };
    let TypeConnective::Disj { variants } = &decl.connective else {
        return StratumBPrerequisiteState::Missing(
            GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
                prerequisite: StratumBPrerequisite::BoundDeclarationCarrier.label(),
                detail: format!(
                    "`BoundDeclaration` must be a Disj sum, got {:?}",
                    decl.connective
                ),
            },
        );
    };
    let labels: Vec<&str> = variants.iter().map(|v| v.label.as_str()).collect();
    if labels == ["StaticBound", "PlatformDependent"] {
        StratumBPrerequisiteState::Ready
    } else {
        StratumBPrerequisiteState::Missing(
            GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
                prerequisite: StratumBPrerequisite::BoundDeclarationCarrier.label(),
                detail: format!(
                    "`BoundDeclaration` variants must be [StaticBound, PlatformDependent], got {labels:?}"
                ),
            },
        )
    }
}

fn integer_inhabitance_rows_state(dag: &Dag) -> StratumBPrerequisiteState {
    let Some(meta) = dag.declaration_by_name("TargetIntegerTypeInhabitance") else {
        return StratumBPrerequisiteState::Missing(
            GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
                prerequisite: StratumBPrerequisite::IntegerInhabitanceRows.label(),
                detail: "missing substrate declaration `TargetIntegerTypeInhabitance`".to_string(),
            },
        );
    };
    let row_count = dag
        .declarations()
        .iter()
        .filter(|decl| decl.meta_tag == Some(meta.id))
        .count();
    if row_count >= MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS {
        StratumBPrerequisiteState::Ready
    } else {
        StratumBPrerequisiteState::Missing(GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
            prerequisite: StratumBPrerequisite::IntegerInhabitanceRows.label(),
            detail: format!(
                "expected at least {MIN_TARGET_INTEGER_TYPE_INHABITANCE_ROWS} `TargetIntegerTypeInhabitance` rows, got {row_count}"
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use v3_compiler::dag::{Interval, IntervalWidth, PositiveIntervalWidth};
    use v3_compiler::generated_full_bootstrap_dag;
    use v3_grounding_coercion_fold::{
        fold_program_to_target, IntegerBoundProjection, IntegerTargetIntent,
        LanguageSpecProjection, SelectedTargetInhabitance,
    };
    use v3_grounding_lifetime::{BindingId, LifetimeAnalysisReport};

    use super::*;

    /// `generated_full_bootstrap_dag` construction can overflow the default test thread stack.
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

    fn declaration_id_by_name(
        dag: &v3_compiler::dag::Dag,
        name: &str,
    ) -> v3_compiler::dag::DeclarationId {
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("missing declaration `{name}`"))
            .id
    }

    #[test]
    fn bound_declaration_carrier_is_ready_in_full_bootstrap() {
        with_bootstrap_stack(|| {
            let dag = generated_full_bootstrap_dag();
            let readiness = stratum_b_readiness(&dag);
            assert_eq!(
                readiness
                    .iter()
                    .find(|entry| {
                        entry.prerequisite == StratumBPrerequisite::BoundDeclarationCarrier
                    })
                    .map(|entry| &entry.state),
                Some(&StratumBPrerequisiteState::Ready)
            );
        });
    }

    #[test]
    fn integer_inhabitance_rows_are_ready_in_full_bootstrap() {
        with_bootstrap_stack(|| {
            let dag = generated_full_bootstrap_dag();
            let readiness = stratum_b_readiness(&dag);
            assert_eq!(
                readiness
                    .iter()
                    .find(|entry| {
                        entry.prerequisite == StratumBPrerequisite::IntegerInhabitanceRows
                    })
                    .map(|entry| &entry.state),
                Some(&StratumBPrerequisiteState::Ready)
            );
        });
    }

    #[test]
    fn declared_integer_projection_selects_bootstrap_inhabitance_row() {
        with_bootstrap_stack(|| {
            let dag = generated_full_bootstrap_dag();
            let lifetime: LifetimeAnalysisReport = Default::default();
            let projection = LanguageSpecProjection::DeclaredIntegerIntents(BTreeMap::from([(
                BindingId(11),
                IntegerTargetIntent {
                    target_language: dag.rust_language_spec().expect("Rust LanguageSpec"),
                    kernel_integer: declaration_id_by_name(&dag, "UInt32"),
                    algebra: declaration_id_by_name(&dag, "UInt32"),
                    bound: IntegerBoundProjection::Static(Interval::BoundedInterval {
                        lower: 0,
                        width: IntervalWidth::PositiveWidth(PositiveIntervalWidth::UnitCount {
                            units: 4_294_967_295,
                        }),
                    }),
                },
            )]));

            let got = fold_program_to_target(&dag, &lifetime, &projection).expect("projection");
            assert_eq!(
                got.get(&BindingId(11)),
                Some(&SelectedTargetInhabitance {
                    type_realization: declaration_id_by_name(&dag, "rust_u32"),
                })
            );
        });
    }

    #[test]
    fn current_readiness_reports_remaining_external_gates_without_scratch_examples() {
        with_bootstrap_stack(|| {
            let dag = generated_full_bootstrap_dag();
            let missing = stratum_b_missing_prerequisites(&dag);
            assert_eq!(
                missing.len(),
                5,
                "BoundDeclaration and integer inhabitance rows should be ready today"
            );
            assert!(missing.iter().all(|diagnostic| matches!(
                diagnostic,
                GroundingTestsDiagnostic::StratumBPrerequisiteMissing { .. }
            )));
            assert!(missing
                .iter()
                .any(|diagnostic| diagnostic.to_string().contains("program-bound lowering")));
        });
    }

    #[test]
    fn stratum_b_prerequisite_labels_are_stable() {
        assert_eq!(
            StratumBPrerequisite::BoundDeclarationCarrier.label(),
            "BoundDeclaration carrier"
        );
        assert_eq!(
            StratumBPrerequisite::SharedBindingIdentity.label(),
            "shared binding identity"
        );
    }
}
