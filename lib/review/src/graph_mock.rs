//! Mock specifications for review DAGs.
//!
//! Provides MockSpec definitions for dry-run testing and testgen.

use crate::{Criteria, Check, ReviewOutput, Finding, Location};
use gunbc_ir::transport::llm::mock;
use gunbc_ir::transport::ShellResponse;
use gunbc_ir::Value;
use gunbc_test::{InputConstraint, MockSpec};

// ============================================================================
// Default review criteria (for testing and CLI defaults)
// ============================================================================

/// Default code review criteria.
///
/// Covers common code quality checks suitable for general-purpose review.
pub fn default_criteria() -> Criteria {
    Criteria {
        name: "code-review".to_string(),
        description: "General code review for correctness, security, and quality".to_string(),
        checks: vec![
            Check {
                id: "correctness".to_string(),
                question: "Are there any bugs, logic errors, or incorrect behavior?".to_string(),
                examples: vec![
                    "Off-by-one errors".to_string(),
                    "Null/None dereference".to_string(),
                    "Missing error handling".to_string(),
                ],
            },
            Check {
                id: "security".to_string(),
                question: "Are there any security vulnerabilities?".to_string(),
                examples: vec![
                    "SQL injection".to_string(),
                    "Command injection".to_string(),
                    "Hardcoded credentials".to_string(),
                ],
            },
            Check {
                id: "performance".to_string(),
                question: "Are there any performance issues or inefficiencies?".to_string(),
                examples: vec![
                    "Unnecessary allocations in hot paths".to_string(),
                    "O(n^2) where O(n) is possible".to_string(),
                ],
            },
            Check {
                id: "clarity".to_string(),
                question: "Is the code clear and maintainable?".to_string(),
                examples: vec![
                    "Confusing variable names".to_string(),
                    "Missing context for complex logic".to_string(),
                ],
            },
        ],
    }
}

// ============================================================================
// Inline Review MockSpec
// ============================================================================

/// Mock specification for the inline review graph.
///
/// Suitable for testing `build_inline_review_graph()`.
pub fn inline_review_mock_spec() -> MockSpec {
    let review_json = serde_json::json!({
        "findings": [
            {
                "check_id": "correctness",
                "issue_key": "unwrap_without_context",
                "location": {"type": "file_line", "file": "src/main.rs", "line": 10},
                "observation": "Using unwrap() without context can make debugging difficult",
                "candidate_fix": "Use .expect(\"reason\") or ? operator"
            }
        ],
        "summary": "Found 1 issue: an unwrap() call that should use proper error handling."
    });

    let llm_answer = serde_json::to_string_pretty(&review_json).unwrap();
    let llm_response = mock::mock_openai_response(&llm_answer);
    let criteria = default_criteria();

    MockSpec::new("review-inline")
        // Input mocks for entrypoints (inline graph has NO config node —
        // it's the low-level building block)
        .input_mock(
            "prepare_prompt",
            "artifact",
            Value::Str("fn main() { let x: Option<i32> = None; x.unwrap(); }".into()),
        )
        .input_mock(
            "prepare_prompt",
            "criteria",
            Value::Json(serde_json::to_value(&criteria).unwrap()),
        )
        .input_mock("prepare_llm", "provider", Value::Str("openai".into()))
        .input_mock("prepare_llm", "model", Value::Str("gpt-4o".into()))
        // Transport mock: LLM execute
        .transport_mock(
            "execute_llm",
            "response",
            Value::Response(llm_response.clone().into()),
        )
        .boundary(
            "execute_llm",
            "response",
            Value::Response(llm_response.into()),
        )
        // Boundary: parse_response outputs
        .boundary(
            "parse_response",
            "output",
            Value::Json(serde_json::to_value(&ReviewOutput {
                schema_version: ReviewOutput::SCHEMA_VERSION.to_string(),
                criteria_name: "code-review".to_string(),
                source: "llm".to_string(),
                findings: vec![Finding {
                    id: crate::hash_finding_id("unwrap_without_context", "correctness"),
                    check_id: "correctness".to_string(),
                    issue_key: "unwrap_without_context".to_string(),
                    location: Location::FileLine {
                        file: "src/main.rs".to_string(),
                        line: 10,
                    },
                    observation: "Using unwrap() without context can make debugging difficult"
                        .to_string(),
                    candidate_fix: Some(
                        "Use .expect(\"reason\") or ? operator".to_string(),
                    ),
                }],
                candidate_remediations: None,
                summary: "Found 1 issue: an unwrap() call that should use proper error handling."
                    .to_string(),
            })
            .unwrap()),
        )
        .boundary(
            "parse_response",
            "errors",
            Value::Json(serde_json::json!([])),
        )
        .expects_input("artifact", InputConstraint::NonEmpty)
        .expects_input("criteria", InputConstraint::NonEmpty)
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
}

