//! The required-CI measurement receipt and the one relation that decides whether a measured
//! lane is admitted.
//!
//! HAND-RUST GATE explicit deferral. Lane: required-ci-measurement-host-realization. The
//! authority is `v2.workflow.required_ci_measurement`; this seed code realizes its filesystem
//! write, JSON transport and readback because required CI runs the bootstrapped
//! `claim_executor` before a generated replacement owns those effects. The coproduct, the
//! blocker fields and the admission relation originate in `.dag`
//! (`required_ci_measurement_admitted`); this module is one realization of them and adds no
//! competing model. It dissolves at the ROADMAP row `v1-zero-hand-maintained-rust`.
//!
//! WHY THIS LIVES IN THE LIBRARY AND NOT BESIDE ITS ONE CALLER. The relation it carries is the
//! one a required lane's PROCESS EXIT is derived from, and that derivation was wrong for at
//! least two landed runs (`gunbc.recurring_failure_mode`
//! `gate_reported_success_on_a_phase_it_refused`). A relation whose defect is invisible needs a
//! discriminating control that EXECUTES it, and the only Rust the repository documents a local
//! execution route for is the library: `cargo test --release -p v1-compiler --lib`
//! (`gunbc.repo_self_build` `repo_self_test_command`). A `#[cfg(test)]` module beside the
//! binary would compile on the merge path and run on no route at all.

use serde::{Deserialize, Serialize};

pub const REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION: u8 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RequiredCiBlocker {
    pub phase: String,
    pub identity: String,
    pub cause: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "standing", rename_all = "snake_case")]
pub enum RequiredCiMeasurementReceipt {
    MeasurementCompleted { blockers: Vec<RequiredCiBlocker> },
    MeasurementUnreached { cause: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VersionedRequiredCiMeasurementReceipt {
    pub version: u8,
    pub receipt: RequiredCiMeasurementReceipt,
}

/// The verdict a required lane's exit status is derived from.
///
/// It is a coproduct rather than a `bool` so that the refusing arm carries the located
/// diagnostics its caller must print: a refusal that cannot say which phase and which identity
/// declined is a degradation that is neither typed nor located.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequiredCiAdmission {
    Admitted { summary: String },
    Refused { diagnostics: Vec<String> },
}

impl RequiredCiAdmission {
    pub fn is_admitted(&self) -> bool {
        matches!(self, RequiredCiAdmission::Admitted { .. })
    }

    /// The lines the caller prints, in order, whichever arm was reached.
    pub fn diagnostics(&self) -> Vec<String> {
        match self {
            RequiredCiAdmission::Admitted { summary } => vec![summary.clone()],
            RequiredCiAdmission::Refused { diagnostics } => diagnostics.clone(),
        }
    }
}

pub fn completed_required_ci_measurement_receipt(
    blockers: Vec<RequiredCiBlocker>,
) -> RequiredCiMeasurementReceipt {
    RequiredCiMeasurementReceipt::MeasurementCompleted { blockers }
}

pub fn write_required_ci_measurement_receipt(
    path: &str,
    receipt: RequiredCiMeasurementReceipt,
) -> Result<(), String> {
    let encoded = serde_json::to_vec_pretty(&VersionedRequiredCiMeasurementReceipt {
        version: REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION,
        receipt,
    })
    .map_err(|e| format!("encode required CI measurement receipt: {e}"))?;
    std::fs::write(path, encoded)
        .map_err(|e| format!("write required CI measurement receipt {path}: {e}"))
}

/// THE ONE ADMISSION RELATION, realizing `v2.workflow.required_ci_measurement`
/// `required_ci_measurement_admitted`: a measurement is admitted exactly when it COMPLETED and
/// carries NO blockers. Every other standing -- an unreached instrument, one blocker, many --
/// refuses.
pub fn adjudicate_required_ci_measurement(
    receipt: &RequiredCiMeasurementReceipt,
) -> RequiredCiAdmission {
    match receipt {
        RequiredCiMeasurementReceipt::MeasurementUnreached { cause } => {
            RequiredCiAdmission::Refused {
                diagnostics: vec![format!(
                    "required-ci: adjudication REFUSED standing=measurement_unreached cause={cause}"
                )],
            }
        }
        RequiredCiMeasurementReceipt::MeasurementCompleted { blockers } if blockers.is_empty() => {
            RequiredCiAdmission::Admitted {
                summary: "required-ci: adjudication PASSED standing=measurement_completed \
                          blockers=0"
                    .to_string(),
            }
        }
        RequiredCiMeasurementReceipt::MeasurementCompleted { blockers } => {
            let mut diagnostics: Vec<String> = blockers
                .iter()
                .map(|blocker| {
                    format!(
                        "required-ci: adjudication BLOCKING phase={} identity={} cause={}",
                        blocker.phase, blocker.identity, blocker.cause
                    )
                })
                .collect();
            diagnostics.push(format!(
                "required-ci: adjudication REFUSED standing=measurement_completed blockers={}",
                blockers.len()
            ));
            RequiredCiAdmission::Refused { diagnostics }
        }
    }
}

/// Adjudicate what is ON DISK at `path`, which is the form both consumers need: the separate
/// `--adjudicate-measurement-receipt` mode reads a receipt another process published, and the
/// measuring process itself reads BACK what it just wrote rather than adjudicating the value it
/// intended to write. A write that silently truncated is then a refusal, not a pass.
pub fn adjudicate_required_ci_measurement_receipt_file(path: &str) -> RequiredCiAdmission {
    let body = match std::fs::read(path) {
        Ok(body) => body,
        Err(e) => {
            return RequiredCiAdmission::Refused {
                diagnostics: vec![format!(
                    "required-ci: adjudication REFUSED receipt unreadable path={path} cause={e}"
                )],
            }
        }
    };
    let versioned: VersionedRequiredCiMeasurementReceipt = match serde_json::from_slice(&body) {
        Ok(versioned) => versioned,
        Err(e) => {
            return RequiredCiAdmission::Refused {
                diagnostics: vec![format!(
                    "required-ci: adjudication REFUSED receipt malformed path={path} cause={e}"
                )],
            }
        }
    };
    if versioned.version != REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION {
        return RequiredCiAdmission::Refused {
            diagnostics: vec![format!(
                "required-ci: adjudication REFUSED receipt version={} expected={}",
                versioned.version, REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION
            )],
        };
    }
    adjudicate_required_ci_measurement(&versioned.receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocker(phase: &str, identity: &str, cause: &str) -> RequiredCiBlocker {
        RequiredCiBlocker {
            phase: phase.to_string(),
            identity: identity.to_string(),
            cause: cause.to_string(),
        }
    }

    fn temp_receipt_path(tag: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "required_ci_measurement_{tag}_{}.json",
            std::process::id()
        ));
        path
    }

