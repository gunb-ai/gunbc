//! **Layer:** integration
//!
//! T-Substrate-AnthropicSchemaMirror lockstep ratchet.
//!
//! Pins the v3 typed mirror in `src/v3/std/anthropic_schema.dag` against
//! the v2 authority in `dsl/extdeps/llm/anthropic.dag`. The mirror is
//! type authority only (no `data` rows). For every mirrored type the
//! ratchet asserts:
//!
//! - The v3 declaration resolves in the full bootstrap.
//! - **Set equality (not subset)** between the v2 field/variant set
//!   extracted directly from `dsl/extdeps/llm/anthropic.dag` and the v3
//!   field/variant set walked from the bootstrap. v2 additions surface
//!   as a missing-on-v3 diff; v3 drift surfaces as missing-on-v2.
//! - **Structural optionality**: every field marked `<label>: <T>?`
//!   in the v2 source must lower as `Cardinality(AtMostOne, _)` on the
//!   v3 mirror; every non-optional v2 field must NOT.
//!
//! The v2 file is parsed by v2 stage0, not by v3, so the ratchet works
//! by extracting type blocks from the v2 source text and comparing
//! their structural projections to the v3 bootstrap declarations.
//! Lockstep discipline mirrors `method_registry_test.rs`'s algebra-template
//! scan against `dsl/std/algebra.dag`.

use std::collections::BTreeSet;
use v3_compiler::dag::{CardinalityBound, Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

const V2_SOURCE: &str = include_str!("../../../../../dsl/extdeps/llm/anthropic.dag");

// ── v3 bootstrap projections ──────────────────────────────────────────

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn conj_field_ty(dag: &Dag, owner: &str, label: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(owner)
        .unwrap_or_else(|| panic!("`{owner}` missing from full bootstrap"));
    let children = match &decl.connective {
        TypeConnective::Conj { children } => children,
        other => panic!("`{owner}` is not a Conj: {other:?}"),
    };
    children
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("`{owner}.{label}` missing"))
        .ty
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

/// True iff the v3 declaration `owner.<field>` lowers as
/// `Cardinality(AtMostOne, _)`. The `T?` surface syntax produces this
/// connective; structural inspection of the field's referenced
/// declaration is therefore the v3-side ground truth for optionality.
fn v3_field_is_optional(dag: &Dag, owner: &str, field: &str) -> bool {
    let ty = conj_field_ty(dag, owner, field);
    match &dag.declaration(ty).connective {
        TypeConnective::Cardinality(p) => p.bound() == CardinalityBound::AtMostOne,
        _ => false,
    }
}

// ── v2 source-text extractors ─────────────────────────────────────────

/// Extract the source text of the `type <name>` block from the v2 file.
/// Block ends at the next top-level keyword on a line, or end-of-file.
fn v2_type_block(name: &str) -> &'static str {
    let opens = [
        format!("type {name} {{"),
        format!("type {name}\n"),
        format!("type {name} ="),
    ];
    let start = opens
        .iter()
        .find_map(|n| V2_SOURCE.find(n.as_str()))
        .unwrap_or_else(|| {
            panic!(
                "v2 source `dsl/extdeps/llm/anthropic.dag` no longer declares \
                 `type {name}` — lockstep with v3 mirror is broken."
            )
        });
    let after_keyword = start + "type ".len();
    let rest = &V2_SOURCE[after_keyword..];
    let stop = ["\ntype ", "\ndata ", "\nservice ", "\nfn ", "\nmodule "];
    let end_in_rest = stop
        .iter()
        .filter_map(|kw| rest.find(kw))
        .min()
        .unwrap_or(rest.len());
    &V2_SOURCE[start..(after_keyword + end_in_rest)]
}

