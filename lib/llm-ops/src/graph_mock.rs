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

use gunbc_ir::transport::cloud::{
    CloudProviderKind, CloudRuntimeKind, CloudSecretConfig, CloudSecretRef,
};
use gunbc_ir::transport::llm::mock;
use gunbc_ir::{SecretString, Value};
use gunbc_test::{InputConstraint, MockSpec, NodeExample, OutputMatcher};
use std::collections::BTreeMap;

fn mock_credential() -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "token".to_string(),
        Value::Secret(SecretString::new("<MOCK_API_KEY>")),
    );
    map.insert("source_type".to_string(), Value::Str("static".to_string()));
    map.insert("scheme".to_string(), Value::Str("bearer".to_string()));
    map.insert(
        "cap".to_string(),
        Value::Secret(SecretString::new("capability")),
    );
    Value::Map(map)
}

fn mock_cloud_config() -> Value {
    CloudSecretConfig {
        provider: CloudProviderKind::Gcp,
        runtime: CloudRuntimeKind::LocalDev,
        audience: "local-dev".to_string(),
        project_or_account: "mock-secrets".to_string(),
        secret: CloudSecretRef {
            prefix: "ci-".to_string(),
            name: String::new(),
            delimiter: String::new(),
            version: None,
        },
        service_account_or_role: Some("ci-secrets@mock.iam.gserviceaccount.com".to_string()),
        impersonate_account_or_role: None,
    }
    .into()
}

fn with_cloud_env(spec: MockSpec) -> MockSpec {
    spec.boundary("cloud_env", "config", mock_cloud_config())
        .boundary(
            "cloud_env",
            "request_url",
            Value::Str("https://example.com/oidc".into()),
        )
        .boundary(
            "cloud_env",
            "request_token",
            Value::Str("mock-oidc-token".into()),
        )
        .boundary("bind_secret", "config", mock_cloud_config())
        .boundary("cloud_credential", "credential", mock_credential())
        .include_prefixed_runtime_mocks(
            "cloud_credential/gcp_wif_secret",
            &gunbc_lib_gcp_ops::graph_mock::gcp_local_mock_spec(),
        )
        // Probe-observer: GCP sub-DAG terminal needs chain-safe observer
        .live_expected_output(
            "cloud_credential/gcp_wif_secret/parse_set_iam",
            "ok",
            OutputMatcher::IsBool,
        )
}

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
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "llm-openai",
    builder = "crate::graph::build_chat_completion_graph()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "llm-openai",
    output = "lib/llm-ops/src/generated_tests.rs",
    module = "llm_openai_generated_tests",
    builder = "crate::graph::build_chat_completion_graph()",
    no_boundary_tests
)]
pub fn openai_mock_spec() -> MockSpec {
    let response = mock::mock_openai_response("The code looks good. No issues found.");

    with_cloud_env(MockSpec::new("llm-openai"))
        // Input mocks for DAG entry points (dangling inputs on prepare node and resolve_auth)
        .input_mock(
            "chat_completion/prepare",
            "provider",
            Value::Str("openai".into()),
        )
        .input_mock("resolve_auth", "provider", Value::Str("openai".into()))
        .input_mock(
            "chat_completion/prepare",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .input_mock(
            "chat_completion/prepare",
            "messages",
            Value::Json(serde_json::json!([
                {"role": "user", "content": "Review this code: fn main() {}"}
            ])),
        )
        // Transport mock: execute node response (DryRun interception)
        .transport_mock(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                response.clone(),
            )),
        )
        // Boundary: execute (transport) outputs (for documentation/chain validation)
        .boundary(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        // Boundary: parse outputs (what downstream consumers see)
        .boundary(
            "chat_completion/parse",
            "content",
            Value::Str("The code looks good. No issues found.".into()),
        )
        .boundary(
            "chat_completion/parse",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .boundary(
            "chat_completion/parse",
            "finish_reason",
            Value::Str("Stop".into()),
        )
        .boundary("chat_completion/parse", "input_tokens", Value::Int(10))
        .boundary("chat_completion/parse", "output_tokens", Value::Int(20))
        // Input expectations
        .expects_input(
            "provider",
            InputConstraint::OneOf(vec![Value::Str("openai".into())]),
        )
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("chat_completion/prepare")
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .input("messages", Value::Str("Hello".into()))
                .output("request", OutputMatcher::non_empty())
                .output(
                    "provider",
                    OutputMatcher::exact(Value::Str("openai".into())),
                )
                .description("OpenAI prepare emits REST request and echoes provider"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("openai".into()))
                .output("service", OutputMatcher::exact(Value::Str("openai".into())))
                .output("scheme", OutputMatcher::exact(Value::Str("bearer".into())))
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps OpenAI provider to API key env var"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input(
                    "response",
                    Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                        mock::mock_openai_response("Test response content."),
                    )),
                )
                .output(
                    "content",
                    OutputMatcher::exact(Value::Str("Test response content.".into())),
                )
                .output("model", OutputMatcher::exact(Value::Str("gpt-4o".into())))
                .output(
                    "finish_reason",
                    OutputMatcher::exact(Value::Str("Stop".into())),
                )
                .output("input_tokens", OutputMatcher::exact(Value::Int(10)))
                .output("output_tokens", OutputMatcher::exact(Value::Int(20)))
                .description("OpenAI parse extracts content, model, tokens from REST response"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("OpenAI parse handles skipped transport response"),
        )
        .resource_credential("credential:openai", Some(3_600_000))
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
        .skip_node_example("scope_preflight")
}

