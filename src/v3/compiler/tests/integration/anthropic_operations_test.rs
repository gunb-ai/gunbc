//! **Layer:** integration
//!
//! Acceptance for `anthropic_operations: List<Operation>` substrate fixture
//! at `src/v3/std/anthropic_operations.dag` (T-Ground services.dag PR-β
//! Phase 1 pilot; mirrors the #1195 `MethodTemplateContract` Phase 1
//! pattern). Validates:
//!
//! 1. The fixture lowers as `ValueBody::List` (not `Unparsed` —
//!    post-#1195/#1196 regression-class lesson).
//! 2. Operation names are unique within the list (the structural
//!    invariant analogous to `dag_method` uniqueness in
//!    `method_template_contract_per_target_dag_method_unique`).
//! 3. The expected pilot row (`Messages`) is present with the expected
//!    endpoint shape (POST /v1/messages) — lockstep with the v2 source
//!    of truth at `dsl/extdeps/llm/anthropic.dag:182-198`.
//! 4. Every `ParamToken.name` from the path template resolves into the
//!    operation's input-field map. The walk + assertion are wired
//!    structurally; vacuous for `/v1/messages` (zero `ParamToken`s)
//!    but rows added later (path-template variables like
//!    `/v1/secrets/{secret_name}`) inherit the discipline by
//!    construction without test rewrite.

use std::collections::HashSet;
use v3_compiler::dag::{Dag, FieldValue, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;

const ANTHROPIC_OPERATIONS: &str = "anthropic_operations";

fn list_value_body<'a>(dag: &'a Dag, name: &str) -> &'a Vec<FieldValue> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    let body = decl
        .value_body
        .as_ref()
        .unwrap_or_else(|| panic!("`{name}` has no value body — must be a `data` declaration"));
    let ValueBody::List(rows) = body else {
        panic!(
            "`{name}` value body must be `ValueBody::List` (declared as \
             `List<Operation>`); got {body:?}"
        );
    };
    rows
}

fn record_field<'a>(fields: &'a [(String, FieldValue)], label: &str) -> &'a FieldValue {
    fields
        .iter()
        .find(|(l, _)| l == label)
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("missing `{label}` field; have {fields:?}"))
}

fn string_literal(value: &FieldValue) -> &str {
    use v3_compiler::dag::LiteralBits;
    match value {
        FieldValue::Literal(LiteralBits::String(s)) => s.as_str(),
        other => panic!("expected String literal; got {other:?}"),
    }
}

#[test]
fn anthropic_operations_lowers_as_list() {
    let dag = generated_full_bootstrap_dag();
    // Just calling list_value_body asserts the body is List — panics otherwise.
    let _rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);
}

#[test]
fn anthropic_operations_names_unique() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);

    let mut seen: HashSet<String> = HashSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let FieldValue::Record(fields) = row else {
            panic!(
                "row {idx} in `{ANTHROPIC_OPERATIONS}` is not a `FieldValue::Record` — \
                 every `Operation` row must be a record literal"
            );
        };
        let name = string_literal(record_field(fields, "name")).to_string();
        assert!(
            seen.insert(name.clone()),
            "duplicate operation name `{name}` in `{ANTHROPIC_OPERATIONS}` at row {idx}"
        );
    }
}

