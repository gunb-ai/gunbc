//! Mock specification for LLM chat completion workflows.
//!
//! This file declares:
//! - What mock values boundary nodes provide
//! - What input constraints upstream must satisfy
//!
//! Used by testgen for:
//! - Dry-run testing with realistic mock values
//! - Chain validation with other tools
//!
//! # Provider-specific mocks
//!
//! Each provider has its own mock spec since their response formats differ.
//! The `execute` boundary returns provider-specific REST responses that
//! the `parse` node then converts to unified outputs.

use gunbc_ir::transport::llm::mock;
use gunbc_ir::{Value, SecretString};
use gunbc_test::{InputConstraint, MockSpec, NodeExample, OutputMatcher};

/// Mock specification for OpenAI chat completion.
///
/// # Boundary Mocks
///
/// - `execute`: Transport boundary returning an OpenAI-format response
/// - `parse`: Final outputs (content, model, tokens)
///
/// # Input Expectations
///
/// - `provider`: Must be "openai"
/// - `model`: Non-empty string (e.g., "gpt-4o")
/// - `messages`: JSON array of {role, content}
pub fn openai_mock_spec() -> MockSpec {
    let response = mock::mock_openai_response("The code looks good. No issues found.");

    MockSpec::new("llm-openai")
        // Input mocks for DAG entry points (dangling inputs on prepare node)
        .input_mock("prepare", "provider", Value::Str("openai".into()))
        .input_mock("prepare", "model", Value::Str("gpt-4o".into()))
        .input_mock("prepare", "messages", Value::Json(serde_json::json!([
            {"role": "user", "content": "Review this code: fn main() {}"}
        ])))
        // Transport mock: execute node response (DryRun interception)
        .transport_mock(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response.clone())),
        )
        // Boundary: execute (transport) outputs (for documentation/chain validation)
        .boundary(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        // Boundary: parse outputs (what downstream consumers see)
        .boundary(
            "parse",
            "content",
            Value::Str("The code looks good. No issues found.".into()),
        )
        .boundary(
            "parse",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .boundary(
            "parse",
            "finish_reason",
            Value::Str("Stop".into()),
        )
        .boundary("parse", "input_tokens", Value::Int(10))
        .boundary("parse", "output_tokens", Value::Int(20))
        // Input expectations
        .expects_input("provider", InputConstraint::OneOf(vec![Value::Str("openai".into())]))
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("prepare")
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .input("messages", Value::Str("Hello".into()))
                .output("request", OutputMatcher::non_empty())
                .output("provider", OutputMatcher::exact(Value::Str("openai".into())))
                .description("OpenAI prepare emits REST request and echoes provider"),
        )
        .node_example(
            NodeExample::new("parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("OpenAI parse handles skipped transport response"),
        )
}

/// Mock specification for Anthropic chat completion.
///
/// # Boundary Mocks
///
/// Same structure as OpenAI but with Anthropic-format responses.
pub fn anthropic_mock_spec() -> MockSpec {
    let response = mock::mock_anthropic_response(
        "The function handles edge cases well. Consider adding a doc comment.",
    );

    MockSpec::new("llm-anthropic")
        // Input mocks for DAG entry points (dangling inputs on prepare node)
        .input_mock("prepare", "provider", Value::Str("anthropic".into()))
        .input_mock("prepare", "model", Value::Str("claude-sonnet-4-20250514".into()))
        .input_mock("prepare", "messages", Value::Json(serde_json::json!([
            {"role": "user", "content": "Review this function for edge cases"}
        ])))
        // Transport mock: execute node response (DryRun interception)
        .transport_mock(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response.clone())),
        )
        // Boundary: execute (transport) outputs (for documentation/chain validation)
        .boundary(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        // Boundary: parse outputs
        .boundary(
            "parse",
            "content",
            Value::Str(
                "The function handles edge cases well. Consider adding a doc comment.".into(),
            ),
        )
        .boundary(
            "parse",
            "model",
            Value::Str("claude-sonnet-4-20250514".into()),
        )
        .boundary(
            "parse",
            "finish_reason",
            Value::Str("Stop".into()),
        )
        .boundary("parse", "input_tokens", Value::Int(10))
        .boundary("parse", "output_tokens", Value::Int(20))
        // Input expectations
        .expects_input(
            "provider",
            InputConstraint::OneOf(vec![Value::Str("anthropic".into())]),
        )
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("prepare")
                .input("provider", Value::Str("anthropic".into()))
                .input("model", Value::Str("claude-sonnet-4-20250514".into()))
                .input("messages", Value::Str("Hello".into()))
                .output("request", OutputMatcher::non_empty())
                .output("provider", OutputMatcher::exact(Value::Str("anthropic".into())))
                .description("Anthropic prepare emits REST request and echoes provider"),
        )
        .node_example(
            NodeExample::new("parse")
                .input("provider", Value::Str("anthropic".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Anthropic parse handles skipped transport response"),
        )
}

/// Mock specification for code review workflow.
///
/// Simulates a code review request with a detailed response.
pub fn code_review_mock_spec() -> MockSpec {
    let review_content = "\
## Code Review\n\n\
### Issues Found\n\
1. **Missing error handling** on line 42 — `unwrap()` should be replaced with `?`\n\
2. **Unused import** `std::io` on line 3\n\n\
### Suggestions\n\
- Add doc comments to public functions\n\
- Consider extracting the validation logic into a separate function\n\n\
Overall: The code is clean and well-structured. Minor fixes recommended.";

    let response = mock::mock_openai_response_full(review_content, "gpt-4o", "stop", 150, 85);

    MockSpec::new("llm-code-review")
        // Input mocks for DAG entry points
        .input_mock("prepare", "provider", Value::Str("openai".into()))
        .input_mock("prepare", "model", Value::Str("gpt-4o".into()))
        .input_mock("prepare", "messages", Value::Json(serde_json::json!([
            {"role": "system", "content": "You are a code reviewer. Review the following code."},
            {"role": "user", "content": "fn main() { let x = Some(1); x.unwrap(); }"}
        ])))
        // Transport mock: execute node response
        .transport_mock(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response.clone())),
        )
        .boundary(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        .boundary(
            "parse",
            "content",
            Value::Str(review_content.into()),
        )
        .boundary("parse", "model", Value::Str("gpt-4o".into()))
        .boundary("parse", "finish_reason", Value::Str("Stop".into()))
        .boundary("parse", "input_tokens", Value::Int(150))
        .boundary("parse", "output_tokens", Value::Int(85))
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("prepare")
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .input("messages", Value::Str("Review this code".into()))
                .output("request", OutputMatcher::non_empty())
                .output("provider", OutputMatcher::non_empty())
                .description("Code review prepare emits REST request"),
        )
        .node_example(
            NodeExample::new("parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Code review parse handles skipped transport response"),
        )
}

