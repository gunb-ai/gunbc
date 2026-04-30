//! **Layer:** integration
//!
//! Acceptance for `anthropic_operations: List<Operation>` substrate fixture
//! at `src/v3/std/anthropic_operations.dag` (T-Ground services.dag PR-β
//! Phase 1 pilot; mirrors the #1195 `MethodTemplateContract` Phase 1
//! pattern). Validates:
//!
//! 1. The fixture lowers as `ValueBody::List` (not `Unparsed` —
//!    post-#1195/#1196 regression-class lesson).
//! 2. `Operation.callable.decl` ids are unique within the list (the
//!    structural invariant analogous to `dag_method` uniqueness in
//!    `method_template_contract_per_target_dag_method_unique`; pivot
//!    from `name` to `callable.decl` per #1246's structural decision
//!    that `Operation` doesn't carry a parallel display-name field).
//! 3. The expected pilot row is present and resolves through
//!    `callable.decl` to the `anthropic_messages` callable from #1266
//!    at `src/v3/std/anthropic_messages.dag` — lockstep with the v2
//!    source of truth at `dsl/extdeps/llm/anthropic.dag:180-203`.
//!    Endpoint shape: POST /v1/messages (LiteralToken("v1") +
//!    LiteralToken("messages")).
//! 4. Every `ParamToken.name` from the path template resolves into the
//!    operation's input-field map. The walk + assertion are wired
//!    structurally; vacuous for `/v1/messages` (zero `ParamToken`s)
//!    AND for the empty-input Phase 1 Messages row, but rows added
//!    later (provider operations with path-template variables AND
//!    populated inputs once the parser-grammar extension lands)
//!    inherit the discipline by construction without test rewrite.
//!
//! Input-keys lockstep parity check is **deferred** for the Messages
//! pilot: `Operation.inputs` is `{}` until Substrate's parser-grammar
//! extension supports nested `Map<String, X>` literals as record
//! field values (per `anthropic_operations.dag` header EXPLICIT
//! DEFERRAL §1 + #1133 inbox 4353126932). Re-asserts on Phase 1.5+
//! row-fill cycle.

use std::collections::HashSet;
use v3_compiler::dag::{Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;

const ANTHROPIC_OPERATIONS: &str = "anthropic_operations";

/// Resolve a `Variant.constructor` DeclarationId to the variant's
/// **structural** label by consulting the parent sum's `Disj.variants`
/// list — NOT by reading `Declaration.name` on the constructor decl
/// (which is a bolted-on convenience that may drift from the
/// declared-variant authority). The Disj's variants list IS the
/// single-authority for variant-label-to-constructor mapping.
fn variant_label_in_parent(dag: &Dag, parent_name: &str, constructor: DeclarationId) -> String {
    let parent = dag
        .declaration_by_name(parent_name)
        .unwrap_or_else(|| panic!("expected parent sum `{parent_name}` missing from bootstrap"));
    let TypeConnective::Disj { variants } = &parent.connective else {
        panic!(
            "`{parent_name}` is not a Disj sum; cannot resolve variant labels: {:?}",
            parent.connective
        );
    };
    variants
        .iter()
        .find(|v| v.ty == constructor)
        .map(|v| v.label.clone())
        .unwrap_or_else(|| {
            panic!(
                "constructor `DeclarationId({})` is not a variant of `{parent_name}`; \
                 declared variants: {:?}",
                constructor.raw(),
                variants.iter().map(|v| &v.label).collect::<Vec<_>>()
            )
        })
}

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

/// Resolve `Operation.callable.decl` (the typed-edge into the
/// callable-decl registry, per `services.dag:98-100` `CallableRef`
/// shape) to the underlying DeclarationId. Used for uniqueness
/// checks + pilot-row lookups; callable identity replaces the
/// previously-modeled `name: String` field per #1246's structural
/// decision (`Operation` doesn't carry a parallel display-name field).
fn callable_decl_id(fields: &[(String, FieldValue)]) -> DeclarationId {
    let callable = record_field(fields, "callable");
    let FieldValue::Record(callable_fields) = callable else {
        panic!("`callable` must be a `CallableRef` record; got {callable:?}");
    };
    let decl = record_field(callable_fields, "decl");
    let FieldValue::Reference(id) = decl else {
        panic!(
            "`callable.decl` must be a `FieldValue::Reference(DeclarationId)` \
             pointing at a top-level callable; got {decl:?}"
        );
    };
    *id
}

#[test]
fn anthropic_operations_lowers_as_list() {
    let dag = generated_full_bootstrap_dag();
    let _rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);
}

#[test]
fn anthropic_operations_callable_unique() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);

    let mut seen: HashSet<DeclarationId> = HashSet::new();
    for (idx, row) in rows.iter().enumerate() {
        let FieldValue::Record(fields) = row else {
            panic!(
                "row {idx} in `{ANTHROPIC_OPERATIONS}` is not a `FieldValue::Record` — \
                 every `Operation` row must be a record literal"
            );
        };
        let id = callable_decl_id(fields);
        assert!(
            seen.insert(id),
            "duplicate `callable.decl` ({}) in `{ANTHROPIC_OPERATIONS}` at row {idx}",
            id.raw()
        );
    }
}

