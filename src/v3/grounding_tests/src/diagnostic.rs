//! Lane-local fail-closed outcomes for T-Ground-Tests (Stratum A scaffold).
//!
//! Converges toward the shared `EmissionDiagnostic` substrate carrier per
//! `docs/briefs/t-ground-diagnostic.md` / manager #1216 — this crate keeps a
//! small structural sum until that hand-off lands.

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
        }
    }
}

impl std::error::Error for GroundingTestsDiagnostic {}