/// Mock specification for testing API key as secret.
///
/// Models the secret resolution pattern: the API key flows as a
/// `Value::Secret` and gets resolved at the transport boundary.
pub fn secret_api_key_mock_spec() -> MockSpec {
    let response = mock::mock_openai_response("Response with secret auth.");

    MockSpec::new("llm-secrets")
        // Input mocks for DAG entry points
        .input_mock("prepare", "provider", Value::Str("openai".into()))
        .input_mock("prepare", "model", Value::Str("gpt-4o".into()))
        .input_mock("prepare", "messages", Value::Json(serde_json::json!([
            {"role": "user", "content": "Hello with auth"}
        ])))
        // Transport mock: execute node response
        .transport_mock(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response.clone())),
        )
        .boundary(
            "resolve_api_key",
            "api_key",
            Value::Secret(SecretString::new("<MOCK_API_KEY>")),
        )
        .boundary(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        .boundary(
            "parse",
            "content",
            Value::Str("Response with secret auth.".into()),
        )
        .boundary("parse", "model", Value::Str("gpt-4o".into()))
        .boundary("parse", "finish_reason", Value::Str("Stop".into()))
        .boundary("parse", "input_tokens", Value::Int(10))
        .boundary("parse", "output_tokens", Value::Int(10))
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        // API key resource: lease-based (keys can expire)
        .resource_lease("api:openai_key", 3_600_000) // 1 hour
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("prepare")
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .input("messages", Value::Str("Hello".into()))
                .output("request", OutputMatcher::non_empty())
                .output("provider", OutputMatcher::non_empty())
                .description("Secret auth prepare emits REST request"),
        )
        .node_example(
            NodeExample::new("parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Secret auth parse handles skipped transport response"),
        )
}

