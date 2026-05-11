//! **Layer:** integration
//!
//! Hermetic JSON examples for Anthropic Messages `tool_result.content`
//! (scalar string vs nested block array). Every serde shape below is
//! **ratcheted** against [`generated_full_bootstrap_dag`] in
//! `tool_result_wire_demo_projection_is_locked_to_modeled_dag_surface`
//! (same discipline as `anthropic_messages_wire_demo_test.rs`), so the
//! demo cannot drift while `src/v3/std/anthropic_schema.dag` /
//! `dsl/extdeps/llm/anthropic.dag` stay lockstep via
//! `anthropic_schema_lockstep_test.rs`.
//!
//! Authority: `dsl/extdeps/llm/anthropic.dag` (`UntaggedVariant` on
//! `AnthropicToolResultContent`, internally tagged `AnthropicToolResultBlock`
//! with `StripPrefixSuffixAndSnakeCase { prefix: \"Anthropic\", suffix: \"Block\" }`).

use serde::Serialize;
use serde_json::json;
use v3_compiler::dag::{CardinalityBound, Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DemoUserContent {
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
        /// Matches bootstrap `List<AnthropicToolResultTextBlock>` — only `text`
        /// rows are schema-valid nested entries for this arm.
        content: Vec<ToolResultSearchNestedTextWire>,
        /// Mirrors `AnthropicSearchResultBlock.citations?` (`AnthropicSearchResultCitationsConfig`).
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<SearchResultCitationsWire>,
        /// Mirrors `AnthropicSearchResultBlock.cache_control?` (`CacheControl` on v2 authority).
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControlWire>,
    },
    ToolReference {
        tool_name: String,
    },
}

/// Nested `search_result.content[]` rows: DAG `AnthropicToolResultTextBlock` carries wire
/// `type` + `text` (same JSON shape as this internally-tagged enum).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ToolResultSearchNestedTextWire {
    Text { text: String },
}

/// Mirrors `AnthropicSearchResultCitationsConfig` on the modeled `search_result` row.
#[derive(Debug, Clone, Serialize)]
struct SearchResultCitationsWire {
    enabled: bool,
}