#[test]
#[ignore = "Phase 1 pilot row deferred until Substrate's parser-grammar extension \
            lands (nested Map<String, X> literals in record field-value position). \
            Once anthropic_operations.dag's Messages row populates, drop the \
            #[ignore] and re-arm; the assertions inside still match the post-#1246 \
            shape (callable.decl == anthropic_messages, POST /v1/messages, \
            empty/populated inputs)."]
fn anthropic_operations_messages_pilot_present() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);

    let anthropic_messages_id = dag
        .declaration_by_name("anthropic_messages")
        .expect("expected callable `anthropic_messages` (per #1266) missing from full bootstrap")
        .id;

    let messages = rows
        .iter()
        .find_map(|row| match row {
            FieldValue::Record(fields) => {
                if callable_decl_id(fields) == anthropic_messages_id {
                    Some(fields)
                } else {
                    None
                }
            }
            _ => None,
        })
        .expect(
            "Messages operation pilot row (callable.decl == anthropic_messages) missing from \
             anthropic_operations",
        );

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
    let method_label = variant_label_in_parent(&dag, "HttpMethod", *constructor);
    assert_eq!(
        method_label, "POST",
        "Messages endpoint method must be POST (per anthropic.dag:197); got `{method_label}`"
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
        .map(|token| {
            let FieldValue::Variant {
                constructor,
                payload,
            } = token
            else {
                panic!("expected UrlPathToken variant; got {token:?}");
            };
            // Assert the constructor IS LiteralToken before extracting text.
            // Resolution goes through the parent `UrlPathToken` Disj's
            // `variants` list — the structural single-authority.
            let variant_label = variant_label_in_parent(&dag, "UrlPathToken", *constructor);
            assert_eq!(
                variant_label, "LiteralToken",
                "Messages endpoint path must contain only LiteralToken \
                 variants (per anthropic.dag:198 `/v1/messages` literal segments); \
                 got `{variant_label}` token"
            );
            assert_eq!(
                payload.len(),
                1,
                "LiteralToken payload must carry one field; got {payload:?}"
            );
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
        })
        .collect();
    assert_eq!(
        token_texts,
        vec!["v1".to_string(), "messages".to_string()],
        "Messages endpoint path must be /v1/messages (per anthropic.dag:198); \
         got tokens {token_texts:?}"
    );

    // Inputs check: empty for the Phase 1 Messages pilot (parser-grammar
    // gap defers populated input-keys map; see anthropic_operations.dag
    // EXPLICIT DEFERRAL §1). When Substrate's parser-grammar extension
    // lands, the row's `inputs` populates and this assertion grows to
    // re-establish v2-parity input-key set per anthropic.dag:183-189.
    let inputs = record_field(messages, "inputs");
    let FieldValue::Map(input_map) = inputs else {
        panic!("`inputs` must be a `Map<String, InputField>`; got {inputs:?}");
    };
    assert!(
        input_map.entries().is_empty(),
        "Messages `inputs` must be empty for the Phase 1 pilot (parser-grammar gap \
         defers nested-map literals). Once Substrate's parser extension lands, \
         populate this row + grow the assertion to v2-parity keys. Got {} entries: {:?}",
        input_map.entries().len(),
        input_map.entries().iter().map(|(k, _)| k).collect::<Vec<_>>()
    );
}

/// Boundary check: every `ParamToken.name` token in every operation's
/// path template MUST resolve to a key in that operation's
/// `inputs: Map<String, InputField>`. Vacuous on the Phase 1 pilot
/// (`Messages` has zero `ParamToken`s — `/v1/messages` is pure
/// literal segments — AND zero inputs pending parser-grammar
/// extension), but the walk + assertion are wired so any future row
/// carrying path-template variables AND populated inputs inherits
/// the discipline without test rewrite.
#[test]
fn anthropic_operations_param_tokens_resolve_to_input_keys() {
    let dag = generated_full_bootstrap_dag();
    let rows = list_value_body(&dag, ANTHROPIC_OPERATIONS);

    for (idx, row) in rows.iter().enumerate() {
        let FieldValue::Record(fields) = row else {
            panic!("row {idx} not a record");
        };
        let op_callable_id = callable_decl_id(fields);
        let op_label = dag
            .declaration(op_callable_id)
            .name
            .as_deref()
            .map(str::to_string)
            .unwrap_or_else(|| format!("DeclarationId({})", op_callable_id.raw()));

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
            let variant_label = variant_label_in_parent(&dag, "UrlPathToken", *constructor);
            if variant_label == "ParamToken" {
                let name = match payload.first() {
                    Some(FieldValue::Literal(_)) => string_literal(&payload[0]).to_string(),
                    Some(FieldValue::Record(inner)) => {
                        string_literal(record_field(inner, "name")).to_string()
                    }
                    other => panic!("unexpected ParamToken payload: {other:?}"),
                };
                assert!(
                    input_keys.contains(name.as_str()),
                    "operation `{op_label}` path token {tidx} `ParamToken({name})` does \
                     not resolve to an input field key; have {input_keys:?}"
                );
            }
        }
    }
}
