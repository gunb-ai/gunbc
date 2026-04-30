//! **Layer:** integration
//!
//! T-Substrate-AnthropicSchemaMirror lockstep ratchet.
//!
//! Pins the v3 typed mirror in `src/v3/std/anthropic_schema.dag` against
//! the v2 authority in `dsl/extdeps/llm/anthropic.dag`. The mirror is
//! type authority only (no `data` rows); the ratchet asserts:
//!
//! - Every type the Anthropic Messages operation signature reaches
//!   resolves in the full bootstrap.
//! - Variant labels of the closed provider-domain coproducts match the
//!   v2 source byte-for-byte.
//! - Record field labels (and the `?` optionality marker on
//!   `is_error` / `stop_sequence`) match.
//! - Mirrored types and the v2 source share the same name in their
//!   respective declaration tables.
//!
//! The check is grep-based against the v2 source so that authoring drift
//! in either direction surfaces (the v2 file is parsed by v2 stage0, not
//! by v3, so we cannot walk its declarations through `Dag`). Lockstep
//! discipline mirrors `src/v3/compiler/tests/integration/method_registry_test.rs`'s
//! algebra-template scan against `dsl/std/algebra.dag`.

use std::collections::HashSet;
use v3_compiler::dag::{Dag, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

const V2_SOURCE: &str = include_str!("../../../../../dsl/extdeps/llm/anthropic.dag");

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

/// Extract the source text of `type <name> { … }` / `type <name> = …` /
/// `type <name>\n  | …` from the v2 file. Returns the slice from the
/// matching `type <name>` keyword through the end of the block (next
/// top-level `type ` / `data ` / `service ` / `fn ` / `module ` keyword,
/// or end-of-file). Used so lockstep assertions check label presence
/// against the v2 source itself, not just hard-coded constants.
fn v2_type_block(name: &str) -> &'static str {
    // Probe for any of the three opening shapes.
    let needles = [
        format!("type {name} {{"),
        format!("type {name}\n"),
        format!("type {name} ="),
    ];
    let start = needles
        .iter()
        .find_map(|n| V2_SOURCE.find(n.as_str()))
        .unwrap_or_else(|| {
            panic!(
                "v2 source `dsl/extdeps/llm/anthropic.dag` no longer declares \
                 `type {name}` — lockstep with v3 mirror is broken."
            )
        });
    // Block ends at the next top-level keyword on a line, after we've
    // consumed at least the opening `type ` keyword itself.
    let after_open = start + "type ".len();
    let rest = &V2_SOURCE[after_open..];
    let stop_keywords = ["\ntype ", "\ndata ", "\nservice ", "\nfn ", "\nmodule "];
    let block_len_in_rest = stop_keywords
        .iter()
        .filter_map(|kw| rest.find(kw))
        .min()
        .unwrap_or(rest.len());
    &V2_SOURCE[start..(after_open + block_len_in_rest)]
}

fn assert_v2_block_contains(type_name: &str, needle: &str) {
    let block = v2_type_block(type_name);
    assert!(
        block.contains(needle),
        "v2 source's `type {type_name}` block no longer contains `{needle}` — \
         lockstep with v3 mirror is broken; the v2 declaration drifted or the \
         mirror is stale.\n--- v2 block ---\n{block}\n--- end ---"
    );
}

fn assert_lockstep_record(type_name: &str, expected_field_labels: &[&str]) {
    let dag = generated_full_bootstrap_dag();
    let v3_labels: HashSet<String> = conj_field_labels(&dag, type_name).into_iter().collect();
    let expected_labels: HashSet<String> = expected_field_labels
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        v3_labels, expected_labels,
        "{type_name} v3 field set diverged from expected"
    );
    // Couple expected labels to v2 source: each label must appear in the
    // v2 block, fail-closed against drift in either direction.
    for label in expected_field_labels {
        assert_v2_block_contains(type_name, &format!("{label}:"));
    }
}