/// Mock specification for Anthropic chat completion.
///
/// # Boundary Mocks
///
/// Same structure as OpenAI but with Anthropic-format responses.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "llm-anthropic",
    builder = "crate::graph::build_chat_completion_graph()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "llm-anthropic",
    output = "lib/llm-ops/src/generated_tests_anthropic.rs",
    module = "llm_anthropic_generated_tests",
    builder = "crate::graph::build_chat_completion_graph()",
    no_boundary_tests
)]
pub fn anthropic_mock_spec() -> MockSpec {
    let response = mock::mock_anthropic_response(
        "The function handles edge cases well. Consider adding a doc comment.",
    );

    with_cloud_env(MockSpec::new("llm-anthropic"))
        // Input mocks for DAG entry points (dangling inputs on prepare node and resolve_auth)
        .input_mock(
            "chat_completion/prepare",
            "provider",
            Value::Str("anthropic".into()),
        )
        .input_mock("resolve_auth", "provider", Value::Str("anthropic".into()))
        .input_mock(
            "chat_completion/prepare",
            "model",
            Value::Str("claude-sonnet-4-20250514".into()),
        )
        .input_mock(
            "chat_completion/prepare",
            "messages",
            Value::Json(serde_json::json!([
                {"role": "user", "content": "Review this function for edge cases"}
            ])),
        )
        // Transport mock: execute node response (DryRun interception)
        .transport_mock(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                response.clone(),
            )),
        )
        // Boundary: execute (transport) outputs (for documentation/chain validation)
        .boundary(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        // Boundary: parse outputs
        .boundary(
            "chat_completion/parse",
            "content",
            Value::Str(
                "The function handles edge cases well. Consider adding a doc comment.".into(),
            ),
        )
        .boundary(
            "chat_completion/parse",
            "model",
            Value::Str("claude-sonnet-4-20250514".into()),
        )
        .boundary(
            "chat_completion/parse",
            "finish_reason",
            Value::Str("Stop".into()),
        )
        .boundary("chat_completion/parse", "input_tokens", Value::Int(10))
        .boundary("chat_completion/parse", "output_tokens", Value::Int(20))
        // Input expectations
        .expects_input(
            "provider",
            InputConstraint::OneOf(vec![Value::Str("anthropic".into())]),
        )
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("chat_completion/prepare")
                .input("provider", Value::Str("anthropic".into()))
                .input("model", Value::Str("claude-sonnet-4-20250514".into()))
                .input("messages", Value::Str("Hello".into()))
                .output("request", OutputMatcher::non_empty())
                .output(
                    "provider",
                    OutputMatcher::exact(Value::Str("anthropic".into())),
                )
                .description("Anthropic prepare emits REST request and echoes provider"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("anthropic".into()))
                .output(
                    "service",
                    OutputMatcher::exact(Value::Str("anthropic".into())),
                )
                .output("scheme", OutputMatcher::exact(Value::Str("header".into())))
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str("x-api-key".into())),
                )
                .description("Resolve auth maps Anthropic provider to header auth"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("anthropic".into()))
                .input(
                    "response",
                    Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                        mock::mock_anthropic_response("Anthropic test response."),
                    )),
                )
                .output(
                    "content",
                    OutputMatcher::exact(Value::Str("Anthropic test response.".into())),
                )
                .output(
                    "model",
                    OutputMatcher::exact(Value::Str("claude-sonnet-4-20250514".into())),
                )
                .output(
                    "finish_reason",
                    OutputMatcher::exact(Value::Str("Stop".into())),
                )
                .output("input_tokens", OutputMatcher::exact(Value::Int(10)))
                .output("output_tokens", OutputMatcher::exact(Value::Int(20)))
                .description("Anthropic parse extracts content, model, tokens from REST response"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("anthropic".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Anthropic parse handles skipped transport response"),
        )
        .resource_credential("credential:anthropic", Some(3_600_000))
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
        .skip_node_example("scope_preflight")
}

