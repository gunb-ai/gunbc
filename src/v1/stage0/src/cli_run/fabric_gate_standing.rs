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
    // PRESENCE IS READ SEPARATELY FROM SHAPE, so that a field carrying the WRONG TYPE is a
    // different located cause than a field that is ABSENT. The earlier form matched
    // `if let Value::Bool(b) = field`, which left `holds` as `None` for both, and reported one
    // cause naming neither field: a caller was told the report "did not carry a readable `holds`
    // and `text`" whether it had shipped a String for `holds`, omitted it, or both. The refusal
    // was correct -- the line stopped -- but §5 asks the diagnostic to be TYPED and LOCATED, and a
    // cause that cannot distinguish "you sent the wrong type" from "you sent nothing" tells the
    // author which line to look at and not what is wrong with it.
    let mut holds_field: Option<&Value> = None;
    let mut text_field: Option<&Value> = None;
    for (sym, field) in fields.iter() {
        if ctx.sym_eq(*sym, "holds") {
            holds_field = Some(field);
        } else if ctx.sym_eq(*sym, "text") {
            text_field = Some(field);
        }
    }
    report_from_fields(holds_field, text_field)
}

/// Decide the report from the two located fields. PURE, and separated from `read_report` for
/// exactly one reason: the field LOOKUP needs an `InterpContext` (symbol comparison) while the
/// DECISION does not, and an `InterpContext` cannot be built without a prepared corpus. Leaving
/// them fused would have made the distinction this function exists to draw -- absent versus
/// wrong-type -- reachable only through a whole compiled scope, which is why it went unwitnessed
/// long enough for a reviewer to find it by reading. The split is not scaffolding: the loop above
/// keeps the only part that genuinely needs the context.
fn report_from_fields(
    holds_field: Option<&Value>,
    text_field: Option<&Value>,
) -> Result<StandingReport, String> {
    let holds = match holds_field {
        Some(Value::Bool(b)) => Some(*b),
        _ => None,
    };
    let text = match text_field {
        Some(Value::Str(s)) => Some(s.to_string()),
        _ => None,
    };
    match (holds, text) {
        (Some(holds), Some(text)) => Ok(StandingReport { holds, text }),
        // EVERY unreadable field is reported, not just the first: an author fixing one and
        // re-running to discover the next pays a round trip per field, and the second field's
        // state is already known right here.
        (holds, text) => {
            let mut causes: Vec<String> = Vec::new();
            if holds.is_none() {
                causes.push(field_cause("holds", "Bool", holds_field));
            }
            if text.is_none() {
                causes.push(field_cause("text", "Str", text_field));
            }
            Err(format!(
                "FABRIC-STANDING REFUSAL cause=ReportFieldsUnreadable {} — FabricStandingReport \
                 did not carry a readable `holds` and `text`.",
                causes.join(" ")
            ))
        }
    }
}

/// Locate one unreadable report field: absent, or present carrying the wrong type.
///
/// The two are separate causes because they have separate remedies -- add the field, or change
/// what is written into it -- and collapsing them costs the reader exactly the distinction that
/// decides which one to make.
fn field_cause(name: &str, expected: &str, found: Option<&Value>) -> String {
    match found {
        None => format!("field={name}:absent expected={expected}"),
        Some(v) => format!(
            "field={name}:wrong-type expected={expected} got={}",
            v.type_label_public()
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

#[cfg(test)]
mod tests {
    use super::*;

    // Matched rather than `unwrap_err`, which would require `Debug` on `StandingReport` -- a
    // production derive added for a test's convenience. The test bends, not the subject.
    fn refusal(r: Result<StandingReport, String>) -> String {
        match r {
            Ok(_) => panic!("expected a refusal, got a report"),
            Err(e) => e,
        }
    }

    // THE DISCRIMINATING PAIR the reviewer's note asks for: the SAME missing `holds` reached two
    // ways must not produce the same cause. Before the split these were one string.
    #[test]
    fn a_wrong_typed_field_is_a_different_cause_than_an_absent_one() {
        let text = Value::Str("ok".into());
        let wrong = Value::Str("true".into());

        let absent = refusal(report_from_fields(None, Some(&text)));
        let mistyped = refusal(report_from_fields(Some(&wrong), Some(&text)));

        assert!(absent.contains("field=holds:absent"), "{absent}");
        assert!(mistyped.contains("field=holds:wrong-type"), "{mistyped}");
        assert_ne!(absent, mistyped);
        // Both remain refusals: the fail-closed property is what this change must NOT alter.
        assert!(absent.contains("cause=ReportFieldsUnreadable"));
        assert!(mistyped.contains("cause=ReportFieldsUnreadable"));
    }

    // Both unreadable fields are located in one refusal, so fixing them is one round trip.
    #[test]
    fn every_unreadable_field_is_located_not_only_the_first() {
        let err = refusal(report_from_fields(None, None));
        assert!(err.contains("field=holds:absent"), "{err}");
        assert!(err.contains("field=text:absent"), "{err}");
    }

    // THE POSITIVE CONTROL, without which the two refusals above are indistinguishable from a
    // function that refuses unconditionally.
    #[test]
    fn a_well_formed_report_is_read() {
        let holds = Value::Bool(true);
        let text = Value::Str("standing".into());
        let report = match report_from_fields(Some(&holds), Some(&text)) {
            Ok(r) => r,
            Err(e) => panic!("well-formed report refused: {e}"),
        };
        assert!(report.holds);
        assert_eq!(report.text, "standing");
    }
}
