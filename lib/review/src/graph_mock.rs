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
//! - `credential_env`: Provides credential for LLM provider

use crate::graph::{build_diff_review_graph, build_inline_review_graph};
use crate::{Check, Criteria};
use gunbc_ir::transport::llm::mock;
use gunbc_ir::transport::{ShellResponse, TransportResponse};
use gunbc_ir::{SecretString, Value};
use gunbc_test::{extract_mock_requirements, InputConstraint, MockSpec, NodeExample, OutputMatcher};
use std::collections::BTreeMap;

fn mock_credential() -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "token".to_string(),
        Value::Secret(SecretString::new("<MOCK_API_KEY>")),
    );
    map.insert(
        "source_type".to_string(),
        Value::Str("static".to_string()),
    );
    map.insert("scheme".to_string(), Value::Str("bearer".to_string()));
    map.insert(
        "cap".to_string(),
        Value::Secret(SecretString::new("capability")),
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
#[gunbc_testgen_registry_macros::testgen_target(
    name = "review-inline",
    output = "lib/review/src/generated_tests_inline.rs",
    module = "review_inline_generated_tests",
    builder = "crate::graph::build_inline_review_graph()",
    no_boundary_tests
)]
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
    let criteria_json = Value::Json(serde_json::to_value(&criteria).unwrap());

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "review-inline")
        // Resource: credential_env provides credential
        .boundary("credential_env", "credential:llm", mock_credential())
        .expect("credential_env credential:llm should match type")
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
            criteria_json.clone(),
        )
        .input_mock("parse_response", "criteria", criteria_json.clone())
        .input_mock(
            "prepare_llm",
            "content",
            Value::Str("fn main() { let x: Option<i32> = None; x.unwrap(); }".into()),
        )
        .input_mock("prepare_llm", "provider", Value::Str("openai".into()))
        .input_mock("prepare_llm", "model", Value::Str("gpt-4o".into()))
        .expects_input("artifact", InputConstraint::NonEmpty)
        .expects_input("criteria", InputConstraint::NonEmpty)
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        .node_example(
            NodeExample::new("prepare_prompt")
                .input("artifact", Value::Str("fn main() {}".into()))
                .input("criteria", criteria_json.clone())
                .output("question", OutputMatcher::non_empty())
                .output("system_prompt", OutputMatcher::non_empty())
                .description("builds prompt text from artifact + criteria"),
        )
        .node_example(
            NodeExample::new("prepare_llm")
                .input("content", Value::Str("fn main() {}".into()))
                .input("question", Value::Str("Review this code".into()))
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .output("request", OutputMatcher::Any)
                .output(
                    "provider",
                    OutputMatcher::exact(Value::Str("openai".into())),
                )
                .description("builds an LLM request from prompt content"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("openai".into()))
                .output(
                    "service",
                    OutputMatcher::exact(Value::Str("openai".into())),
                )
                .output(
                    "env_var",
                    OutputMatcher::exact(Value::Str("OPENAI_API_KEY".into())),
                )
                .output(
                    "scheme",
                    OutputMatcher::exact(Value::Str("bearer".into())),
                )
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("resolves OpenAI auth scheme and env var"),
        )
        .node_example(
            NodeExample::new("credential_env")
                .input("service", Value::Str("openai".into()))
                .input("env_var", Value::Str("PATH".into()))
                .input("scheme", Value::Str("bearer".into()))
                .input("header_name", Value::Str(String::new()))
                .output("credential:llm", OutputMatcher::Any)
                .description("loads credential from environment variable"),
        )
        .node_example(
            NodeExample::new("parse_llm")
                .input("provider", Value::Str("openai".into()))
                .input(
                    "response",
                    Value::Response(TransportResponse::Rest(
                        mock::mock_openai_response(&llm_answer),
                    )),
                )
                .output("answer", OutputMatcher::non_empty())
                .description("extracts answer text from LLM response"),
        )
        .node_example(
            NodeExample::new("parse_response")
                .input("answer", Value::Str(llm_answer.clone()))
                .input("criteria", criteria_json.clone())
                .output("output", OutputMatcher::Any)
                .output("errors", OutputMatcher::Any)
                .description("parses LLM answer into review output"),
        )
}

// ============================================================================
// Diff Review MockSpec
// ============================================================================