/// Parse a v2 record block (`type X { foo: T, bar: U?, … }`) and return
/// `(field_label, is_optional)` tuples in declaration order. Optionality
/// is read from the trailing `?` on the type expression.
///
/// The parser handles fields written one-per-line or comma-separated on
/// one line. It strips trailing commas/whitespace, picks the head token
/// before `:` as the label, and checks whether the type expression
/// (between `:` and the next `,` / line-end / `}`) ends in `?` (after
/// trimming whitespace).
fn v2_record_fields(name: &str) -> Vec<(String, bool)> {
    let block = v2_type_block(name);
    let body_start = block.find('{').unwrap_or_else(|| {
        panic!("v2 `type {name}` is not a record block (no `{{` in extracted text)")
    }) + 1;
    let body_end = block
        .rfind('}')
        .unwrap_or_else(|| panic!("v2 `type {name}` record block has no closing `}}`"));
    let body = &block[body_start..body_end];

    let mut out = Vec::new();
    // Split on either newline or comma; each piece is one field (or
    // empty/whitespace).
    for piece in body.split(|c: char| c == '\n' || c == ',') {
        let trimmed = piece.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let Some(colon) = trimmed.find(':') else {
            // Skip non-field lines (e.g. inline comments without `//`).
            continue;
        };
        let label = trimmed[..colon].trim().to_string();
        if label.is_empty()
            || !label
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            continue;
        }
        let ty_text_raw = trimmed[colon + 1..].trim();
        // Strip a trailing `//` line comment if any.
        let ty_text = match ty_text_raw.find("//") {
            Some(idx) => ty_text_raw[..idx].trim(),
            None => ty_text_raw,
        };
        // Optionality = type expression ends with `?`.
        let is_optional = ty_text.ends_with('?');
        out.push((label, is_optional));
    }
    out
}

/// Parse a v2 disj block and return variant label names in declaration
/// order. Handles both inline (`type X = A | B | C`) and multi-line
/// (`type X\n  = Foo { ... }\n  | Bar { ... }`) forms.
fn v2_disj_variants(name: &str) -> Vec<String> {
    let block = v2_type_block(name);
    let eq = block.find('=').unwrap_or_else(|| {
        panic!("v2 `type {name}` is not a disj block (no `=` in extracted text)")
    });
    let after_eq = &block[eq + 1..];
    // Strip block-level comments so `|` inside `//` lines doesn't fool us.
    // Cheap approach: drop line comments by walking lines.
    let mut cleaned = String::with_capacity(after_eq.len());
    for line in after_eq.lines() {
        let line_no_comment = match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        };
        cleaned.push_str(line_no_comment);
        cleaned.push('\n');
    }
    let mut out = Vec::new();
    for piece in cleaned.split('|') {
        let mut chunk = piece.trim().to_string();
        // Drop optional `{ … }` payload on the variant.
        if let Some(brace) = chunk.find('{') {
            chunk.truncate(brace);
        }
        let label = chunk.trim().to_string();
        if label.is_empty() {
            continue;
        }
        // The label must be a bare identifier — anything with `=`,
        // whitespace inside, etc. is malformed and we error out so a
        // future v2 syntax change doesn't silently mask drift.
        let Some(first) = label.chars().next() else {
            continue;
        };
        if !(first.is_alphabetic() || first == '_') {
            continue;
        }
        if !label.chars().all(|c| c.is_alphanumeric() || c == '_') {
            panic!(
                "v2 `type {name}` variant fragment `{label}` is not a bare identifier — \
                 lockstep parser needs an update for new v2 syntax"
            );
        }
        out.push(label);
    }
    out
}

// ── Lockstep assertions ───────────────────────────────────────────────

fn assert_record_lockstep(type_name: &str) {
    let dag = generated_full_bootstrap_dag();
    let v3_labels: BTreeSet<String> = conj_field_labels(&dag, type_name).into_iter().collect();
    let v2_fields = v2_record_fields(type_name);
    let v2_labels: BTreeSet<String> = v2_fields.iter().map(|(l, _)| l.clone()).collect();
    assert_eq!(
        v3_labels,
        v2_labels,
        "lockstep drift on `type {type_name}` field set: v3 mirror \
         (`src/v3/std/anthropic_schema.dag`) and v2 source \
         (`dsl/extdeps/llm/anthropic.dag`) disagree. \
         v3-only: {:?}; v2-only: {:?}",
        v3_labels.difference(&v2_labels).collect::<Vec<_>>(),
        v2_labels.difference(&v3_labels).collect::<Vec<_>>()
    );
    // Structural optionality: each v2-`T?` field must lower as
    // `Cardinality(AtMostOne, _)` on the v3 mirror, and each v2-`T`
    // field must NOT.
    for (label, v2_optional) in &v2_fields {
        let v3_optional = v3_field_is_optional(&dag, type_name, label);
        assert_eq!(
            v3_optional,
            *v2_optional,
            "lockstep optionality drift on `type {type_name}.{label}`: \
             v2 source marks it {} but v3 mirror lowers as {}.",
            if *v2_optional {
                "optional (`T?`)"
            } else {
                "required (`T`)"
            },
            if v3_optional {
                "Cardinality(AtMostOne, _)"
            } else {
                "non-optional"
            }
        );
    }
}