#[test]
fn anthropic_operations_messages_pilot_present() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);

    let messages = rows
        .iter()
        .find_map(|row| match row {
            FieldValue::Record(fields) => {
                if string_literal(record_field(fields, "name")) == "Messages" {
                    Some(fields)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect("`Messages` operation pilot row missing from anthropic_operations");

    // Endpoint check: method=POST, path=/v1/messages (LiteralToken("v1") +
    // LiteralToken("messages")). Lockstep with v2 source of truth at
    // dsl/extdeps/llm/anthropic.dag:194-200.
    let endpoint = record_field(messages, "endpoint");
    let FieldValue::Record(endpoint_fields) = endpoint else {
        panic!("`endpoint` field must be a record; got {endpoint:?}");
    };

    let method = record_field(endpoint_fields, "method");
    let FieldValue::Variant { constructor, .. } = method else {
        panic!("`endpoint.method` must be a `HttpMethod` variant; got {method:?}");
    };
    let method_name = dag
        .declaration(*constructor)
        .name
        .as_deref()
        .expect("HttpMethod variant should have a name");
    assert_eq!(
        method_name, "POST",
        "Messages endpoint method must be POST (per anthropic.dag:197); got `{method_name}`"
    );

    let path = record_field(endpoint_fields, "path");
    let FieldValue::Record(path_fields) = path else {
        panic!("`endpoint.path` must be a `PathTemplate` record; got {path:?}");
    };
    let tokens = record_field(path_fields, "tokens");
    let FieldValue::List(token_list) = tokens else {
        panic!("`endpoint.path.tokens` must be a List; got {tokens:?}");
    };
    let token_texts: Vec<String> = token_list
        .iter()
        .map(|token| match token {
            FieldValue::Variant { payload, .. } if payload.len() == 1 => {
                // LiteralToken { text: String } — payload[0] is the text record/field.
                // The variant payload encoding may vary; accept either a direct String
                // literal or a single-field Record { text: <String> }.
                match &payload[0] {
                    FieldValue::Literal(_) => string_literal(&payload[0]).to_string(),
                    FieldValue::Record(inner) => {
                        string_literal(record_field(inner, "text")).to_string()
                    }
                    other => panic!("unexpected LiteralToken payload shape: {other:?}"),
                }
            }
            other => panic!("expected LiteralToken variant; got {other:?}"),
        })
        .collect();
    assert_eq!(
        token_texts,
        vec!["v1".to_string(), "messages".to_string()],
        "Messages endpoint path must be /v1/messages (per anthropic.dag:198); \
         got tokens {token_texts:?}"
    );

    // Input keys present (lockstep with v2 input block at anthropic.dag:183-189).
    let inputs = record_field(messages, "inputs");
    let FieldValue::Map(input_map) = inputs else {
        panic!("`inputs` must be a `Map<String, InputField>`; got {inputs:?}");
    };
    let expected_inputs: HashSet<&str> = [
        "api_key",
        "model",
        "messages",
        "max_tokens",
        "temperature",
        "system",
    ]
    .into_iter()
    .collect();
    let actual_refs: HashSet<&str> = input_map
        .entries()
        .iter()
        .map(|(k, _)| k.as_str())
        .collect();
    assert_eq!(
        actual_refs, expected_inputs,
        "Messages input-field keys must match anthropic.dag:183-189 input block"
    );
}

/// Boundary check: every `ParamToken.name` token in every operation's
/// path template MUST resolve to a key in that operation's
/// `inputs: Map<String, InputField>`. Vacuous on the Phase 1 pilot
/// (`Messages` has zero `ParamToken`s — `/v1/messages` is pure
/// literal segments), but the walk + assertion are wired so any
/// future row carrying path-template variables inherits the
/// discipline without test rewrite. This is the structural
/// `ParamToken.name` resolution check named in
/// `src/v3/std/services.dag` PR-α header (boundary check at fixture
/// load alongside the first row).
#[test]
fn anthropic_operations_param_tokens_resolve_to_input_keys() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);

    for (idx, row) in rows.iter().enumerate() {
        let FieldValue::Record(fields) = row else {
            panic!("row {idx} not a record");
        };
        let op_name = string_literal(record_field(fields, "name")).to_string();

        let inputs = record_field(fields, "inputs");
        let FieldValue::Map(input_map) = inputs else {
            panic!("`inputs` must be a Map; got {inputs:?}");
        };
        let input_keys: HashSet<&str> = input_map
            .entries()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();

        let endpoint = record_field(fields, "endpoint");
        let FieldValue::Record(endpoint_fields) = endpoint else {
            panic!("`endpoint` must be a record");
        };
        let path = record_field(endpoint_fields, "path");
        let FieldValue::Record(path_fields) = path else {
            panic!("`path` must be a PathTemplate record");
        };
        let tokens = record_field(path_fields, "tokens");
        let FieldValue::List(token_list) = tokens else {
            panic!("`path.tokens` must be a List");
        };

        for (tidx, token) in token_list.iter().enumerate() {
            let FieldValue::Variant {
                constructor,
                payload,
            } = token
            else {
                panic!("path token {tidx} must be a UrlPathToken variant; got {token:?}");
            };
            let variant_name = dag
                .declaration(*constructor)
                .name
                .as_deref()
                .expect("UrlPathToken variant should have a name");
            if variant_name == "ParamToken" {
                // ParamToken { name: String } — payload[0] is the name.
                let name = match payload.first() {
                    Some(FieldValue::Literal(_)) => string_literal(&payload[0]).to_string(),
                    Some(FieldValue::Record(inner)) => {
                        string_literal(record_field(inner, "name")).to_string()
                    }
                    other => panic!("unexpected ParamToken payload: {other:?}"),
                };
                assert!(
                    input_keys.contains(name.as_str()),
                    "operation `{op_name}` path token {tidx} `ParamToken({name})` does \
                     not resolve to an input field key; have {input_keys:?}"
                );
            }
            // LiteralToken variants are unconditionally accepted; no resolution check.
        }
    }
}
