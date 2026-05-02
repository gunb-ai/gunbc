//! Stratum B readiness scaffold.
//!
//! This module is intentionally a readiness/checklist surface, not production
//! routing certification. `docs/audit/grounding-tests-stratum-b-scaffold-readiness.md`
//! concludes that full Stratum B assertions are still gated on declared
//! Coercion-Fold projection facts. Until those land, this module keeps the gates
//! typed and visible without constructing local candidate rows or reading
//! `ScratchIntExamples` as if it were the production fold.

use v3_compiler::dag::{Dag, TypeConnective};

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
            state: StratumBPrerequisiteState::PendingExternal {
                detail: "Rust/Python/Go integer inhabitance rows must carry algebra and BoundDeclaration facts",
            },
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

#[cfg(test)]
mod tests {
    use v3_compiler::generated_full_bootstrap_dag;

    use super::*;

    #[test]
    fn bound_declaration_carrier_is_ready_in_full_bootstrap() {
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
    }

    #[test]
    fn current_readiness_reports_external_gates_without_production_assertions() {
        let dag = generated_full_bootstrap_dag();
        let missing = stratum_b_missing_prerequisites(&dag);
        assert_eq!(
            missing.len(),
            6,
            "only the BoundDeclaration carrier should be ready today"
        );
        assert!(missing.iter().all(|diagnostic| matches!(
            diagnostic,
            GroundingTestsDiagnostic::StratumBPrerequisiteMissing { .. }
        )));
        assert!(missing
            .iter()
            .any(|diagnostic| diagnostic.to_string().contains("program-bound lowering")));
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
