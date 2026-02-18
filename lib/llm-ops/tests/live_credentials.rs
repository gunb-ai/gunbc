use gunbc_exec::{execute_with_mode_and_inputs, BoundaryMocks, ExecutionMode};
use gunbc_ir::transport::cloud::CloudProviderKind;
use gunbc_ir::Value;
use gunbc_lib_cloud_ops::detect_cloud_env_requirements;
use gunbc_lib_llm_ops::graph::build_chat_completion_graph;
use gunbc_test::{guard_test_with_env, FermiCost, TestClass};

fn input_mocks(provider: &str, model: &str) -> BoundaryMocks {
    let mut mocks = BoundaryMocks::new();
    mocks.set_input("prepare", "provider", Value::Str(provider.to_string()));
    mocks.set_input("prepare", "model", Value::Str(model.to_string()));
    mocks.set_input(
        "prepare",
        "messages",
        Value::Json(serde_json::json!([
            {"role": "user", "content": "ping"}
        ])),
    );
    // Keep responses small/cheap.
    mocks.set_input("prepare", "max_tokens", Value::Int(16));
    mocks
}

fn run_live_chat(name: &str, provider: &str, model: &str) {
    let env_req = detect_cloud_env_requirements();
    if env_req.provider != CloudProviderKind::Gcp {
        return;
    }
    if !guard_test_with_env(
        name,
        TestClass::Integration,
        FermiCost::M,
        &["http"],
        env_req.required,
        env_req.required_any_of,
    ) {
        return;
    }

    let dag = build_chat_completion_graph().unwrap();
    let inputs = input_mocks(provider, model);

    let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&inputs))
        .expect("live LLM execution should succeed");

    let parse = log.get("parse").expect("parse node should run");
    let content = parse
        .outputs
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        !content.trim().is_empty(),
        "expected non-empty content from live response"
    );

    let model_out = parse
        .outputs
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        !model_out.trim().is_empty(),
        "expected non-empty model in live response"
    );
}

#[test]
fn test_openai_live_chat_completion() {
    run_live_chat("test_openai_live_chat_completion", "openai", "gpt-4o");
}

#[test]
fn test_anthropic_live_chat_completion() {
    run_live_chat(
        "test_anthropic_live_chat_completion",
        "anthropic",
        "claude-sonnet-4-20250514",
    );
}