/// Mirrors `CacheControl` (`dsl/extdeps/llm/anthropic.dag`) — wire `type` key (e.g. `"ephemeral"`).
#[derive(Debug, Clone, Serialize)]
struct CacheControlWire {
    #[serde(rename = "type")]
    wire_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum ImageSourceWire {
    #[serde(rename = "base64")]
    Base64 {
        media_type: String,
        /// JSON key `data` matches Anthropic wire for base64 sources (`llm.dag`).
        data: String,
    },
}

#[derive(Debug, Clone, Serialize)]
struct PlainTextDocumentSourceWire {
    #[serde(rename = "type")]
    source_type: &'static str,
    media_type: &'static str,
    /// Wire field name `data` (`AnthropicToolResultPlainTextDocumentSource` in v2).
    data: String,
}

#[derive(Debug, Clone, Serialize)]
struct DemoMessage {
    role: &'static str,
    content: Vec<DemoUserContent>,
}

// ── Bootstrap projections (aligned with anthropic_messages_wire_demo_test) ──

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

fn v3_canonical_ty(dag: &Dag, ty: DeclarationId) -> String {
    let decl = dag.declaration(ty);
    if let Some(name) = decl.name.as_deref() {
        return name.to_string();
    }
    match &decl.connective {
        TypeConnective::Cardinality(p) if p.bound() == CardinalityBound::AtMostOne => {
            format!("{}?", v3_canonical_ty(dag, p.element()))
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let template_name = dag
                .declaration(*template)
                .name
                .clone()
                .unwrap_or_else(|| v3_canonical_ty(dag, *template));
            if arguments.is_empty() {
                template_name
            } else {
                let args: Vec<String> = arguments
                    .iter()
                    .map(|a| v3_canonical_ty(dag, a.value))
                    .collect();
                format!("{}<{}>", template_name, args.join(", "))
            }
        }
        _ => "<anon>".to_string(),
    }
}

type V3PayloadField = (String, String, bool);

fn v3_variant_payload_fields(
    dag: &Dag,
    parent_name: &str,
    variant_label: &str,
) -> Option<Vec<V3PayloadField>> {
    let parent = dag
        .declaration_by_name(parent_name)
        .unwrap_or_else(|| panic!("`{parent_name}` missing from full bootstrap"));
    let variants = match &parent.connective {
        TypeConnective::Disj { variants } => variants,
        other => panic!("`{parent_name}` is not a Disj: {other:?}"),
    };
    let target = variants
        .iter()
        .find(|v| v.label == variant_label)
        .unwrap_or_else(|| panic!("`{parent_name}::{variant_label}` variant missing"))
        .ty;
    let payload = dag.declaration(target);
    match &payload.connective {
        TypeConnective::Conj { children } if !children.is_empty() => Some(
            children
                .iter()
                .map(|f| {
                    let optional = matches!(
                        &dag.declaration(f.ty).connective,
                        TypeConnective::Cardinality(p)
                            if p.bound() == CardinalityBound::AtMostOne
                    );
                    let raw_canonical = v3_canonical_ty(dag, f.ty);
                    let ty_canonical = raw_canonical
                        .strip_suffix('?')
                        .map(|s| s.trim_end().to_string())
                        .unwrap_or(raw_canonical);
                    (f.label.clone(), ty_canonical, optional)
                })
                .collect(),
        ),
        _ => None,
    }
}

/// Mirrors `anthropic_tool_result_block_wire_contract` strip prefix/suffix + snake_case.
fn anthropic_tool_result_block_wire_tag(variant_label: &str) -> String {
    const PREFIX: &str = "Anthropic";
    const SUFFIX: &str = "Block";
    let mid = variant_label
        .strip_prefix(PREFIX)
        .unwrap_or(variant_label)
        .strip_suffix(SUFFIX)
        .unwrap_or(variant_label);
    pascal_chunk_to_snake(mid)
}

fn pascal_chunk_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn tool_result_wire_demo_projection_is_locked_to_modeled_dag_surface() {
    let dag = generated_full_bootstrap_dag();

    assert_eq!(
        disj_variant_labels(&dag, "AnthropicToolResultContent"),
        vec!["ToolResultText", "ToolResultBlocks"],
        "UntaggedVariant serde demo arms must track AnthropicToolResultContent"
    );
    let tool_result_text =
        v3_variant_payload_fields(&dag, "AnthropicToolResultContent", "ToolResultText")
            .expect("ToolResultText tuple/record payload");
    assert_eq!(
        tool_result_text,
        vec![("_0".to_string(), "String".to_string(), false)],
        "ToolResultText(String) lowers as positional _0 on the v3 mirror"
    );
    let tool_result_blocks =
        v3_variant_payload_fields(&dag, "AnthropicToolResultContent", "ToolResultBlocks")
            .expect("ToolResultBlocks payload");
    assert_eq!(
        tool_result_blocks,
        vec![(
            "_0".to_string(),
            "List<AnthropicToolResultBlock>".to_string(),
            false
        )],
        "ToolResultBlocks(List<…>) lowers as positional _0"
    );

    let block_variants = disj_variant_labels(&dag, "AnthropicToolResultBlock");
    let expected_wire_tags = [
        "text",
        "image",
        "document",
        "search_result",
        "tool_reference",
    ];
    assert_eq!(
        block_variants.len(),
        expected_wire_tags.len(),
        "demo serde arms vs modeled AnthropicToolResultBlock variant count"
    );
    for (variant, expected_tag) in block_variants.iter().zip(expected_wire_tags.iter()) {
        assert_eq!(
            anthropic_tool_result_block_wire_tag(variant),
            *expected_tag,
            "InternallyTaggedObject naming for `{variant}` must serialize as `{expected_tag}`"
        );
    }

    let user_tool =
        v3_variant_payload_fields(&dag, "AnthropicUserContentBlock", "UserToolResultBlock")
            .expect("UserToolResultBlock payload");
    let labels: Vec<&str> = user_tool.iter().map(|(l, _, _)| l.as_str()).collect();
    assert_eq!(
        labels,
        vec!["tool_use_id", "content", "is_error"],
        "demo ToolResult fields must match UserToolResultBlock conj order/labels"
    );

    let plain_fields = conj_field_labels(&dag, "AnthropicToolResultPlainTextDocumentSource");
    assert_eq!(
        plain_fields,
        vec!["type", "media_type", "document_data"],
        "document source demo JSON keys map wire `data` ↔ mirror `document_data` (see lockstep)"
    );

    let search_payload = v3_variant_payload_fields(
        &dag,
        "AnthropicToolResultBlock",
        "AnthropicSearchResultBlock",
    )
    .expect("AnthropicSearchResultBlock record payload");
    let (_, content_ty, content_optional) = search_payload
        .iter()
        .find(|(l, _, _)| l == "content")
        .expect("search_result.content field");
    assert_eq!(content_ty, "List<AnthropicToolResultTextBlock>");
    assert!(
        !content_optional,
        "search nested content list is required on the wire row"
    );
    let (_, citations_ty, citations_optional) = search_payload
        .iter()
        .find(|(l, _, _)| l == "citations")
        .expect("search_result.citations field");
    assert_eq!(citations_ty, "AnthropicSearchResultCitationsConfig");
    assert!(
        citations_optional,
        "citations is optional on AnthropicSearchResultBlock per v2 authority"
    );
    let (_, cache_ty, cache_optional) = search_payload
        .iter()
        .find(|(l, _, _)| l == "cache_control")
        .expect("search_result.cache_control field");
    assert_eq!(cache_ty, "CacheControl");
    assert!(
        cache_optional,
        "cache_control is optional on AnthropicSearchResultBlock per v2 authority"
    );
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
    let nested_text = ToolResultSearchNestedTextWire::Text {
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
                    citations: Some(SearchResultCitationsWire { enabled: true }),
                    cache_control: Some(CacheControlWire {
                        wire_type: "ephemeral".to_string(),
                    }),
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
                     "content": [{"type": "text", "text": "nested line"}],
                     "citations": {"enabled": true},
                     "cache_control": {"type": "ephemeral"}},
                    {"type": "tool_reference", "tool_name": "my_tool"}
                ],
                "is_error": false
            }]
        })
    );
}