// ============================================================================
// Diff Review MockSpec
// ============================================================================

/// Mock specification for the diff review graph.
///
/// Suitable for testing `build_diff_review_graph()`.
pub fn diff_review_mock_spec() -> MockSpec {
    let diff_output = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,5 @@
 fn main() {
+    let x: Option<i32> = None;
+    x.unwrap();
 }";

    let review_json = serde_json::json!({
        "findings": [
            {
                "check_id": "correctness",
                "issue_key": "unwrap_on_none",
                "location": {"type": "file_line", "file": "src/main.rs", "line": 3},
                "observation": "unwrap() on a None value will panic at runtime",
                "candidate_fix": "Handle the None case explicitly"
            }
        ],
        "summary": "Found 1 correctness issue in the diff."
    });

    let llm_answer = serde_json::to_string_pretty(&review_json).unwrap();
    let llm_response = mock::mock_openai_response(&llm_answer);

    MockSpec::new("review-diff")
        // provider, model, criteria come from config node — no input mocks needed
        // Transport mock: git diff execute
        .transport_mock(
            "execute_diff",
            "response",
            Value::Response(ShellResponse::ok(diff_output).into()),
        )
        .boundary(
            "execute_diff",
            "response",
            Value::Response(ShellResponse::ok(diff_output).into()),
        )
        // Transport mock: LLM execute
        .transport_mock(
            "execute_llm",
            "response",
            Value::Response(llm_response.clone().into()),
        )
        .boundary(
            "execute_llm",
            "response",
            Value::Response(llm_response.into()),
        )
        // Boundary: parse_response outputs
        .boundary(
            "parse_response",
            "output",
            Value::Json(review_json),
        )
        .boundary(
            "parse_response",
            "errors",
            Value::Json(serde_json::json!([])),
        )
        // Boundary: parse_diff stats
        .boundary(
            "parse_diff",
            "stats",
            Value::Str("+2 -0 across 1 files".into()),
        )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_criteria() {
        let criteria = default_criteria();
        assert_eq!(criteria.name, "code-review");
        assert_eq!(criteria.checks.len(), 4);
        assert!(criteria.checks.iter().any(|c| c.id == "correctness"));
        assert!(criteria.checks.iter().any(|c| c.id == "security"));
    }

    #[test]
    fn test_inline_review_mock_spec_has_boundaries() {
        let spec = inline_review_mock_spec();
        assert!(spec.get_boundary_mock("execute_llm", "response").is_some());
        assert!(spec.get_boundary_mock("parse_response", "output").is_some());
        assert!(spec.get_boundary_mock("parse_response", "errors").is_some());
    }

    #[test]
    fn test_diff_review_mock_spec_has_boundaries() {
        let spec = diff_review_mock_spec();
        assert!(spec.get_boundary_mock("execute_diff", "response").is_some());
        assert!(spec.get_boundary_mock("execute_llm", "response").is_some());
        assert!(spec.get_boundary_mock("parse_response", "output").is_some());
        assert!(spec.get_boundary_mock("parse_diff", "stats").is_some());
    }

    #[test]
    fn test_diff_review_mock_spec_has_two_transport_boundaries() {
        let spec = diff_review_mock_spec();
        assert!(
            spec.get_boundary_mock("execute_diff", "response").is_some(),
            "should have git diff transport boundary"
        );
        assert!(
            spec.get_boundary_mock("execute_llm", "response").is_some(),
            "should have LLM transport boundary"
        );
    }
}
