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
use gunbc_test::{InputConstraint, MockSpec};

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
        // Boundary: execute (transport) outputs
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
        // Boundary: execute (transport) outputs
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
}

/// Mock specification for testing API key as secret.
///
/// Models the secret resolution pattern: the API key flows as a
/// `Value::Secret` and gets resolved at the transport boundary.
pub fn secret_api_key_mock_spec() -> MockSpec {
    MockSpec::new("llm-secrets")
        .boundary(
            "resolve_api_key",
            "api_key",
            Value::Secret(SecretString::new("<MOCK_API_KEY>")),
        )
        .boundary(
            "execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                mock::mock_openai_response("Response with secret auth."),
            )),
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
}

/// Mock specification for testing rate limiting / error scenarios.
pub fn rate_limited_mock_spec() -> MockSpec {
    let error_response = mock::mock_openai_error(
        429,
        "rate_limit_exceeded",
        "Rate limit exceeded. Please retry after 60 seconds.",
    );

    MockSpec::new("llm-rate-limited")
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