/// Mock specification for code review workflow.
///
/// Simulates a code review request with a detailed response.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "llm-code-review",
    builder = "crate::graph::build_chat_completion_graph()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "llm-code-review",
    output = "lib/llm-ops/src/generated_tests_code_review.rs",
    module = "llm_code_review_generated_tests",
    builder = "crate::graph::build_chat_completion_graph()",
    no_boundary_tests
)]
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

    with_cloud_env(MockSpec::new("llm-code-review"))
        // Input mocks for DAG entry points
        .input_mock("chat_completion/prepare", "provider", Value::Str("openai".into()))
        .input_mock("resolve_auth", "provider", Value::Str("openai".into()))
        .input_mock("chat_completion/prepare", "model", Value::Str("gpt-4o".into()))
        .input_mock("chat_completion/prepare", "messages", Value::Json(serde_json::json!([
            {"role": "system", "content": "You are a code reviewer. Review the following code."},
            {"role": "user", "content": "fn main() { let x = Some(1); x.unwrap(); }"}
        ])))
        // Transport mock: execute node response
        .transport_mock(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response.clone())),
        )
        .boundary(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        .boundary(
            "chat_completion/parse",
            "content",
            Value::Str(review_content.into()),
        )
        .boundary("chat_completion/parse", "model", Value::Str("gpt-4o".into()))
        .boundary("chat_completion/parse", "finish_reason", Value::Str("Stop".into()))
        .boundary("chat_completion/parse", "input_tokens", Value::Int(150))
        .boundary("chat_completion/parse", "output_tokens", Value::Int(85))
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("chat_completion/prepare")
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .input("messages", Value::Str("Review this code".into()))
                .output("request", OutputMatcher::non_empty())
                .output("provider", OutputMatcher::non_empty())
                .description("Code review prepare emits REST request"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("openai".into()))
                .output("service", OutputMatcher::exact(Value::Str("openai".into())))
                .output(
                    "scheme",
                    OutputMatcher::exact(Value::Str("bearer".into())),
                )
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps OpenAI provider to bearer auth"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Response(
                    gunbc_ir::transport::TransportResponse::Rest(
                        mock::mock_openai_response("Code looks good.")
                    )
                ))
                .output("content", OutputMatcher::exact(Value::Str("Code looks good.".into())))
                .output("model", OutputMatcher::exact(Value::Str("gpt-4o".into())))
                .description("Code review parse extracts content from REST response"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Code review parse handles skipped transport response"),
        )
        .resource_credential("credential:openai", Some(3_600_000))
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
        .skip_node_example("scope_preflight")
}

