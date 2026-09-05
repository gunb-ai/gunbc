//! THE RETAINED CONSUMER OF `gunbc.fabric_gate_coverage`, AND NOTHING MORE THAN A HOST BOUNDARY.
//!
//! The adjudication lives entirely in `.dag`: which gate obligations exist, which witness
//! discharges each, whether the ledger is bound to the revision under adjudication, and what a
//! refusal says. This file reads one file, hands its bytes and one commit id to that module, and
//! maps the answer to an exit code. It decides nothing.
//!
//! WHY THE READ IS HERE AND NOT IN THE MODULE. `v2.extdeps.file_system` models a filesystem
//! interface, and no realization handler routes it in this seed, so a `.dag` caller cannot read a
//! path today. Host IO is the seed's remaining job — the same boundary
//! `terminal_ledger_publish` sits on, in the other direction: that module renders bytes in `.dag`
//! and writes them here; this one reads bytes here and adjudicates them in `.dag`.
//!
//! WHY IT IS A SECOND PROCESS RATHER THAN A PHASE OF THE FLOOR. The floor's measurement step
//! carries `continue-on-error: true`, deliberately: it is an instrument, and its own red must not
//! decide the job. A verdict raised inside it would inherit that, so the wall would be a
//! decoration. Running against the PUBLISHED artifact from a step that does not
//! continue-on-error is what gives the refusal a consequential edge, and it costs one parse plus
//! one fold over the enrolment roster — no witness runs here and no subject is prepared.
#![allow(clippy::uninlined_format_args)]

use crate::v1_interpreter::{self, str_value, ExecutionMode, InterpContext, Value};

const FABRIC_GATE_COVERAGE_ENTRY_UNDER_ROOT: &str = "gunbc/fabric/fabric_gate_coverage.dag";

/// The artifact this consumer adjudicates, named by the producer that writes it.
pub use super::terminal_ledger_publish::TERMINAL_LEDGER_PATH;

fn coverage_entry(source_roots: &[String]) -> Result<String, String> {
    source_roots
        .iter()
        .map(|root| std::path::Path::new(root).join(FABRIC_GATE_COVERAGE_ENTRY_UNDER_ROOT))
        .find(|candidate| candidate.is_file())
        .map(|found| found.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "FABRIC-STANDING REFUSAL cause=CoverageAuthorityAbsent \
                 entry={FABRIC_GATE_COVERAGE_ENTRY_UNDER_ROOT} roots={roots:?} — the gate coverage \
                 authority is not under any declared source root, so no standing can be derived.",
                roots = source_roots
            )
        })
}

fn build_ctx(source_roots: &[String]) -> Result<InterpContext, String> {
    let entry = coverage_entry(source_roots)?;
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index_for_discovery_corpus(&index, &entry)
        .map_err(|e| {
            format!(
                "FABRIC-STANDING REFUSAL cause=CoverageAuthorityUnresolved entry={entry} \
                 detail={e} — the coverage authority did not resolve, so no standing is derived \
                 and the run is not green."
            )
        })?;
    Ok(super::make_eval_context(
        &graph,
        indices,
        ExecutionMode::Hermetic,
    ))
}

pub struct StandingReport {
    pub holds: bool,
    pub text: String,
}

/// EVERY UNREADABLE SHAPE IS A REFUSAL. A report the seed cannot read means the two sides disagree
/// about the result type, and defaulting `holds` in either direction would be the fabricated
/// answer this whole join exists to refuse — `true` would green an unadjudicated run, `false`
/// would red one for a reason that is not about the repository.
fn read_report(ctx: &InterpContext, value: &Value) -> Result<StandingReport, String> {
    let Value::Record { fields, .. } = value else {
        return Err(format!(
            "FABRIC-STANDING REFUSAL cause=ReportShapeUnexpected got={} — expected a \
             FabricStandingReport.",
            value.type_label_public()
        ));
    };
    let mut holds: Option<bool> = None;
    let mut text: Option<String> = None;
    for (sym, field) in fields.iter() {
        if ctx.sym_eq(*sym, "holds") {
            if let Value::Bool(b) = field {
                holds = Some(*b);
            }
        } else if ctx.sym_eq(*sym, "text") {
            if let Value::Str(s) = field {
                text = Some(s.to_string());
            }
        }
    }
    match (holds, text) {
        (Some(holds), Some(text)) => Ok(StandingReport { holds, text }),
        _ => Err(
            "FABRIC-STANDING REFUSAL cause=ReportFieldsUnreadable — FabricStandingReport did not \
             carry a readable `holds` and `text`."
                .to_string(),
        ),
    }
}

