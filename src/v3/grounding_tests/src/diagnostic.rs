//! Lane-local fail-closed outcomes for T-Ground-Tests.
//!
//! The shared `EmissionDiagnostic` substrate carrier **LANDED** at
//! `src/v3/std/diagnostics.dag` per #1216 brief + #1133 dispatch 4355793511.
//! However, this crate's variants (`StratumARowCountMismatch` with usize
//! fields, `StratumARegistryResolutionFailed { row_index, ... }`,
//! `StratumBPrerequisiteMissing`, etc.)
//! are **test-outcome-specific** — they carry test-side measurement
//! coordinates that don't naturally fit the fold/emission failure pattern
//! the substrate carrier authors. These stay lane-local pending a
//! follow-up dispatch deciding whether to (a) extend the substrate
//! carrier with test-side variants, (b) author a sibling
//! `GroundingTestsDiagnostic` substrate carrier, or (c) keep the
//! lane-local Rust mirror permanently as test-outcome scaffolding.
//! Manager-acknowledged in dispatch 4355793511.

use std::fmt;

/// Typed failure for Stratum A routing-parity checks (`t-ground-tests.md` §Test plan item 1).
///
/// **Practice 4 (`docs/modeling-discipline.md`): 🟡 YELLOW** — lane-local `Result` carrier until
/// these cases fold into shared `EmissionDiagnostic` (`t-ground-diagnostic.md` / manager #1216).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundingTestsDiagnostic {
    /// `List<MethodTemplateContract>` row count drift vs Director-locked Phase 1 shape.
    StratumARowCountMismatch {
        list_name: String,
        expected: usize,
        actual: usize,
    },
    /// `dag_method: MethodRef` did not resolve to a `MethodDeclaration` name literal.
    StratumARegistryResolutionFailed {
        list_name: String,
        row_index: usize,
        detail: String,
    },
    /// Two `generated_full_bootstrap_dag()` runs produced different Stratum-A **list** digests for
    /// the same `list_name` (list-scoped witness; not a single `dag_method` row).
    StratumALockstepListDigestMismatch { list_name: String, detail: String },
    /// Expected Substrate declaration or connective shape was absent or non-conforming.
    StratumADagProjectionFailed { step: &'static str, detail: String },
    /// Stratum B cannot run production algebra-homomorphism assertions until a named upstream
    /// prerequisite is present. This is a readiness diagnostic, not a skipped production test.
    StratumBPrerequisiteMissing {
        prerequisite: &'static str,
        detail: String,
    },
}

impl fmt::Display for GroundingTestsDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroundingTestsDiagnostic::StratumARowCountMismatch {
                list_name,
                expected,
                actual,
            } => write!(
                f,
                "Stratum A row-count mismatch for `{list_name}`: expected {expected}, got {actual}"
            ),
            GroundingTestsDiagnostic::StratumARegistryResolutionFailed {
                list_name,
                row_index,
                detail,
            } => write!(
                f,
                "Stratum A registry resolution failed for `{list_name}` row {row_index}: {detail}"
            ),
            GroundingTestsDiagnostic::StratumALockstepListDigestMismatch { list_name, detail } => {
                write!(
                    f,
                    "Stratum A lockstep list-digest mismatch for `{list_name}`: {detail}"
                )
            }
            GroundingTestsDiagnostic::StratumADagProjectionFailed { step, detail } => {
                write!(f, "Stratum A Dag projection failed at `{step}`: {detail}")
            }
            GroundingTestsDiagnostic::StratumBPrerequisiteMissing {
                prerequisite,
                detail,
            } => write!(
                f,
                "Stratum B prerequisite `{prerequisite}` is not ready: {detail}"
            ),
        }
    }
}

impl std::error::Error for GroundingTestsDiagnostic {}
