//! Identity-grain join of `v2.workflow.floor_expected_red` against witness verdicts.
//!
//! Every enrolled identity receives exactly one of three outcomes:
//! - `StillRed` — the witness ran and failed on its subject (not infra/budget flake).
//! - `NowPasses` — the witness ran and returned true; retire the enrollment, keep the witness.
//! - `NotEvaluated` — no subject verdict (not in manifest, host tool missing, …).
//!
//! Completeness is an identity join over the full roster, never a scan of failure-log lines.

use std::collections::BTreeMap;
use std::io::Write;

/// Minimal verdict input so this module does not depend on `cli_run`'s full claim machinery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessEvalVerdict {
    Passed,
    BoolFalse,
    NotBool(String),
    RuntimeError(String),
    HostToolUnresolved {
        name: String,
        probed: Vec<String>,
    },
    BudgetExceeded {
        elapsed_ms: u64,
        budget_ms: u64,
        kind: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedRedJoinDisposition {
    StillRed,
    NowPasses,
    NotEvaluated { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedRedRosterJoinRow {
    pub identity: String,
    pub disposition: ExpectedRedJoinDisposition,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedRedRosterJoinReport {
    /// `git rev-parse HEAD` when available; absent rather than a fabricated placeholder.
    pub run_head: Option<String>,
    pub run_note: String,
    pub rows: Vec<ExpectedRedRosterJoinRow>,
}

impl ExpectedRedRosterJoinReport {
    pub fn new(run_head: Option<String>, run_note: String, roster: &[String]) -> Self {
        let rows = roster
            .iter()
            .map(|identity| ExpectedRedRosterJoinRow {
                identity: identity.clone(),
                disposition: ExpectedRedJoinDisposition::NotEvaluated {
                    reason: "not_observed".to_string(),
                },
                detail: String::new(),
            })
            .collect();
        Self {
            run_head,
            run_note,
            rows,
        }
    }

    pub fn roster_len(&self) -> usize {
        self.rows.len()
    }

    pub fn still_red(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| matches!(row.disposition, ExpectedRedJoinDisposition::StillRed))
            .count()
    }

    pub fn now_passes(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| matches!(row.disposition, ExpectedRedJoinDisposition::NowPasses))
            .count()
    }

    pub fn not_evaluated(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| {
                matches!(
                    row.disposition,
                    ExpectedRedJoinDisposition::NotEvaluated { .. }
                )
            })
            .count()
    }

    pub fn record_observed(&mut self, identity: &str, verdict: &WitnessEvalVerdict) {
        let Some(row) = self.row_mut(identity) else {
            return;
        };
        let (disposition, detail) = classify_verdict(verdict);
        row.disposition = disposition;
        row.detail = detail;
    }

    pub fn finalize_not_observed(&mut self) {
        for row in &mut self.rows {
            if matches!(
                row.disposition,
                ExpectedRedJoinDisposition::NotEvaluated { ref reason }
                    if reason == "not_observed"
            ) {
                row.disposition = ExpectedRedJoinDisposition::NotEvaluated {
                    reason: "not_in_executed_manifest".to_string(),
                };
                row.detail =
                    "enrolled on roster but no matching claim executed in this run".to_string();
            }
        }
    }

    fn row_mut(&mut self, identity: &str) -> Option<&mut ExpectedRedRosterJoinRow> {
        self.rows.iter_mut().find(|row| row.identity == identity)
    }
}

pub fn classify_verdict(verdict: &WitnessEvalVerdict) -> (ExpectedRedJoinDisposition, String) {
    match verdict {
        WitnessEvalVerdict::Passed => (
            ExpectedRedJoinDisposition::NowPasses,
            "witness returned Bool(true)".to_string(),
        ),
        WitnessEvalVerdict::BoolFalse => (
            ExpectedRedJoinDisposition::StillRed,
            "witness returned Bool(false)".to_string(),
        ),
        WitnessEvalVerdict::NotBool(got) => (
            ExpectedRedJoinDisposition::StillRed,
            format!("witness answered {got}, not a Bool"),
        ),
        WitnessEvalVerdict::RuntimeError(message) => {
            (ExpectedRedJoinDisposition::StillRed, message.clone())
        }
        WitnessEvalVerdict::HostToolUnresolved { name, probed } => (
            ExpectedRedJoinDisposition::NotEvaluated {
                reason: "host_tool_unresolved".to_string(),
            },
            format!(
                "host tool unresolved: {name:?} (probed: {})",
                probed.join(", ")
            ),
        ),
        WitnessEvalVerdict::BudgetExceeded {
            elapsed_ms,
            budget_ms,
            kind,
        } => (
            ExpectedRedJoinDisposition::StillRed,
            format!(
                "exceeded {kind} budget ({elapsed_ms}ms elapsed against {budget_ms}ms hard \
                 cutoff)"
            ),
        ),
    }
}

pub fn disposition_label(disposition: &ExpectedRedJoinDisposition) -> &'static str {
    match disposition {
        ExpectedRedJoinDisposition::StillRed => "still_red",
        ExpectedRedJoinDisposition::NowPasses => "now_passes",
        ExpectedRedJoinDisposition::NotEvaluated { .. } => "not_evaluated",
    }
}

pub fn not_evaluated_reason(disposition: &ExpectedRedJoinDisposition) -> &str {
    match disposition {
        ExpectedRedJoinDisposition::NotEvaluated { reason } => reason.as_str(),
        _ => "",
    }
}