    // THE DISCRIMINATING RED, stated as the defect it was written against: run 35503853026's
    // floor job refused the changed-witness observation, counted the refusal, and the step that
    // carried it concluded SUCCESS. The refusal is correct and stays; what may never happen
    // again is a refused phase reaching an admitted verdict.
    //
    // It EXECUTES the relation rather than asserting a rendered string: delete the blocker-bearing
    // arm of `adjudicate_required_ci_measurement`, or restore an arm that returns `Admitted` for a
    // non-empty blocker set, and this goes red.
    #[test]
    fn a_refused_phase_is_never_admitted() {
        let receipt = completed_required_ci_measurement_receipt(vec![blocker(
            "floor",
            "ChangedWitnessObservationFailed",
            "the base side could not be reconstructed (dag/std/types.dag differs)",
        )]);
        let admission = adjudicate_required_ci_measurement(&receipt);
        assert!(
            !admission.is_admitted(),
            "a completed measurement carrying a blocker was admitted: {admission:?}"
        );
        assert!(
            admission
                .diagnostics()
                .iter()
                .any(|line| line.contains("phase=floor")
                    && line.contains("ChangedWitnessObservationFailed")),
            "the refusal did not locate the phase and identity that declined: {admission:?}"
        );
    }

    // THE GREEN CONTROL. Without it the red above is satisfiable by refusing everything, which
    // is the same gate lying in the other direction.
    #[test]
    fn a_clean_measurement_is_admitted() {
        let receipt = completed_required_ci_measurement_receipt(vec![]);
        assert!(adjudicate_required_ci_measurement(&receipt).is_admitted());
    }

    #[test]
    fn an_unreached_instrument_is_never_admitted() {
        let receipt = RequiredCiMeasurementReceipt::MeasurementUnreached {
            cause: "instrument did not return".to_string(),
        };
        assert!(!adjudicate_required_ci_measurement(&receipt).is_admitted());
    }

    // THE ROUTE, not only the answer: the measuring process writes a receipt and then reads that
    // FILE back. This runs the write, the JSON transport and the readback that the required lane
    // runs, so a receipt that serializes its blockers away -- or a version bump that leaves the
    // reader behind -- is refused rather than passed.
    #[test]
    fn a_written_receipt_carrying_a_blocker_refuses_on_readback() {
        let path = temp_receipt_path("blocking");
        let path_str = path.to_string_lossy().to_string();
        write_required_ci_measurement_receipt(
            &path_str,
            completed_required_ci_measurement_receipt(vec![blocker(
                "floor",
                "ChangedWitnessObservationFailed",
                "planted",
            )]),
        )
        .expect("write the receipt under test");
        let admission = adjudicate_required_ci_measurement_receipt_file(&path_str);
        let _ = std::fs::remove_file(&path);
        assert!(
            !admission.is_admitted(),
            "a published receipt carrying a blocker was admitted on readback: {admission:?}"
        );
    }

    #[test]
    fn a_written_clean_receipt_is_admitted_on_readback() {
        let path = temp_receipt_path("clean");
        let path_str = path.to_string_lossy().to_string();
        write_required_ci_measurement_receipt(
            &path_str,
            completed_required_ci_measurement_receipt(vec![]),
        )
        .expect("write the receipt under test");
        let admission = adjudicate_required_ci_measurement_receipt_file(&path_str);
        let _ = std::fs::remove_file(&path);
        assert!(admission.is_admitted(), "{admission:?}");
    }

    // A RECEIPT THAT WAS NEVER WRITTEN IS NOT A CLEAN ONE. The absorbing arm here would be to
    // treat an unreadable path as nothing-to-report.
    #[test]
    fn an_absent_receipt_file_is_refused() {
        let path = temp_receipt_path("absent");
        let _ = std::fs::remove_file(&path);
        assert!(
            !adjudicate_required_ci_measurement_receipt_file(&path.to_string_lossy()).is_admitted()
        );
    }
}