/// Mock specification for testing API key as secret.
///
/// Models the secret resolution pattern: the API key flows as a
/// `Value::Secret` and gets resolved at the transport boundary.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "llm-secrets",
    builder = "crate::graph::build_chat_completion_graph()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "llm-secrets",
    output = "lib/llm-ops/src/generated_tests_secrets.rs",
    module = "llm_secrets_generated_tests",
    builder = "crate::graph::build_chat_completion_graph()",
    no_boundary_tests
)]
pub fn secret_api_key_mock_spec() -> MockSpec {
    let response = mock::mock_openai_response("Response with secret auth.");

    with_cloud_env(MockSpec::new("llm-secrets"))
        // Input mocks for DAG entry points
        .input_mock(
            "chat_completion/prepare",
            "provider",
            Value::Str("openai".into()),
        )
        .input_mock("resolve_auth", "provider", Value::Str("openai".into()))
        .input_mock(
            "chat_completion/prepare",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .input_mock(
            "chat_completion/prepare",
            "messages",
            Value::Json(serde_json::json!([
                {"role": "user", "content": "Hello with auth"}
            ])),
        )
        // Transport mock: execute node response
        .transport_mock(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                response.clone(),
            )),
        )
        .boundary(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        .boundary(
            "chat_completion/parse",
            "content",
            Value::Str("Response with secret auth.".into()),
        )
        .boundary(
            "chat_completion/parse",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .boundary(
            "chat_completion/parse",
            "finish_reason",
            Value::Str("Stop".into()),
        )
        .boundary("chat_completion/parse", "input_tokens", Value::Int(10))
        .boundary("chat_completion/parse", "output_tokens", Value::Int(10))
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        // API key resource: lease-based (keys can expire)
        .resource_lease("api:openai_key", 3_600_000) // 1 hour
        // Node I/O examples: verify pure node behavior
        .node_example(
            NodeExample::new("chat_completion/prepare")
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .input("messages", Value::Str("Hello".into()))
                .output("request", OutputMatcher::non_empty())
                .output("provider", OutputMatcher::non_empty())
                .description("Secret auth prepare emits REST request"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("openai".into()))
                .output("service", OutputMatcher::exact(Value::Str("openai".into())))
                .output("scheme", OutputMatcher::exact(Value::Str("bearer".into())))
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps OpenAI provider to bearer auth"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input(
                    "response",
                    Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                        mock::mock_openai_response("Authenticated response."),
                    )),
                )
                .output(
                    "content",
                    OutputMatcher::exact(Value::Str("Authenticated response.".into())),
                )
                .output("model", OutputMatcher::exact(Value::Str("gpt-4o".into())))
                .description("Secret auth parse extracts content from REST response"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Secret auth parse handles skipped transport response"),
        )
        .resource_credential("credential:openai", Some(3_600_000))
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
        .skip_node_example("scope_preflight")
}

/// Mock specification for testing rate limiting / error scenarios.
#[gunbc_testgen_registry_macros::testgen_target(skip)]
pub fn rate_limited_mock_spec() -> MockSpec {
    let error_response = mock::mock_openai_error(
        429,
        "rate_limit_exceeded",
        "Rate limit exceeded. Please retry after 60 seconds.",
    );

    with_cloud_env(MockSpec::new("llm-rate-limited"))
        // Input mocks for DAG entry points
        .input_mock(
            "chat_completion/prepare",
            "provider",
            Value::Str("openai".into()),
        )
        .input_mock("resolve_auth", "provider", Value::Str("openai".into()))
        .input_mock(
            "chat_completion/prepare",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .input_mock(
            "chat_completion/prepare",
            "messages",
            Value::Json(serde_json::json!([
                {"role": "user", "content": "This request will be rate limited"}
            ])),
        )
        // Transport mock: execute node returns error response
        .transport_mock(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                error_response.clone(),
            )),
        )
        .boundary(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(error_response)),
        )
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        .expects_input("messages", InputConstraint::NonEmpty)
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("openai".into()))
                .output("service", OutputMatcher::exact(Value::Str("openai".into())))
                .output("scheme", OutputMatcher::exact(Value::Str("bearer".into())))
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps OpenAI provider to bearer auth"),
        )
        .resource_credential("credential:openai", Some(3_600_000))
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
        .skip_node_example("scope_preflight")
}