fn assert_disj_lockstep(type_name: &str) {
    let dag = generated_full_bootstrap_dag();
    let v3_labels: BTreeSet<String> = disj_variant_labels(&dag, type_name).into_iter().collect();
    let v2_labels: BTreeSet<String> = v2_disj_variants(type_name).into_iter().collect();
    assert_eq!(
        v3_labels,
        v2_labels,
        "lockstep drift on `type {type_name}` variant set: v3 mirror and \
         v2 source disagree. v3-only: {:?}; v2-only: {:?}",
        v3_labels.difference(&v2_labels).collect::<Vec<_>>(),
        v2_labels.difference(&v3_labels).collect::<Vec<_>>()
    );
}

// ── Tests ─────────────────────────────────────────────────────────────

#[test]
fn anthropic_chat_message_lockstep() {
    assert_disj_lockstep("AnthropicChatMessage");
}

#[test]
fn anthropic_user_content_block_lockstep() {
    assert_disj_lockstep("AnthropicUserContentBlock");
}

#[test]
fn anthropic_assistant_content_block_lockstep() {
    assert_disj_lockstep("AnthropicAssistantContentBlock");
}

#[test]
fn anthropic_stop_reason_lockstep() {
    assert_disj_lockstep("AnthropicStopReason");
}

#[test]
fn anthropic_messages_200_text_block_lockstep() {
    assert_record_lockstep("AnthropicMessages200TextBlock");
}

#[test]
fn anthropic_messages_200_usage_lockstep() {
    assert_record_lockstep("AnthropicMessages200Usage");
}

#[test]
fn anthropic_messages_200_body_lockstep() {
    assert_record_lockstep("AnthropicMessages200Body");
}

#[test]
fn anthropic_user_content_block_user_tool_result_block_optionality_lockstep() {
    // `UserToolResultBlock` is a variant payload, so the v2-record
    // extraction above does not reach it. Probe the v2 block textually
    // for `is_error: Bool?` and additionally walk the v3 variant payload
    // to confirm `is_error` lowers optionally on the variant declaration.
    let v2_block = v2_type_block("AnthropicUserContentBlock");
    assert!(
        v2_block.contains("is_error: Bool?"),
        "v2 source dropped `is_error: Bool?` from \
         `AnthropicUserContentBlock::UserToolResultBlock`."
    );
    let dag = generated_full_bootstrap_dag();
    // The variant payload is itself a Conj declaration in the lowered
    // bootstrap; look it up by the variant's anonymous span. Concretely
    // the variant points at a Conj; we can locate it via the parent
    // disj's variant target id.
    let parent = dag
        .declaration_by_name("AnthropicUserContentBlock")
        .expect("AnthropicUserContentBlock missing from bootstrap");
    let TypeConnective::Disj { variants } = &parent.connective else {
        panic!("AnthropicUserContentBlock is not a Disj");
    };
    let variant_target = variants
        .iter()
        .find(|v| v.label == "UserToolResultBlock")
        .expect("UserToolResultBlock variant missing")
        .ty;
    let payload = dag.declaration(variant_target);
    let TypeConnective::Conj { children } = &payload.connective else {
        panic!("UserToolResultBlock payload is not a Conj");
    };
    let is_error = children
        .iter()
        .find(|f| f.label == "is_error")
        .expect("UserToolResultBlock.is_error field missing");
    let is_error_decl = dag.declaration(is_error.ty);
    let optional = matches!(
        &is_error_decl.connective,
        TypeConnective::Cardinality(p) if p.bound() == CardinalityBound::AtMostOne
    );
    assert!(
        optional,
        "v3 mirror `AnthropicUserContentBlock::UserToolResultBlock.is_error` \
         must lower as `Cardinality(AtMostOne, Bool)` to mirror v2 `Bool?`."
    );
}

#[test]
fn anthropic_schema_authors_no_data_rows() {
    // Type authority only in this PR.
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
