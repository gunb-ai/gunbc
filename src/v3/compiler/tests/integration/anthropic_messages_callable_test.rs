//! **Layer:** integration
//!
//! T-Substrate-AnthropicMessagesCallable shape ratchet.
//!
//! Pins the `fn anthropic_messages` declaration in
//! `src/v3/std/anthropic_messages.dag` so PR-β can rebase against a
//! stable callable target via `CallableRef.decl`. Asserts:
//!
//! - The declaration resolves in the full bootstrap and is `Arrow`-shaped.
//! - Parameter labels and types match the v3 mirror in
//!   `v3.std.anthropic_schema` plus kernel `Secret` / `String` / `Int` /
//!   `Float?` / `String?` for the primitives the v2 source carries.
//! - Output type is `AnthropicMessages200Body` (the 200 response slot,
//!   per the v2 `response { 200 => … }` row).
//! - The declaration is acceptable as a `CallableRef` target, i.e.
//!   `Operation.callable: CallableRef` with `decl` set to
//!   `anthropic_messages`'s id resolves structurally (the field type
//!   reach matches the substrate-level `DeclarationRef` admits-anything
//!   policy that PR-β fixture load fail-closes against).
//! - No `Operation` / `data anthropic_operations` row leaks here; that
//!   stays Grounding-owned in PR-β.

use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn arrow_inputs_output(
    dag: &Dag,
    name: &str,
) -> (Vec<DeclarationId>, DeclarationId) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Arrow { inputs, output, .. } => (inputs.clone(), *output),
        other => panic!("`{name}` is not an Arrow: {other:?}"),
    }
}

/// Reuse the same canonicalization rule as the schema-mirror lockstep:
/// named declarations canonicalize as their surface name; anonymous
/// `Cardinality(AtMostOne, T)` and `Instantiation { template, args }`
/// unfold.
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

#[test]
fn anthropic_messages_is_arrow_shaped() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name("anthropic_messages")
        .expect("`anthropic_messages` missing from full bootstrap");
    assert!(
        matches!(decl.connective, TypeConnective::Arrow { .. }),
        "`anthropic_messages` must be `Arrow`-shaped so `CallableRef.decl` \
         can target an actual callable signature, not a `data` row or a \
         type alias. Got: {:?}",
        decl.connective
    );
}

#[test]
fn anthropic_messages_parameter_types_match_v2_source_via_v3_mirror() {
    // Expected canonical parameter types in declaration order. These
    // mirror `dsl/extdeps/llm/anthropic.dag:182-187` modulo the two
    // documented honesty-preserving simplifications:
    //   - `max_tokens: Int` (v2 `Int = 4096`; v3 has no parameter
    //     defaults — the default is a caller-side fold, not a v3
    //     callable contract change).
    //   - return is the 200 body only (`AnthropicMessages200Body`);
    //     error responses are PR-β's response/wire lockstep concern.
    let dag = generated_full_bootstrap_dag();
    let (inputs, _output) = arrow_inputs_output(&dag, "anthropic_messages");
    let actual: Vec<String> = inputs.iter().map(|id| canonical_ty(&dag, *id)).collect();
    let expected: Vec<&str> = vec![
        "Secret",
        "String",
        "List<AnthropicChatMessage>",
        "Int",
        "Float?",
        "String?",
    ];
    assert_eq!(
        actual,
        expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "anthropic_messages parameter types diverged from the v2 \
         operation Messages signature."
    );
}

#[test]
fn anthropic_messages_returns_anthropic_messages_200_body() {
    let dag = generated_full_bootstrap_dag();
    let (_inputs, output) = arrow_inputs_output(&dag, "anthropic_messages");
    let output_canonical = canonical_ty(&dag, output);
    assert_eq!(
        output_canonical, "AnthropicMessages200Body",
        "anthropic_messages must return `AnthropicMessages200Body` \
         (the v2 `response 200 =>` slot from \
         `dsl/extdeps/llm/anthropic.dag`). Error-response carriers are \
         deferred to PR-β's response/wire lockstep lane."
    );
}

#[test]
fn anthropic_messages_is_acceptable_callable_ref_target() {
    // `CallableRef { decl: DeclarationRef }` — `DeclarationRef` admits
    // any declaration at the substrate level (the same #1175 residual
    // `MethodRef` carries). The PR-β fixture-load boundary check that
    // every `Operation.callable.decl` resolves to an `Arrow`-shaped
    // declaration (the actual callable contract) is what this test
    // pre-validates: `anthropic_messages` IS an Arrow with a non-empty
    // input list and a typed output, which is the property PR-β's
    // fail-closed boundary will verify.
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name("anthropic_messages")
        .expect("`anthropic_messages` missing from full bootstrap");
    let (inputs, output) = match &decl.connective {
        TypeConnective::Arrow { inputs, output, .. } => (inputs, *output),
        other => panic!("`anthropic_messages` is not an Arrow: {other:?}"),
    };
    assert!(
        !inputs.is_empty(),
        "callable target for `Operation.callable.decl` must carry the \
         operation's input list — anthropic_messages with empty inputs \
         is structurally unfit."
    );
    let output_decl = dag.declaration(output);
    assert!(
        output_decl.name.is_some(),
        "callable output should resolve to a named declaration, so PR-β \
         lockstep against the response/wire lane has a real type to \
         project. Got anonymous output: {:?}",
        output_decl.connective
    );
}

#[test]
fn anthropic_messages_dag_authors_no_operation_rows() {
    // The `Operation` row that closes the callable-to-source-identity
    // edge (`callable: { decl: anthropic_messages }, inputs: …,
    // endpoint: { method: POST, path: parse_path_template("/v1/messages"), … }`)
    // is Grounding-owned PR-β work in a sibling
    // `src/v3/std/anthropic_operations.dag`. Reject any `data` row
    // landing in `anthropic_messages.dag` (which would mean a
    // producerless `Operation` row or other parallel-surface scaffold
    // leaked here).
    let dag = generated_full_bootstrap_dag();
    let leaks: Vec<String> = dag
        .declarations()
        .iter()
        .filter(|d| d.span.file == "src/v3/std/anthropic_messages.dag")
        .filter(|d| d.value_body.is_some())
        .map(|d| {
            d.name
                .clone()
                .unwrap_or_else(|| format!("DeclarationId({:?})", d.id))
        })
        .collect();
    assert!(
        leaks.is_empty(),
        "anthropic_messages.dag is callable-decl authority only; no \
         `data` rows allowed. PR-β authors operation rows in a sibling \
         file. Found: {leaks:?}."
    );
}
