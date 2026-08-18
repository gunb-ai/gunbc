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
    pub run_head: String,
    pub run_note: String,
    pub roster_len: usize,
    pub still_red: usize,
    pub now_passes: usize,
    pub not_evaluated: usize,
    pub rows: Vec<ExpectedRedRosterJoinRow>,
}

impl ExpectedRedRosterJoinReport {
    pub fn new(run_head: String, run_note: String, roster: &[String]) -> Self {
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
            roster_len: roster.len(),
            still_red: 0,
            now_passes: 0,
            not_evaluated: roster.len(),
            rows,
        }
    }

    pub fn record_observed(&mut self, identity: &str, verdict: &WitnessEvalVerdict) {
        let Some(row) = self.row_mut(identity) else {
            return;
        };
        let (disposition, detail) = classify_verdict(verdict);
        row.disposition = disposition;
        row.detail = detail;
        self.recompute_counts();
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
        self.recompute_counts();
    }

    fn row_mut(&mut self, identity: &str) -> Option<&mut ExpectedRedRosterJoinRow> {
        self.rows.iter_mut().find(|row| row.identity == identity)
    }

    fn recompute_counts(&mut self) {
        self.still_red = 0;
        self.now_passes = 0;
        self.not_evaluated = 0;
        for row in &self.rows {
            match row.disposition {
                ExpectedRedJoinDisposition::StillRed => self.still_red += 1,
                ExpectedRedJoinDisposition::NowPasses => self.now_passes += 1,
                ExpectedRedJoinDisposition::NotEvaluated { .. } => self.not_evaluated += 1,
            }
        }
        self.roster_len = self.rows.len();
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
    writeln!(file, "# run_head\t{}", report.run_head)
        .map_err(|e| format!("expected_red_roster_join header: {e}"))?;
    writeln!(file, "# run_note\t{}", report.run_note.replace('\t', " "))
        .map_err(|e| format!("expected_red_roster_join header: {e}"))?;
    writeln!(
        file,
        "# summary\troster={}\tstill_red={}\tnow_passes={}\tnot_evaluated={}",
        report.roster_len, report.still_red, report.now_passes, report.not_evaluated
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
    eprintln!(
        "[expected-red-roster-join] roster={} still_red={} now_passes={} not_evaluated={} \
         (head={})",
        report.roster_len,
        report.still_red,
        report.now_passes,
        report.not_evaluated,
        report.run_head
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
            "deadbeef".to_string(),
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
        assert_eq!(report.roster_len, 3);
        assert_eq!(report.now_passes, 1);
        assert_eq!(report.not_evaluated, 2);
        assert_eq!(report.still_red, 0);
        let c = report.rows.iter().find(|r| r.identity == "c.w").unwrap();
        assert!(matches!(
            c.disposition,
            ExpectedRedJoinDisposition::NotEvaluated {
                ref reason
            } if reason == "not_in_executed_manifest"
        ));
    }
}