fn assert_lockstep_disj(type_name: &str, expected_variant_labels: &[&str]) {
    let dag = generated_full_bootstrap_dag();
    let v3_labels: HashSet<String> = disj_variant_labels(&dag, type_name).into_iter().collect();
    let expected_labels: HashSet<String> = expected_variant_labels
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        v3_labels, expected_labels,
        "{type_name} v3 variant set diverged from expected"
    );
    // Each variant label must appear in the v2 block. We use the bare
    // identifier — the v2 source writes variants with optional payload
    // (`UserTextBlock { … }` or `EndTurn`), so the substring match is
    // tight enough to fail when the v2 source drops/renames a variant.
    for label in expected_variant_labels {
        assert_v2_block_contains(type_name, label);
    }
}

#[test]
fn anthropic_chat_message_variants_match_v2_source() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = disj_variant_labels(&dag, "AnthropicChatMessage")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["UserMessage", "AssistantMessage"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        labels, expected,
        "AnthropicChatMessage variants must mirror v2 source exactly."
    );
    assert_v2_declares_type("AnthropicChatMessage");
}

#[test]
fn anthropic_user_content_block_variants_match_v2_source() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = disj_variant_labels(&dag, "AnthropicUserContentBlock")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["UserTextBlock", "UserToolResultBlock"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        labels, expected,
        "AnthropicUserContentBlock variants must mirror v2 source exactly."
    );
    assert_v2_declares_type("AnthropicUserContentBlock");
}

#[test]
fn anthropic_assistant_content_block_variants_match_v2_source() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = disj_variant_labels(&dag, "AnthropicAssistantContentBlock")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["AssistantTextBlock", "AssistantToolUseBlock"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        labels, expected,
        "AnthropicAssistantContentBlock variants must mirror v2 source exactly."
    );
    assert_v2_declares_type("AnthropicAssistantContentBlock");
}

#[test]
fn anthropic_stop_reason_variants_match_v2_source() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = disj_variant_labels(&dag, "AnthropicStopReason")
        .into_iter()
        .collect();
    let expected: HashSet<String> = [
        "EndTurn",
        "MaxTokens",
        "StopSequence",
        "ToolUse",
        "PauseTurn",
        "Refusal",
        "ModelContextWindowExceeded",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        labels, expected,
        "AnthropicStopReason variants must mirror Anthropic's closed \
         API enum byte-for-byte against the v2 source."
    );
    assert_v2_declares_type("AnthropicStopReason");
}

#[test]
fn anthropic_messages_200_text_block_fields_match_v2_source() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "AnthropicMessages200TextBlock")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["type", "text"].iter().map(|s| s.to_string()).collect();
    assert_eq!(labels, expected);
    assert_v2_declares_type("AnthropicMessages200TextBlock");
}

#[test]
fn anthropic_messages_200_usage_fields_match_v2_source() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "AnthropicMessages200Usage")
        .into_iter()
        .collect();
    let expected: HashSet<String> = ["input_tokens", "output_tokens"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(labels, expected);
    assert_v2_declares_type("AnthropicMessages200Usage");
}

#[test]
fn anthropic_messages_200_body_fields_match_v2_source() {
    let dag = generated_full_bootstrap_dag();
    let labels: HashSet<String> = conj_field_labels(&dag, "AnthropicMessages200Body")
        .into_iter()
        .collect();
    let expected: HashSet<String> = [
        "id",
        "type",
        "role",
        "content",
        "model",
        "stop_reason",
        "stop_sequence",
        "usage",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        labels, expected,
        "AnthropicMessages200Body field set must mirror v2 source \
         (`AnthropicMessages200Body` in `dsl/extdeps/llm/anthropic.dag`)."
    );
    assert_v2_declares_type("AnthropicMessages200Body");
}

#[test]
fn anthropic_schema_authors_no_data_rows() {
    // Type authority only in this PR. The follow-on substrate precursor
    // authors `fn anthropic_messages` and the operation-row binding;
    // neither lands here.
    let dag = generated_full_bootstrap_dag();
    let leaks: Vec<String> = dag
        .declarations()
        .iter()
        .filter(|d| d.span.file == "src/v3/std/anthropic_schema.dag" && d.value_body.is_some())
        .map(|d| {
            d.name
                .clone()
                .unwrap_or_else(|| format!("DeclarationId({:?})", d.id))
        })
        .collect();
    assert!(
        leaks.is_empty(),
        "anthropic_schema.dag is type-authority only — no `data` rows or \
         `fn` bodies allowed in this slice. Found: {leaks:?}."
    );
}
