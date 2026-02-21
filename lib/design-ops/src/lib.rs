//! Typed design prompt/review transforms for SDLC runtime.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignRequest {
    pub title: String,
    pub idea: String,
    pub context: Option<String>,
    pub acceptance_tests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignReviewFinding {
    pub severity: ReviewSeverity,
    pub check_id: String,
    pub observation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSeverity {
    Blocking,
    Suggestion,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignReviewSummary {
    pub blocking_count: usize,
    pub suggestion_count: usize,
    pub info_count: usize,
    pub approved: bool,
}

pub fn build_design_prompt(request: &DesignRequest) -> String {
    let context_block = request
        .context
        .as_ref()
        .map(|value| format!("\n\nContext:\n{value}"))
        .unwrap_or_default();
    let tests_block = if request.acceptance_tests.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nAcceptance tests:\n{}",
            request
                .acceptance_tests
                .iter()
                .map(|test| format!("- {test}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    format!(
        "Title: {}\n\nIdea:\n{}{}{}\n\nReturn markdown sections: Problem, Approach, Interfaces, Risks, Test Plan, Scope.",
        request.title, request.idea, context_block, tests_block
    )
}

pub fn parse_review_findings(text: &str) -> Vec<DesignReviewFinding> {
    text.lines()
        .filter_map(|line| parse_review_line(line.trim()))
        .collect()
}

pub fn summarize_review(findings: &[DesignReviewFinding]) -> DesignReviewSummary {
    let blocking_count = findings
        .iter()
        .filter(|finding| finding.severity == ReviewSeverity::Blocking)
        .count();
    let suggestion_count = findings
        .iter()
        .filter(|finding| finding.severity == ReviewSeverity::Suggestion)
        .count();
    let info_count = findings
        .iter()
        .filter(|finding| finding.severity == ReviewSeverity::Info)
        .count();
    DesignReviewSummary {
        blocking_count,
        suggestion_count,
        info_count,
        approved: blocking_count == 0,
    }
}

fn parse_review_line(line: &str) -> Option<DesignReviewFinding> {
    // Supported line format:
    // - [blocking] check-id: detail text
    if !line.starts_with("- [") {
        return None;
    }
    let closing = line.find("] ")?;
    let severity = &line[3..closing];
    let (severity, rest) = match severity {
        "blocking" => (ReviewSeverity::Blocking, &line[(closing + 2)..]),
        "suggestion" => (ReviewSeverity::Suggestion, &line[(closing + 2)..]),
        "info" => (ReviewSeverity::Info, &line[(closing + 2)..]),
        _ => return None,
    };
    let (check_id, observation) = rest.split_once(':')?;
    Some(DesignReviewFinding {
        severity,
        check_id: check_id.trim().to_string(),
        observation: observation.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_includes_context_and_acceptance_tests() {
        let prompt = build_design_prompt(&DesignRequest {
            title: "Improve intake idempotency".to_string(),
            idea: "Persist deterministic run keys".to_string(),
            context: Some("Current flow duplicates issue comments".to_string()),
            acceptance_tests: vec![
                "cargo test -q -p gunbc-dag --test sdlc_cli".to_string(),
                "cargo run -q -p gunbc-dag --bin gunbc-sdlc -- intake --dry-run".to_string(),
            ],
        });
        assert!(prompt.contains("Title: Improve intake idempotency"));
        assert!(prompt.contains("Context:\nCurrent flow duplicates issue comments"));
        assert!(prompt.contains("Acceptance tests:\n- cargo test -q -p gunbc-dag --test sdlc_cli"));
    }

    #[test]
    fn parse_and_summarize_review_findings() {
        let findings = parse_review_findings(
            "- [blocking] schema-version: version field is missing\n- [suggestion] retries: add explicit retry backoff note\n- [info] docs: include rollout notes",
        );
        assert_eq!(findings.len(), 3);
        let summary = summarize_review(&findings);
        assert_eq!(summary.blocking_count, 1);
        assert_eq!(summary.suggestion_count, 1);
        assert_eq!(summary.info_count, 1);
        assert!(!summary.approved);
    }
}