pub fn write_join_tsv(path: &str, report: &ExpectedRedRosterJoinReport) -> Result<(), String> {
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("expected_red_roster_join create {path}: {e}"))?;
    match &report.run_head {
        Some(head) => writeln!(file, "# run_head\t{head}")
            .map_err(|e| format!("expected_red_roster_join header: {e}"))?,
        None => writeln!(file, "# run_head\t")
            .map_err(|e| format!("expected_red_roster_join header: {e}"))?,
    }
    writeln!(file, "# run_note\t{}", report.run_note.replace('\t', " "))
        .map_err(|e| format!("expected_red_roster_join header: {e}"))?;
    writeln!(
        file,
        "# summary\troster={}\tstill_red={}\tnow_passes={}\tnot_evaluated={}",
        report.roster_len(),
        report.still_red(),
        report.now_passes(),
        report.not_evaluated()
    )
    .map_err(|e| format!("expected_red_roster_join header: {e}"))?;
    writeln!(file, "identity\tdisposition\tnot_evaluated_reason\tdetail")
        .map_err(|e| format!("expected_red_roster_join header: {e}"))?;
    for row in &report.rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}",
            row.identity,
            disposition_label(&row.disposition),
            not_evaluated_reason(&row.disposition),
            row.detail.replace('\t', " ").replace('\n', " ")
        )
        .map_err(|e| format!("expected_red_roster_join row: {e}"))?;
    }
    Ok(())
}

pub fn emit_join_summary(report: &ExpectedRedRosterJoinReport) {
    let head = report
        .run_head
        .as_deref()
        .map_or("(unresolved)", |head| head);
    eprintln!(
        "[expected-red-roster-join] roster={} still_red={} now_passes={} not_evaluated={} \
         (head={head})",
        report.roster_len(),
        report.still_red(),
        report.now_passes(),
        report.not_evaluated(),
    );
    eprintln!("[expected-red-roster-join] {}", report.run_note);
    let mut reason_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &report.rows {
        if let ExpectedRedJoinDisposition::NotEvaluated { ref reason } = row.disposition {
            *reason_counts.entry(reason.as_str()).or_default() += 1;
        }
    }
    for (reason, count) in reason_counts {
        eprintln!("[expected-red-roster-join] not_evaluated.{reason}={count}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_exceeded_at_hard_cutoff_is_still_red() {
        let verdict = WitnessEvalVerdict::BudgetExceeded {
            elapsed_ms: 509,
            budget_ms: 500,
            kind: "wall",
        };
        let (disp, detail) = classify_verdict(&verdict);
        assert_eq!(disp, ExpectedRedJoinDisposition::StillRed);
        assert!(detail.contains("hard cutoff"));
    }

    #[test]
    fn substantive_budget_overrun_is_still_red() {
        let verdict = WitnessEvalVerdict::BudgetExceeded {
            elapsed_ms: 600,
            budget_ms: 500,
            kind: "cpu",
        };
        let (disp, _) = classify_verdict(&verdict);
        assert_eq!(disp, ExpectedRedJoinDisposition::StillRed);
    }

    #[test]
    fn host_tool_unresolved_is_not_evaluated() {
        let verdict = WitnessEvalVerdict::HostToolUnresolved {
            name: "cargo".to_string(),
            probed: vec!["/usr/bin/cargo".to_string()],
        };
        let (disp, detail) = classify_verdict(&verdict);
        assert!(matches!(
            disp,
            ExpectedRedJoinDisposition::NotEvaluated {
                reason
            } if reason == "host_tool_unresolved"
        ));
        assert!(detail.contains("cargo"));
    }

    #[test]
    fn pass_is_now_passes() {
        let (disp, _) = classify_verdict(&WitnessEvalVerdict::Passed);
        assert_eq!(disp, ExpectedRedJoinDisposition::NowPasses);
    }

    #[test]
    fn join_covers_full_roster() {
        let roster = vec!["a.w".to_string(), "b.w".to_string(), "c.w".to_string()];
        let mut report = ExpectedRedRosterJoinReport::new(
            Some("deadbeef".to_string()),
            "synthetic fixture".to_string(),
            &roster,
        );
        report.record_observed("a.w", &WitnessEvalVerdict::Passed);
        report.record_observed(
            "b.w",
            &WitnessEvalVerdict::HostToolUnresolved {
                name: "cargo".to_string(),
                probed: vec![],
            },
        );
        report.finalize_not_observed();
        assert_eq!(report.roster_len(), 3);
        assert_eq!(report.now_passes(), 1);
        assert_eq!(report.not_evaluated(), 2);
        assert_eq!(report.still_red(), 0);
        let c = report.rows.iter().find(|r| r.identity == "c.w").unwrap();
        assert!(matches!(
            c.disposition,
            ExpectedRedJoinDisposition::NotEvaluated {
                ref reason
            } if reason == "not_in_executed_manifest"
        ));
    }

    #[test]
    fn summary_counts_derive_from_rows() {
        let roster = vec!["x.w".to_string()];
        let mut report = ExpectedRedRosterJoinReport::new(None, "fixture".to_string(), &roster);
        assert_eq!(report.not_evaluated(), 1);
        report.record_observed("x.w", &WitnessEvalVerdict::BoolFalse);
        assert_eq!(report.still_red(), 1);
        assert_eq!(report.not_evaluated(), 0);
    }

    #[test]
    fn unresolved_run_head_writes_empty_tsv_field() {
        let report = ExpectedRedRosterJoinReport::new(None, "fixture".to_string(), &[]);
        let path = std::env::temp_dir().join("expected_red_roster_join_test.tsv");
        write_join_tsv(path.to_str().unwrap(), &report).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.starts_with("# run_head\t\n"));
        let _ = std::fs::remove_file(path);
    }
}
