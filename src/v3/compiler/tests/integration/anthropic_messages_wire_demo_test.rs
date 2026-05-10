//! **Layer:** integration
//!
//! R3 gate #68 (`anthropic_wire_demonstration`): hermetic Anthropic
//! Messages request/response cycle against a deterministic mock.
//!
//! This deliberately does not call the live Anthropic API. The gate is
//! the typed wire cycle: construct a request with the same field surface
//! as `fn anthropic_messages`, serialize it to the REST body shape from
//! `dsl/extdeps/llm/anthropic.dag`, feed a deterministic mock 200 body
//! through the typed response mirror, and assert the projected result.

use serde::{Deserialize, Serialize};
use serde_json::json;
use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RequestRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RequestContentBlock {
    Text { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RequestMessage {
    role: RequestRole,
    content: Vec<RequestContentBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct AnthropicMessagesRequest {
    model: String,
    messages: Vec<RequestMessage>,
    max_tokens: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResponseRole {
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    PauseTurn,
    Refusal,
    ModelContextWindowExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseContentBlock {
    Text { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Usage {
    input_tokens: i64,
    output_tokens: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct AnthropicMessages200Body {
    id: String,
    #[serde(rename = "type")]
    body_type: String,
    role: ResponseRole,
    content: Vec<ResponseContentBlock>,
    model: String,
    stop_reason: StopReason,
    stop_sequence: Option<String>,
    usage: Usage,
}

fn canonical_ty(dag: &Dag, ty: DeclarationId) -> String {
    let decl = dag.declaration(ty);
    if let Some(name) = decl.name.as_deref() {
        return name.to_string();
    }
    match &decl.connective {
        TypeConnective::Cardinality(p) => format!("{}?", canonical_ty(dag, p.element())),
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let template_name = dag
                .declaration(*template)
                .name
                .clone()
                .unwrap_or_else(|| canonical_ty(dag, *template));
            if arguments.is_empty() {
                template_name
            } else {
                let args: Vec<String> = arguments
                    .iter()
                    .map(|a| canonical_ty(dag, a.value))
                    .collect();
                format!("{}<{}>", template_name, args.join(", "))
            }
        }
        _ => "<anon>".to_string(),
    }
}

fn anthropic_messages_signature(dag: &Dag) -> (Vec<String>, String) {
    let decl = dag
        .declaration_by_name("anthropic_messages")
        .expect("`anthropic_messages` missing from full bootstrap");
    let TypeConnective::Arrow { inputs, output, .. } = &decl.connective else {
        panic!(
            "`anthropic_messages` must be Arrow-shaped; got {:?}",
            decl.connective
        );
    };
    (
        inputs.iter().map(|id| canonical_ty(dag, *id)).collect(),
        canonical_ty(dag, *output),
    )
}

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn disj_variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("`{name}` is not a Disj: {other:?}"),
    }
}

#[test]
fn anthropic_messages_demo_projection_is_locked_to_modeled_dag_surface() {
    let dag = generated_full_bootstrap_dag();

    let body_fields = conj_field_labels(&dag, "AnthropicMessages200Body");
    for field in [
        "id",
        "type",
        "role",
        "content",
        "model",
        "stop_reason",
        "stop_sequence",
        "usage",
    ] {
        assert!(
            body_fields.iter().any(|actual| actual == field),
            "demo response projection field `{field}` must exist on AnthropicMessages200Body"
        );
    }

    let usage_fields = conj_field_labels(&dag, "AnthropicMessages200Usage");
    for field in ["input_tokens", "output_tokens"] {
        assert!(
            usage_fields.iter().any(|actual| actual == field),
            "demo usage projection field `{field}` must exist on AnthropicMessages200Usage"
        );
    }

    assert_eq!(
        disj_variant_labels(&dag, "AnthropicMessages200Role"),
        vec!["Assistant"],
        "demo response role enum must stay locked to the modeled response role"
    );
    assert_eq!(
        disj_variant_labels(&dag, "AnthropicStopReason"),
        vec![
            "EndTurn",
            "MaxTokens",
            "StopSequence",
            "ToolUse",
            "PauseTurn",
            "Refusal",
            "ModelContextWindowExceeded",
        ],
        "demo stop-reason enum must stay locked to AnthropicStopReason"
    );
    assert!(
        disj_variant_labels(&dag, "AnthropicMessages200ContentBlock")
            .iter()
            .any(|variant| variant == "MessagesTextBlock"),
        "demo text response block must be backed by the modeled MessagesTextBlock variant"
    );
}

#[test]
fn anthropic_messages_typed_wire_cycle_executes_against_deterministic_mock() {
    let dag = generated_full_bootstrap_dag();
    let (inputs, output) = anthropic_messages_signature(&dag);
    assert_eq!(
        inputs,
        vec![
            "Secret",
            "String",
            "List<AnthropicChatMessage>",
            "Int",
            "Float?",
            "String?",
        ],
        "demo request must stay aligned to the typed anthropic_messages input surface"
    );
    assert_eq!(
        output, "AnthropicMessages200Body",
        "demo response must stay aligned to the typed 200 response mirror"
    );

    let request = AnthropicMessagesRequest {
        model: "claude-3-5-sonnet-latest".to_string(),
        messages: vec![RequestMessage {
            role: RequestRole::User,
            content: vec![RequestContentBlock::Text {
                text: "Reply with the deterministic token.".to_string(),
            }],
        }],
        max_tokens: 16,
        temperature: Some(0.0),
        system: Some("Use the mock transcript exactly.".to_string()),
    };

    let body = serde_json::to_value(&request).expect("request serializes");
    assert_eq!(
        body,
        json!({
            "model": "claude-3-5-sonnet-latest",
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": "Reply with the deterministic token."
                }]
            }],
            "max_tokens": 16,
            "temperature": 0.0,
            "system": "Use the mock transcript exactly."
        }),
        "request body must match the Anthropic Messages REST body shape"
    );

    let mock_response = json!({
        "id": "msg_mock_01",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": "deterministic-token"
        }],
        "model": "claude-3-5-sonnet-latest",
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 11,
            "output_tokens": 3,
            "cache_creation_input_tokens": null,
            "cache_read_input_tokens": null,
            "service_tier": null
        },
        "container": null
    });

    let typed: AnthropicMessages200Body =
        serde_json::from_value(mock_response).expect("mock 200 body deserializes");
    assert_eq!(typed.role, ResponseRole::Assistant);
    assert_eq!(typed.stop_reason, StopReason::EndTurn);
    assert_eq!(typed.usage.input_tokens, 11);
    assert_eq!(typed.usage.output_tokens, 3);
    assert_eq!(
        typed.content,
        vec![ResponseContentBlock::Text {
            text: "deterministic-token".to_string()
        }],
        "typed response projection should recover the deterministic mock text"
    );
}

#[test]
fn anthropic_messages_wire_demo_is_hermetic() {
    assert_eq!(
        serde_json::to_value(RequestRole::Assistant).expect("assistant role serializes"),
        json!("assistant"),
        "assistant request role must serialize to the Anthropic wire string"
    );

    let request = AnthropicMessagesRequest {
        model: "claude-3-5-sonnet-latest".to_string(),
        messages: vec![RequestMessage {
            role: RequestRole::User,
            content: vec![RequestContentBlock::Text {
                text: "No network is used.".to_string(),
            }],
        }],
        max_tokens: 8,
        temperature: None,
        system: None,
    };

    let body = serde_json::to_value(&request).expect("request serializes");
    assert!(
        body.get("api_key").is_none(),
        "API credentials must not enter the JSON body; auth stays outside the deterministic mock"
    );
    assert!(
        body.get("temperature").is_none() && body.get("system").is_none(),
        "optional request fields should be omitted when absent"
    );
}
