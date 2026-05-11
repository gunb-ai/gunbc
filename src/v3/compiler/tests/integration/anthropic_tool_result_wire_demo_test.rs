//! **Layer:** integration
//!
//! Hermetic JSON shape ratchet for Anthropic Messages `tool_result.content`:
//! scalar string vs JSON array of nested blocks (r3-coproduct-2). Mirrors the
//! authority modeled in `dsl/extdeps/llm/anthropic.dag` (`UntaggedVariant` on
//! `AnthropicToolResultContent` + internally tagged `AnthropicToolResultBlock`).

use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DemoUserContent {
    #[allow(dead_code)] // completeness vs Messages API user blocks; tests focus on tool_result
    Text { text: String },
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<ToolResultContentWire>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum ToolResultContentWire {
    Scalar(String),
    Blocks(Vec<ToolResultBlockWire>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ToolResultBlockWire {
    Text {
        text: String,
    },
    Image {
        source: ImageSourceWire,
    },
    Document {
        source: PlainTextDocumentSourceWire,
    },
    SearchResult {
        source: String,
        title: String,
        content: Vec<ToolResultBlockWire>,
    },
    ToolReference {
        tool_name: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum ImageSourceWire {
    #[serde(rename = "base64")]
    Base64 { media_type: String, data: String },
    #[allow(dead_code)] // parity with API; scalar demo uses base64 only
    #[serde(rename = "url")]
    Url { url: String },
}

#[derive(Debug, Clone, Serialize)]
struct PlainTextDocumentSourceWire {
    #[serde(rename = "type")]
    source_type: &'static str,
    media_type: &'static str,
    data: String,
}

#[derive(Debug, Clone, Serialize)]
struct DemoMessage {
    role: &'static str,
    content: Vec<DemoUserContent>,
}

#[test]
fn tool_result_content_serializes_scalar_string_at_json_boundary() {
    let msg = DemoMessage {
        role: "user",
        content: vec![DemoUserContent::ToolResult {
            tool_use_id: "toolu_01".to_string(),
            content: Some(ToolResultContentWire::Scalar(
                "plain scalar result".to_string(),
            )),
            is_error: None,
        }],
    };
    assert_eq!(
        serde_json::to_value(&msg).expect("serialize"),
        json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": "toolu_01",
                "content": "plain scalar result"
            }]
        })
    );
}

#[test]
fn tool_result_content_serializes_nested_block_array_without_outer_wrapper_object() {
    let nested_text = ToolResultBlockWire::Text {
        text: "nested line".to_string(),
    };
    let msg = DemoMessage {
        role: "user",
        content: vec![DemoUserContent::ToolResult {
            tool_use_id: "toolu_02".to_string(),
            content: Some(ToolResultContentWire::Blocks(vec![
                ToolResultBlockWire::Text {
                    text: "hello".to_string(),
                },
                ToolResultBlockWire::Image {
                    source: ImageSourceWire::Base64 {
                        media_type: "image/jpeg".to_string(),
                        data: "AAA=".to_string(),
                    },
                },
                ToolResultBlockWire::Document {
                    source: PlainTextDocumentSourceWire {
                        source_type: "text",
                        media_type: "text/plain",
                        data: "doc bytes".to_string(),
                    },
                },
                ToolResultBlockWire::SearchResult {
                    source: "web".to_string(),
                    title: "Example".to_string(),
                    content: vec![nested_text],
                },
                ToolResultBlockWire::ToolReference {
                    tool_name: "my_tool".to_string(),
                },
            ])),
            is_error: Some(false),
        }],
    };
    assert_eq!(
        serde_json::to_value(&msg).expect("serialize"),
        json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": "toolu_02",
                "content": [
                    {"type": "text", "text": "hello"},
                    {"type": "image", "source": {
                        "type": "base64",
                        "media_type": "image/jpeg",
                        "data": "AAA="
                    }},
                    {"type": "document", "source": {
                        "type": "text",
                        "media_type": "text/plain",
                        "data": "doc bytes"
                    }},
                    {"type": "search_result", "source": "web", "title": "Example",
                     "content": [{"type": "text", "text": "nested line"}]},
                    {"type": "tool_reference", "tool_name": "my_tool"}
                ],
                "is_error": false
            }]
        })
    );
}