/// Mock specification for credential lifecycle testing.
///
/// Tests credential acquire/timeout/refresh/revoke behavior via resource simulation.
/// Generates `generated_tests_credential.rs` with lifecycle-specific tests.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "llm-credential-lifecycle",
    builder = "crate::graph::build_chat_completion_graph()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "llm-credential-lifecycle",
    output = "lib/llm-ops/src/generated_tests_credential.rs",
    module = "llm_credential_lifecycle_generated_tests",
    builder = "crate::graph::build_chat_completion_graph()",
    no_boundary_tests,
    live_flow_tests,
    live_class = "integration",
    live_fermi = "M",
    live_required(
        "GCP_WIF_PROVIDER",
        "GCP_SECRETS_PROJECT",
        "GCP_SECRETS_PREFIX",
        "ACTIONS_ID_TOKEN_REQUEST_URL",
        "ACTIONS_ID_TOKEN_REQUEST_TOKEN"
    ),
    live_required_any_of("GCP_SECRETS_SA", "GCP_SECRETS_IMPERSONATE_SA")
)]
pub fn credential_lifecycle_mock_spec() -> MockSpec {
    let response = mock::mock_openai_response("Credential lifecycle test response.");

    with_cloud_env(MockSpec::new("llm-credential-lifecycle"))
        // Input mocks for DAG entry points
        .input_mock(
            "chat_completion/prepare",
            "provider",
            Value::Str("openai".into()),
        )
        .input_mock("resolve_auth", "provider", Value::Str("openai".into()))
        .input_mock(
            "chat_completion/prepare",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .input_mock(
            "chat_completion/prepare",
            "messages",
            Value::Json(serde_json::json!([
                {"role": "user", "content": "Credential lifecycle test"}
            ])),
        )
        // Transport mock
        .transport_mock(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                response.clone(),
            )),
        )
        .boundary(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        .boundary(
            "chat_completion/parse",
            "content",
            Value::Str("Credential lifecycle test response.".into()),
        )
        .boundary(
            "chat_completion/parse",
            "model",
            Value::Str("gpt-4o".into()),
        )
        .boundary(
            "chat_completion/parse",
            "finish_reason",
            Value::Str("Stop".into()),
        )
        .boundary("chat_completion/parse", "input_tokens", Value::Int(10))
        .boundary("chat_completion/parse", "output_tokens", Value::Int(20))
        // Live expectations: LLM returns non-empty content
        .live_expected_output(
            "chat_completion/parse",
            "content",
            OutputMatcher::non_empty(),
        )
        .live_expected_output("chat_completion/parse", "model", OutputMatcher::IsString)
        .live_expected_output(
            "chat_completion/parse",
            "finish_reason",
            OutputMatcher::IsString,
        )
        .live_expected_output(
            "chat_completion/parse",
            "input_tokens",
            OutputMatcher::IntGe(0),
        )
        .live_expected_output(
            "chat_completion/parse",
            "output_tokens",
            OutputMatcher::IntGe(0),
        )
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        // Credential resource: basic (acquire + timeout)
        .resource_credential("credential:openai", Some(3_600_000))
        // Credential resource: refreshable (acquire + timeout + refresh)
        .resource_credential_refreshable("credential:openai:refreshable", 3_600_000, 3_600_000)
        // Node I/O examples
        .node_example(
            NodeExample::new("chat_completion/prepare")
                .input("provider", Value::Str("openai".into()))
                .input("model", Value::Str("gpt-4o".into()))
                .input("messages", Value::Str("Credential lifecycle test".into()))
                .output("request", OutputMatcher::non_empty())
                .output(
                    "provider",
                    OutputMatcher::exact(Value::Str("openai".into())),
                )
                .description("Credential lifecycle prepare emits REST request and echoes provider"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("openai".into()))
                .output("service", OutputMatcher::exact(Value::Str("openai".into())))
                .output("scheme", OutputMatcher::exact(Value::Str("bearer".into())))
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str(String::new())),
                )
                .description("Resolve auth maps OpenAI provider to bearer auth"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input(
                    "response",
                    Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                        mock::mock_openai_response("Credential lifecycle test response."),
                    )),
                )
                .output(
                    "content",
                    OutputMatcher::exact(Value::Str("Credential lifecycle test response.".into())),
                )
                .output("model", OutputMatcher::exact(Value::Str("gpt-4o".into())))
                .description("Credential lifecycle parse extracts content from REST response"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("openai".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Parse handles skipped transport response"),
        )
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
        .skip_node_example("scope_preflight")
}

