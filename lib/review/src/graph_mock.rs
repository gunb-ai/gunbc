//! Mock specifications for review DAGs.
//!
//! This file uses the typed mock builder pattern to construct MockSpecs
//! that are "impossible by construction" — the DAG's requirements are
//! extracted and mocks are type-checked at construction time.
//!
//! # Review Graphs
//!
//! Two main review graphs with different input modes:
//! - `inline_review_mock_spec()`: For inline content review
//! - `diff_review_mock_spec()`: For git diff review
//!
//! # Transport Mocks
//!
//! - `execute_llm`: LLM API call
//! - `execute_diff`: Git diff command (diff graph only)
//!
//! # Resource Mocks
//!
//! - `auth_env`: Provides auth token for LLM provider

use crate::graph::{build_diff_review_graph, build_inline_review_graph};
use crate::{Check, Criteria};
use gunbc_ir::transport::llm::mock;
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{SecretString, Value};
use gunbc_test::{extract_mock_requirements, InputConstraint, MockSpec};
use std::collections::BTreeMap;

fn mock_auth_token(service: &str, env_var: &str) -> Value {
    let mut map = BTreeMap::new();
    map.insert("service".to_string(), Value::Str(service.to_string()));
    map.insert("env_var".to_string(), Value::Str(env_var.to_string()));
    map.insert(
        "token".to_string(),
        Value::Secret(SecretString::new("<MOCK_API_KEY>")),
    );
    Value::Map(map)
}

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
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
pub fn inline_review_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_inline_review_graph();

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

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "review-inline")
        // Resource: auth_env provides auth token
        .boundary("auth_env", "auth:llm", mock_auth_token("openai", "OPENAI_API_KEY"))
        .expect("auth_env auth:llm should match type")
        // Transport: execute_llm (LLM API call)
        .transport_response("execute_llm", "response", llm_response.into())
        .expect("execute_llm response should match type")
        // Build spec (pure terminal outputs like parse_response are computed, not mocked)
        .build_unchecked()
        // Input mocks for entrypoints (inline graph has NO config node)
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
        .expects_input("artifact", InputConstraint::NonEmpty)
        .expects_input("criteria", InputConstraint::NonEmpty)
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        .skip_node_example("auth_env")
}

// ============================================================================
// Diff Review MockSpec
// ============================================================================

/// Mock specification for the diff review graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
pub fn diff_review_mock_spec() -> MockSpec {
    // Build the actual DAG to extract requirements
    let dag = build_diff_review_graph();

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

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "review-diff")
        // Resource: auth_env provides auth token
        .boundary("auth_env", "auth:llm", mock_auth_token("openai", "OPENAI_API_KEY"))
        .expect("auth_env auth:llm should match type")
        // Transport: execute_diff (git diff command)
        .transport_response(
            "execute_diff",
            "response",
            TransportResponse::Shell(ShellResponse::ok(diff_output)),
        )
        .expect("execute_diff response should match type")
        // Transport: execute_llm (LLM API call)
        .transport_response("execute_llm", "response", llm_response.into())
        .expect("execute_llm response should match type")
        // Build spec (pure terminal outputs are computed, not mocked)
        .build_unchecked()
        // No input mocks needed — provider, model, criteria come from config node
        .skip_node_example("auth_env")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Mock spec tests (Pattern B - mock value properties)
    // ========================================================================

    #[test]
    fn test_default_criteria() {
        let criteria = default_criteria();
        assert_eq!(criteria.name, "code-review");
        assert_eq!(criteria.checks.len(), 4);
        assert!(criteria.checks.iter().any(|c| c.id == "correctness"));
        assert!(criteria.checks.iter().any(|c| c.id == "security"));
    }

    #[test]
    fn test_typed_builder_rejects_wrong_slot() {
        let dag = build_inline_review_graph();
        let reqs = extract_mock_requirements(&dag, "review-inline");

        // Try to set a mock for a non-existent node
        let result = reqs.boundary_str("nonexistent_node", "port", "value");
        assert!(result.is_err());
    }
}