/// Mock specification for the diff review graph.
///
/// Uses the typed mock builder pattern: the DAG is built first, requirements
/// are extracted from its structure, and mocks are type-checked at construction.
#[gunbc_testgen_registry_macros::testgen_target(
    name = "review-diff",
    output = "lib/review/src/generated_tests_diff.rs",
    module = "review_diff_generated_tests",
    builder = "crate::graph::build_diff_review_graph()",
    tool = "review",
    no_boundary_tests
)]
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
    let criteria = default_criteria();
    let criteria_json = Value::Json(serde_json::to_value(&criteria).unwrap());

    // Extract typed requirements from DAG structure
    extract_mock_requirements(&dag, "review-diff")
        // Resource: credential_env provides credential
        .boundary("credential_env", "credential:llm", mock_credential())
        .expect("credential_env credential:llm should match type")
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
        // Input mocks / expectations (repo_path is a required entrypoint)
        .input_mock("prepare_diff", "repo_path", Value::Str(".".into()))
        .expects_input("repo_path", InputConstraint::Any)
        // No input mocks needed — provider, model, criteria come from config node
        .node_example(
            NodeExample::new("config")
                .output(
                    "provider",
                    OutputMatcher::exact(Value::Str("openai".into())),
                )
                .output("model", OutputMatcher::exact(Value::Str("gpt-4o".into())))
                .output("criteria", OutputMatcher::Any)
                .description("emits pipeline config constants"),
        )
        .node_example(
            NodeExample::new("prepare_diff")
                .input("base_ref", Value::Str("main".into()))
                .input("repo_path", Value::Str(".".into()))
                .output("request", OutputMatcher::Any)
                .description("builds git diff request"),
        )
        .node_example(
            NodeExample::new("parse_diff")
                .input(
                    "response",
                    Value::Response(TransportResponse::Shell(ShellResponse::ok(diff_output))),
                )
                .output("diff_files", OutputMatcher::Any)
                .output("stats", OutputMatcher::Any)
                .description("parses diff response into files + stats"),
        )
        .node_example(
            NodeExample::new("format_artifact")
                .input("diff_files", Value::str_map({
                    let mut diff_files = BTreeMap::new();
                    diff_files.insert(
                        "src/main.rs".to_string(),
                        "@@ -1,2 +1,3 @@\n fn main() {}\n+println!(\"hi\");".to_string(),
                    );
                    diff_files
                }))
                .output("artifact", OutputMatcher::contains("src/main.rs"))
                .description("formats diff files into review artifact"),
        )
        .node_example(
            NodeExample::new("prepare_prompt")
                .input("artifact", Value::Str("diff artifact".into()))
                .input("criteria", criteria_json.clone())
                .output("question", OutputMatcher::non_empty())
                .output("system_prompt", OutputMatcher::non_empty())
                .description("builds prompt text from diff artifact"),
        )
        .node_example(
            NodeExample::new("prepare_llm")
                .input("content", Value::Str("diff artifact".into()))
                .input("question", Value::Str("Review the diff".into()))
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .output("request", OutputMatcher::Any)
                .output(
                    "provider",
                    OutputMatcher::exact(Value::Str("openai".into())),
                )
                .description("builds an LLM request from diff prompt"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("openai".into()))
                .output(
                    "service",
                    OutputMatcher::exact(Value::Str("openai".into())),
                )
                .output(
                    "env_var",
                    OutputMatcher::exact(Value::Str("OPENAI_API_KEY".into())),
                )
                .output(
                    "scheme",
                    OutputMatcher::exact(Value::Str("bearer".into())),
                )
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("resolves OpenAI auth scheme and env var"),
        )
        .node_example(
            NodeExample::new("credential_env")
                .input("service", Value::Str("openai".into()))
                .input("env_var", Value::Str("PATH".into()))
                .input("scheme", Value::Str("bearer".into()))
                .input("header_name", Value::Str(String::new()))
                .output("credential:llm", OutputMatcher::Any)
                .description("loads credential from environment variable"),
        )
        .node_example(
            NodeExample::new("parse_llm")
                .input("provider", Value::Str("openai".into()))
                .input(
                    "response",
                    Value::Response(TransportResponse::Rest(
                        mock::mock_openai_response(&llm_answer),
                    )),
                )
                .output("answer", OutputMatcher::non_empty())
                .description("extracts answer text from LLM response"),
        )
        .node_example(
            NodeExample::new("parse_response")
                .input("answer", Value::Str(llm_answer.clone()))
                .input("criteria", criteria_json.clone())
                .output("output", OutputMatcher::Any)
                .output("errors", OutputMatcher::Any)
                .description("parses LLM answer into review output"),
        )
}