/// Mock specification for Anthropic credential lifecycle testing.
///
/// Tests credential acquire/timeout/refresh/revoke behavior via resource simulation.
#[gunbc_testgen_registry_macros::resource_test_target(
    name = "llm-credential-lifecycle-anthropic",
    builder = "crate::graph::build_chat_completion_graph()"
)]
#[gunbc_testgen_registry_macros::testgen_target(
    name = "llm-credential-lifecycle-anthropic",
    output = "lib/llm-ops/src/generated_tests_credential_anthropic.rs",
    module = "llm_credential_lifecycle_anthropic_generated_tests",
    builder = "crate::graph::build_chat_completion_graph()",
    no_boundary_tests,
    live_flow_tests,
    live_class = "integration",
    live_fermi = "M",
    live_required(
        "GCP_WIF_PROVIDER",
        "GCP_SECRETS_PROJECT",
        "GCP_SECRETS_PREFIX",
        "ACTIONS_ID_TOKEN_REQUEST_URL",
        "ACTIONS_ID_TOKEN_REQUEST_TOKEN"
    ),
    live_required_any_of("GCP_SECRETS_SA", "GCP_SECRETS_IMPERSONATE_SA")
)]
pub fn credential_lifecycle_anthropic_mock_spec() -> MockSpec {
    let response = mock::mock_anthropic_response("Credential lifecycle test response.");

    with_cloud_env(MockSpec::new("llm-credential-lifecycle-anthropic"))
        // Input mocks for DAG entry points
        .input_mock(
            "chat_completion/prepare",
            "provider",
            Value::Str("anthropic".into()),
        )
        .input_mock("resolve_auth", "provider", Value::Str("anthropic".into()))
        .input_mock(
            "chat_completion/prepare",
            "model",
            Value::Str("claude-sonnet-4-20250514".into()),
        )
        .input_mock(
            "chat_completion/prepare",
            "messages",
            Value::Json(serde_json::json!([
                {"role": "user", "content": "Credential lifecycle test"}
            ])),
        )
        // Env: resolved auth token
        // Transport mock
        .transport_mock(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                response.clone(),
            )),
        )
        .boundary(
            "chat_completion/execute",
            "response",
            Value::Response(gunbc_ir::transport::TransportResponse::Rest(response)),
        )
        .boundary(
            "chat_completion/parse",
            "content",
            Value::Str("Credential lifecycle test response.".into()),
        )
        .boundary(
            "chat_completion/parse",
            "model",
            Value::Str("claude-sonnet-4-20250514".into()),
        )
        .boundary(
            "chat_completion/parse",
            "finish_reason",
            Value::Str("Stop".into()),
        )
        .boundary("chat_completion/parse", "input_tokens", Value::Int(10))
        .boundary("chat_completion/parse", "output_tokens", Value::Int(20))
        // Live expectations: LLM returns non-empty content
        .live_expected_output(
            "chat_completion/parse",
            "content",
            OutputMatcher::non_empty(),
        )
        .live_expected_output("chat_completion/parse", "model", OutputMatcher::IsString)
        .live_expected_output(
            "chat_completion/parse",
            "finish_reason",
            OutputMatcher::IsString,
        )
        .live_expected_output(
            "chat_completion/parse",
            "input_tokens",
            OutputMatcher::IntGe(0),
        )
        .live_expected_output(
            "chat_completion/parse",
            "output_tokens",
            OutputMatcher::IntGe(0),
        )
        .expects_input("provider", InputConstraint::NonEmpty)
        .expects_input("model", InputConstraint::NonEmpty)
        // Credential resource: basic (acquire + timeout)
        .resource_credential("credential:anthropic", Some(3_600_000))
        // Credential resource: refreshable (acquire + timeout + refresh)
        .resource_credential_refreshable("credential:anthropic:refreshable", 3_600_000, 3_600_000)
        // Node I/O examples
        .node_example(
            NodeExample::new("chat_completion/prepare")
                .input("provider", Value::Str("anthropic".into()))
                .input("model", Value::Str("claude-sonnet-4-20250514".into()))
                .input("messages", Value::Str("Credential lifecycle test".into()))
                .output("request", OutputMatcher::non_empty())
                .output(
                    "provider",
                    OutputMatcher::exact(Value::Str("anthropic".into())),
                )
                .description("Credential lifecycle prepare emits REST request and echoes provider"),
        )
        .node_example(
            NodeExample::new("resolve_auth")
                .input("provider", Value::Str("anthropic".into()))
                .output(
                    "service",
                    OutputMatcher::exact(Value::Str("anthropic".into())),
                )
                .output("scheme", OutputMatcher::exact(Value::Str("header".into())))
                .output(
                    "header_name",
                    OutputMatcher::exact(Value::Str("x-api-key".into())),
                )
                .description("Resolve auth maps Anthropic provider to header auth"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("anthropic".into()))
                .input(
                    "response",
                    Value::Response(gunbc_ir::transport::TransportResponse::Rest(
                        mock::mock_anthropic_response("Credential lifecycle test response."),
                    )),
                )
                .output(
                    "content",
                    OutputMatcher::exact(Value::Str("Credential lifecycle test response.".into())),
                )
                .output(
                    "model",
                    OutputMatcher::exact(Value::Str("claude-sonnet-4-20250514".into())),
                )
                .description("Credential lifecycle parse extracts content from REST response"),
        )
        .node_example(
            NodeExample::new("chat_completion/parse")
                .input("provider", Value::Str("anthropic".into()))
                .input("response", Value::Skipped)
                .output("content", OutputMatcher::Any)
                .output("model", OutputMatcher::Any)
                .description("Parse handles skipped transport response"),
        )
        .skip_node_example("cloud_env")
        .skip_node_example("cloud_credential")
        .skip_node_example("bind_secret")
        .skip_node_example("scope_preflight")
}