/// Mock specification for testing rate limiting / error scenarios.
pub fn rate_limited_mock_spec() -> MockSpec {
    let error_response = mock::mock_openai_error(
        429,
        "rate_limit_exceeded",
        "Rate limit exceeded. Please retry after 60 seconds.",
    );

    MockSpec::new("llm-rate-limited")
        // Input mocks for DAG entry points
        .input_mock("prepare", "provider", Value::Str("openai".into()))
        .input_mock("prepare", "model", Value::Str("gpt-4o".into()))
        .input_mock("prepare", "messages", Value::Json(serde_json::json!([
            {"role": "user", "content": "This request will be rate limited"}
        ])))
        // Transport mock: execute node returns error response
        .transport_mock(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(error_response.clone())),
        )
        .boundary(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(error_response)),
        )
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_test::validate_chain;
    use std::collections::HashMap;

    #[test]
    fn test_openai_mock_spec_has_boundaries() {
        let spec = openai_mock_spec();
        assert!(spec.get_boundary_mock("execute", "response").is_some());
        assert!(spec.get_boundary_mock("parse", "content").is_some());
        assert!(spec.get_boundary_mock("parse", "model").is_some());
    }

    #[test]
    fn test_openai_mock_spec_content() {
        let spec = openai_mock_spec();
        let content = spec.get_boundary_mock("parse", "content").unwrap();
        if let Value::Str(s) = content {
            assert!(s.contains("looks good"));
        } else {
            panic!("Expected string content");
        }
    }

    #[test]
    fn test_anthropic_mock_spec_has_boundaries() {
        let spec = anthropic_mock_spec();
        assert!(spec.get_boundary_mock("execute", "response").is_some());
        assert!(spec.get_boundary_mock("parse", "content").is_some());
    }

    #[test]
    fn test_anthropic_mock_spec_content() {
        let spec = anthropic_mock_spec();
        let content = spec.get_boundary_mock("parse", "content").unwrap();
        if let Value::Str(s) = content {
            assert!(s.contains("doc comment"));
        } else {
            panic!("Expected string content");
        }
    }

    #[test]
    fn test_code_review_mock_spec() {
        let spec = code_review_mock_spec();
        let content = spec.get_boundary_mock("parse", "content").unwrap();
        if let Value::Str(s) = content {
            assert!(s.contains("Code Review"));
            assert!(s.contains("Missing error handling"));
        } else {
            panic!("Expected string content");
        }
    }

    #[test]
    fn test_secret_api_key_mock_spec() {
        let spec = secret_api_key_mock_spec();

        // Secret value should be present
        let api_key = spec
            .get_boundary_mock("resolve_api_key", "api_key")
            .unwrap();
        assert!(matches!(api_key, Value::Secret(_)));

        // Secret should be redacted in display
        assert_eq!(format!("{}", api_key), "***");

        // API key lease should be present
        assert!(spec.get_resource("api:openai_key").is_some());
    }

    #[test]
    fn test_rate_limited_mock_spec() {
        let spec = rate_limited_mock_spec();
        let response = spec.get_boundary_mock("execute", "response").unwrap();
        assert!(matches!(response, Value::Response(_)));
    }

    #[test]
    fn test_chain_validation_self() {
        let spec = openai_mock_spec();
        let mapping = HashMap::new();
        let result = validate_chain(&spec, &spec, &mapping);
        assert!(result.is_ok());
    }

    #[test]
    fn test_secret_lease_expiration() {
        let spec = secret_api_key_mock_spec();
        let resource = spec.get_resource("api:openai_key").unwrap();

        // Should not timeout before duration
        assert!(!resource.should_timeout(1_800_000)); // 30 min
        // Should timeout after duration
        assert!(resource.should_timeout(3_600_001)); // 1 hour + 1ms
    }
}
