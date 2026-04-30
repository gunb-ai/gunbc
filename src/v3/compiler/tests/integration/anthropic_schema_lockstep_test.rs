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
    for piece in body.split(['\n', ',']) {
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

/// Parse a v2 variant payload body (`{ foo: T, bar: U? }`) and return
/// `(field_label, is_optional)` tuples, mirroring `v2_record_fields`'s
/// extraction logic. Used by the disj lockstep to compare per-variant
/// payload field sets and optionality.
fn parse_v2_brace_body_fields(body: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for piece in body.split(['\n', ',']) {
        let trimmed = piece.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let Some(colon) = trimmed.find(':') else {
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
        let ty_text = match ty_text_raw.find("//") {
            Some(idx) => ty_text_raw[..idx].trim(),
            None => ty_text_raw,
        };
        let is_optional = ty_text.ends_with('?');
        out.push((label, is_optional));
    }
    out
}

/// Parse a v2 disj block and return per-variant `(name, payload_fields)`
/// tuples in declaration order. `payload_fields` is `None` for bare
/// variants (`EndTurn`) and `Some([(label, is_optional), ...])` for
/// record-payload variants (`UserToolResultBlock { tool_use_id: String,
/// content: String, is_error: Bool? }`). Handles both inline
/// (`type X = A | B | C`) and multi-line (`type X\n  = Foo { … }\n  | Bar { … }`)
/// shapes.
type V2VariantPayload = Option<Vec<(String, bool)>>;

fn v2_disj_variants(name: &str) -> Vec<(String, V2VariantPayload)> {
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
        let chunk = piece.trim();
        if chunk.is_empty() {
            continue;
        }
        // Split into label-prefix vs payload `{ … }` (if any).
        let (label_part, payload) = match chunk.find('{') {
            Some(brace) => {
                // Find the matching `}` (variants are flat — no nested
                // braces in the v2 anthropic.dag — so a forward search
                // suffices; if a future variant nests we'll need a
                // bracket counter).
                let body_end = chunk[brace + 1..].find('}').unwrap_or_else(|| {
                    panic!(
                        "v2 `type {name}` variant fragment has unmatched `{{` — \
                         lockstep parser needs an update for new v2 syntax: `{chunk}`"
                    )
                });
                let body = &chunk[brace + 1..brace + 1 + body_end];
                (
                    chunk[..brace].trim().to_string(),
                    Some(parse_v2_brace_body_fields(body)),
                )
            }
            None => (chunk.to_string(), None),
        };
        let label = label_part.trim().to_string();
        if label.is_empty() {
            continue;
        }
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
        out.push((label, payload));
    }
    out
}

/// Walk the v3 variant payload's Conj declaration and return its
/// `(field_label, is_optional)` set. Returns `None` if the variant has
/// no record payload (`Atom`/empty Conj).
fn v3_variant_payload_fields(
    dag: &Dag,
    parent_name: &str,
    variant_label: &str,
) -> Option<Vec<(String, bool)>> {
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
                    (f.label.clone(), optional)
                })
                .collect(),
        ),
        _ => None,
    }
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
    let v2_variants = v2_disj_variants(type_name);
    let v2_labels: BTreeSet<String> =
        v2_variants.iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(
        v3_labels, v2_labels,
        "lockstep drift on `type {type_name}` variant set: v3 mirror and \
         v2 source disagree. v3-only: {:?}; v2-only: {:?}",
        v3_labels.difference(&v2_labels).collect::<Vec<_>>(),
        v2_labels.difference(&v3_labels).collect::<Vec<_>>()
    );
    // Per-variant payload lockstep: each variant's record-payload field
    // set + optionality must agree between v2 source and v3 mirror.
    // Bare variants on both sides → no payload check; mismatched shapes
    // (one side bare, the other record-payload) fail closed.
    for (variant_label, v2_payload) in &v2_variants {
        let v3_payload = v3_variant_payload_fields(&dag, type_name, variant_label);
        match (v2_payload, v3_payload) {
            (None, None) => {}
            (Some(v2_fields), Some(v3_fields)) => {
                let v2_set: BTreeSet<String> =
                    v2_fields.iter().map(|(l, _)| l.clone()).collect();
                let v3_set: BTreeSet<String> =
                    v3_fields.iter().map(|(l, _)| l.clone()).collect();
                assert_eq!(
                    v3_set, v2_set,
                    "lockstep drift on payload of \
                     `type {type_name}::{variant_label}`: \
                     v3-only fields: {:?}; v2-only fields: {:?}",
                    v3_set.difference(&v2_set).collect::<Vec<_>>(),
                    v2_set.difference(&v3_set).collect::<Vec<_>>()
                );
                for (label, v2_optional) in v2_fields {
                    let v3_optional = v3_fields
                        .iter()
                        .find(|(l, _)| l == label)
                        .map(|(_, o)| *o)
                        .unwrap_or(false);
                    assert_eq!(
                        v3_optional, *v2_optional,
                        "lockstep optionality drift on \
                         `type {type_name}::{variant_label}.{label}`: v2 marks it {} \
                         but v3 lowers as {}.",
                        if *v2_optional { "optional (`T?`)" } else { "required (`T`)" },
                        if v3_optional { "Cardinality(AtMostOne, _)" } else { "non-optional" }
                    );
                }
            }
            (Some(_), None) => panic!(
                "lockstep drift: v2 `type {type_name}::{variant_label}` carries a \
                 record payload but v3 mirror has none."
            ),
            (None, Some(_)) => panic!(
                "lockstep drift: v3 mirror `type {type_name}::{variant_label}` \
                 carries a record payload but v2 source declares the variant bare."
            ),
        }
    }
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