/// Read the published ledger and adjudicate it against the commit this tree is standing at.
///
/// THE COMMIT IS THE CALLER'S AND IS READ BY A DIFFERENT INSTRUMENT THAN THE LEDGER'S HEADER. The
/// header carries what the floor was told it was running at; the argument carries what the
/// checked-out tree says it is. Reading both from one source would make the comparison agree with
/// itself, and a ledger left behind by an earlier run would then adjudicate as this run's evidence.
pub fn adjudicate_published_ledger(
    source_roots: &[String],
    ledger_path: &str,
    adjudicated_commit: &str,
) -> Result<StandingReport, String> {
    let ledger_text = std::fs::read_to_string(ledger_path).map_err(|e| {
        format!(
            "FABRIC-STANDING REFUSAL cause=LedgerAbsent path={ledger_path} detail={e} — the \
             required floor publishes this artifact on every run, so its absence is a missing \
             producer rather than a missing obligation, and no standing may be derived without it."
        )
    })?;
    let ctx = build_ctx(source_roots)?;
    let args = vec![
        (Some("ledger_text".to_string()), str_value(ledger_text)),
        (
            Some("adjudicated_commit_hex".to_string()),
            str_value(adjudicated_commit),
        ),
    ];
    let value = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "adjudicate_published_ledger", &args, false)
    })
    .map_err(|e| {
        format!(
            "FABRIC-STANDING REFUSAL cause=AdjudicationFailed detail={e} — the coverage authority \
             did not produce a report, so this run has no standing verdict."
        )
    })?;
    read_report(&ctx, &value)
}

/// THE REVISION THE TREE IS STANDING AT, READ FROM GIT RATHER THAN FROM THE ENVIRONMENT.
///
/// WHAT THIS COMPARISON IS, after two wrong descriptions of it. It does NOT compare
/// `git rev-parse HEAD` against `GITHUB_SHA`. It compares a CURRENT revision, from whichever
/// source supplies it, against the revision the LEDGER carries in its header. Those are different
/// operands, and keeping that straight is what settles the question.
///
/// The first description said rev-parse is "a different instrument" from `GITHUB_SHA` and can
/// therefore disagree. Within one correctly checked-out run it cannot: `actions/checkout` puts the
/// worktree at exactly `GITHUB_SHA` on a push and at the merge commit whose SHA is `GITHUB_SHA` on
/// a pull_request. So rev-parse is NOT uniquely capable here, and independence between the two
/// sources is not the reason to prefer it.
///
/// The second description then overcorrected, calling the comparison a decoration on that basis.
/// It is not. Agreement between two sources of the CURRENT revision says nothing about whether
/// either agrees with a SUPPLIED ARTIFACT: a ledger whose header names another revision — left by
/// an earlier run, or substituted — is rejected by both equally. The red is authorable, and the
/// only genuinely vacuous construction would be taking the expected value out of the ledger and
/// comparing the ledger to itself.
///
/// WHAT NEITHER SOURCE CATCHES, which is why this is not R2's answer: an earlier invocation at B,
/// a current invocation also at B, and a stale ledger still naming B. Revision equality holds and
/// the artifact is still the wrong one. Distinguishing that needs an INVOCATION binding, not a
/// revision comparison, and it stays open.
///
/// So the value is read from the tree because it must come from outside the artifact under
/// adjudication, and the worktree is the referent available at that moment — not because it is a
/// second opinion about the same run.
///
/// Every failure is a refusal: a tree whose revision cannot be read cannot adjudicate a ledger
/// bound to one.
pub fn head_commit_of_worktree() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| {
            format!(
                "FABRIC-STANDING REFUSAL cause=RevisionUnreadable detail={e} — `git rev-parse \
                 HEAD` did not run, so the revision this ledger would be adjudicated against is \
                 unknown."
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "FABRIC-STANDING REFUSAL cause=RevisionUnreadable status={:?} stderr={} — the \
             worktree did not name a HEAD commit.",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if commit.is_empty() {
        return Err(
            "FABRIC-STANDING REFUSAL cause=RevisionUnreadable detail=empty — `git \
                    rev-parse HEAD` returned no revision."
                .to_string(),
        );
    }
    Ok(commit)
}
