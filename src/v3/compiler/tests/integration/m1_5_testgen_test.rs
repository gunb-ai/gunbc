use v3_compiler::dag::{
    Behavior, Dag, Declaration, DeclarationId, FieldValue, LiteralBits, PortState, TypeConnective,
    ValueBody,
};
use v3_compiler::lens_cost::cost_of;
use v3_compiler::lens_testgen::{GeneratedClaim, TestgenLens};
use v3_compiler::Diagnostic;

use crate::common::{cached_compile_any, cached_compile_outcome, CachedCompileOutcome};

fn compile_any(src: &str, file: &str) -> Dag {
    cached_compile_any(src, file)
}

fn generated_claim_decl<'a>(dag: &'a Dag, name: &str) -> &'a Declaration {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("generated claim declaration `{name}` not found"))
}

fn structural_fields(decl: &Declaration) -> &[(String, FieldValue)] {
    match &decl.value_body {
        Some(ValueBody::Structural { fields }) => fields.as_slice(),
        Some(ValueBody::Unparsed(_)) => panic!("generated claim should lower structurally"),
        Some(ValueBody::Scalar(_)) => {
            panic!("generated claim should lower as Structural (record shape), got Scalar")
        }
        None => panic!("generated claim declaration should carry a structural value body"),
    }
}

fn string_field(fields: &[(String, FieldValue)], label: &str) -> String {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected `{label}` to be a String literal field"))
}

fn variant_field<'a>(
    dag: &Dag,
    fields: &'a [(String, FieldValue)],
    label: &str,
) -> (String, &'a [FieldValue]) {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("expected `{label}` field"));
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected `{label}` to lower to a variant field");
    };
    (variant_label(dag, *constructor), payload.as_slice())
}

fn variant_label(dag: &Dag, variant_id: DeclarationId) -> String {
    dag.declarations()
        .iter()
        .find_map(|decl| match &decl.connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .find(|variant| variant.ty == variant_id)
                .map(|variant| variant.label.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "variant declaration {:?} not found under any reflected sum",
                variant_id
            )
        })
}

fn variant_value<'a>(dag: &Dag, value: &'a FieldValue) -> (String, &'a [FieldValue]) {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        panic!("expected variant field value, got {value:?}");
    };
    (variant_label(dag, *constructor), payload.as_slice())
}

fn record_value(value: &FieldValue) -> &[(String, FieldValue)] {
    let FieldValue::Record(fields) = value else {
        panic!("expected record field value, got {value:?}");
    };
    fields.as_slice()
}

fn sum_variant(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
    payload: Vec<FieldValue>,
) -> FieldValue {
    let sum_decl = dag
        .declaration_by_name(sum_name)
        .unwrap_or_else(|| panic!("bootstrap should load `{sum_name}`"));
    let TypeConnective::Disj { variants } = &sum_decl.connective else {
        panic!("`{sum_name}` should lower to a Disj");
    };
    let constructor = variants
        .iter()
        .find(|variant| variant.label == variant_label)
        .map(|variant| variant.ty)
        .unwrap_or_else(|| panic!("variant `{variant_label}` not found under `{sum_name}`"));
    FieldValue::Variant {
        constructor,
        payload,
    }
}

fn diagnostic_detail_expectation(dag: &Dag, value: Option<&str>) -> FieldValue {
    match value {
        Some(text) => sum_variant(
            dag,
            "DiagnosticDetailExpectation",
            "Contains",
            vec![FieldValue::Literal(LiteralBits::String(text.to_string()))],
        ),
        None => sum_variant(dag, "DiagnosticDetailExpectation", "AnyDetail", Vec::new()),
    }
}

fn compile_generated_claim_batch(claims: &[&GeneratedClaim<'_>], file_name: &str) -> Dag {
    let source = claims
        .iter()
        .map(|claim| claim.render_declaration_source())
        .collect::<Vec<_>>()
        .join("\n");
    let dag = compile_any(&source, file_name);
    assert!(
        dag.diagnostics().is_empty(),
        "generated claim declarations should compile cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    dag
}

fn assert_compiled_generated_claim_matches(compiled_dag: &Dag, claim: &GeneratedClaim<'_>) {
    let decl = generated_claim_decl(compiled_dag, claim.declaration_name());
    assert_eq!(
        decl.meta_tag,
        compiled_dag
            .declaration_by_name("TestClaim")
            .map(|decl| decl.id),
        "generated data declaration should be typed as TestClaim"
    );
    let compiled_fields = structural_fields(decl);
    assert_eq!(string_field(compiled_fields, "name"), claim_name(claim));
    assert_eq!(
        string_field(compiled_fields, "source"),
        string_field(claim.fields(), "source")
    );
    assert_eq!(
        string_field(compiled_fields, "file_name"),
        string_field(claim.fields(), "file_name")
    );
    assert_eq!(
        variant_field(compiled_dag, compiled_fields, "predicate").0,
        variant_field(claim.dag(), claim.fields(), "predicate").0
    );
}

fn generated_claim_named<'a, 'b>(
    claims: &'b [GeneratedClaim<'a>],
    name: &str,
) -> &'b GeneratedClaim<'a> {
    claims
        .iter()
        .find(|claim| claim_name(claim) == name)
        .unwrap_or_else(|| panic!("generated claim `{name}` not found"))
}

fn claim_name(claim: &GeneratedClaim<'_>) -> String {
    string_field(claim.fields(), "name")
}

fn predicate_holds(
    expectation_dag: &Dag,
    source: &str,
    file_name: &str,
    predicate: &FieldValue,
) -> bool {
    let (label, payload) = variant_value(expectation_dag, predicate);
    match label.as_str() {
        "Compiles" => cached_compile_outcome(source, file_name).is_clean(),
        "FailsWithDiagnostic" => {
            let [reference] = payload else {
                panic!("FailsWithDiagnostic payload should be a single DiagnosticReference");
            };
            match cached_compile_outcome(source, file_name) {
                CachedCompileOutcome::Clean(_) => false,
                CachedCompileOutcome::Semantic(dag) => {
                    diagnostic_matches(expectation_dag, &dag, reference)
                }
            }
        }
        "PortHasState" => {
            let [FieldValue::Literal(LiteralBits::String(bind_name)), expected_state] = payload
            else {
                panic!("PortHasState payload should be (String, PortStateExpectation)");
            };
            let dag = compile_any(source, file_name);
            let Some(bind) = dag.nodes().iter().find_map(|node| match node {
                Behavior::Bind(bind) if bind.name == *bind_name => Some(bind),
                _ => None,
            }) else {
                return false;
            };
            port_state_matches(
                expectation_dag,
                dag.port(bind.value).state(),
                expected_state,
            )
        }
        "CostBounded" => {
            let [FieldValue::Literal(LiteralBits::String(bind_name)), comparator, FieldValue::Literal(LiteralBits::Int(bound))] =
                payload
            else {
                panic!("CostBounded payload should be (String, ComparisonOp, Int)");
            };
            let dag = compile_any(source, file_name);
            let Some(bind) = dag.nodes().iter().find_map(|node| match node {
                Behavior::Bind(bind) if bind.name == *bind_name => Some(bind),
                _ => None,
            }) else {
                return false;
            };
            let actual = crate::common::require_fixture_cost_i64(
                cost_of(&dag, &bind.value),
                &format!("bind `{bind_name}`"),
            );
            compare_cost(expectation_dag, comparator, actual, *bound)
        }
        other => panic!("unsupported TestPredicate variant {other}"),
    }
}

fn diagnostic_matches(expectation_dag: &Dag, actual_dag: &Dag, reference: &FieldValue) -> bool {
    let fields = record_value(reference);
    let kind = fields
        .iter()
        .find(|(label, _)| label == "kind")
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("DiagnosticReference is missing `kind`"));
    let detail_contains = fields
        .iter()
        .find(|(label, _)| label == "detail_contains")
        .map(|(_, value)| diagnostic_detail_filter(expectation_dag, value))
        .unwrap_or_else(|| panic!("DiagnosticReference is missing `detail_contains`"));
    let (kind_label, kind_payload) = variant_value(expectation_dag, kind);
    assert!(
        kind_payload.is_empty(),
        "DiagnosticKind variants should be payload-free, got {kind_payload:?}"
    );
    actual_dag
        .diagnostics()
        .iter()
        .any(|(_, diag)| match_detail(kind_label.as_str(), &detail_contains, diag))
}

fn diagnostic_detail_filter(dag: &Dag, value: &FieldValue) -> Option<String> {
    let (label, payload) = variant_value(dag, value);
    match label.as_str() {
        "AnyDetail" => {
            assert!(
                payload.is_empty(),
                "AnyDetail should be payload-free, got {payload:?}"
            );
            None
        }
        "Contains" => {
            let [FieldValue::Literal(LiteralBits::String(text))] = payload else {
                panic!("Contains should carry a single String literal payload, got {payload:?}");
            };
            Some(text.clone())
        }
        other => panic!("unsupported DiagnosticDetailExpectation variant {other}"),
    }
}

fn match_detail(kind_label: &str, detail_contains: &Option<String>, diag: &Diagnostic) -> bool {
    if diagnostic_kind(diag) != kind_label {
        return false;
    }
    match detail_contains {
        Some(text) => diagnostic_detail(diag).contains(text),
        None => true,
    }
}

fn diagnostic_kind(diag: &Diagnostic) -> &'static str {
    match diag {
        Diagnostic::TokenizerError { .. } => "TokenizerError",
        Diagnostic::ParseError { .. } => "ParseError",
        Diagnostic::TypeMismatch { .. } => "TypeMismatch",
        Diagnostic::ArityMismatch { .. } => "ArityMismatch",
        Diagnostic::ResolveError { .. } => "ResolveError",
        Diagnostic::BranchConditionNotBool { .. } => "BranchConditionNotBool",
    }
}

fn diagnostic_detail(diag: &Diagnostic) -> String {
    match diag {
        Diagnostic::TokenizerError { message, .. } | Diagnostic::ParseError { message, .. } => {
            message.clone()
        }
        Diagnostic::TypeMismatch {
            expected, actual, ..
        } => format!("expected {expected:?}, got {actual:?}"),
        Diagnostic::ArityMismatch {
            function,
            expected,
            actual,
            ..
        } => format!("{function} expected {expected}, got {actual}"),
        Diagnostic::ResolveError { name, .. } => name.clone(),
        Diagnostic::BranchConditionNotBool { .. } => diag.message(),
    }
}

fn port_state_matches(
    expectation_dag: &Dag,
    actual: &PortState,
    expected_state: &FieldValue,
) -> bool {
    let (label, payload) = variant_value(expectation_dag, expected_state);
    assert!(
        payload.is_empty(),
        "PortStateExpectation variants should be payload-free, got {payload:?}"
    );
    matches!(
        (label.as_str(), actual),
        ("Resolved", PortState::Resolved(_)) | ("Unresolved", PortState::Unresolved)
    )
}

fn compare_cost(expectation_dag: &Dag, comparator: &FieldValue, actual: i64, bound: i64) -> bool {
    let (label, payload) = variant_value(expectation_dag, comparator);
    assert!(
        payload.is_empty(),
        "ComparisonOp variants should be payload-free, got {payload:?}"
    );
    match label.as_str() {
        "Eq" => actual == bound,
        "Ne" => actual != bound,
        "Lt" => actual < bound,
        "Le" => actual <= bound,
        "Gt" => actual > bound,
        "Ge" => actual >= bound,
        other => panic!("unsupported ComparisonOp variant {other}"),
    }
}

fn compiles_predicate(dag: &Dag) -> FieldValue {
    sum_variant(dag, "TestPredicate", "Compiles", Vec::new())
}

fn diagnostic_predicate(dag: &Dag, kind: &str, detail_contains: Option<&str>) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "FailsWithDiagnostic",
        vec![FieldValue::Record(vec![
            (
                String::from("kind"),
                sum_variant(dag, "DiagnosticKind", kind, Vec::new()),
            ),
            (
                String::from("detail_contains"),
                diagnostic_detail_expectation(dag, detail_contains),
            ),
        ])],
    )
}

fn port_state_predicate(dag: &Dag, bind_name: &str, state: &str) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "PortHasState",
        vec![
            FieldValue::Literal(LiteralBits::String(bind_name.to_string())),
            sum_variant(dag, "PortStateExpectation", state, Vec::new()),
        ],
    )
}

fn cost_bounded_predicate(dag: &Dag, bind_name: &str, comparator: &str, bound: i64) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "CostBounded",
        vec![
            FieldValue::Literal(LiteralBits::String(bind_name.to_string())),
            sum_variant(dag, "ComparisonOp", comparator, Vec::new()),
            FieldValue::Literal(LiteralBits::Int(bound)),
        ],
    )
}

// Querying the full bootstrapped stdlib through `TestgenLens` still costs too
// much for the required PR gate. Keep one representative round-trip for manual
// runs instead of the earlier exhaustive sweep.
#[test]
#[ignore = "representative testgen query spot-check; still too expensive for required PR CI"]
fn representative_generated_claims_round_trip_as_structural_testclaim_values() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load std files cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let claims = TestgenLens::new(&dag).query();
    let representative_claims = [
        generated_claim_named(&claims, "TestClaim compiles"),
        generated_claim_named(&claims, "TestClaim witness resolves"),
        generated_claim_named(&claims, "TestClaim witness has bounded cost"),
        generated_claim_named(&claims, "List<Int> requires exhaustive match"),
        generated_claim_named(&claims, "TestClaim rejects field type mismatch"),
    ];

    for (claim, predicate_kind) in [
        (representative_claims[0], "Compiles"),
        (representative_claims[1], "PortHasState"),
        (representative_claims[2], "CostBounded"),
        (representative_claims[3], "FailsWithDiagnostic"),
        (representative_claims[4], "FailsWithDiagnostic"),
    ] {
        assert_eq!(
            variant_field(claim.dag(), claim.fields(), "predicate").0,
            predicate_kind,
            "representative claim `{}` should lower to `{predicate_kind}`",
            claim_name(claim)
        );
    }

    let compiled = compile_generated_claim_batch(
        &representative_claims,
        "representative_generated_claims_batch.dag",
    );
    for claim in representative_claims {
        assert_compiled_generated_claim_matches(&compiled, claim);
    }
}

#[test]
fn structural_predicates_cover_representative_regression_fixtures() {
    let dag = Dag::new();
    let positive_source = "type Box<T> { value: T }\nfn wrap(x: Int) -> Box<Int> = { value: x }\n";
    for (label, predicate) in [
        ("compiles", compiles_predicate(&dag)),
        (
            "resolved port-state",
            port_state_predicate(&dag, "wrap", "Resolved"),
        ),
        (
            "bounded cost",
            cost_bounded_predicate(&dag, "wrap", "Eq", 1),
        ),
    ] {
        assert!(
            predicate_holds(
                &dag,
                positive_source,
                "testgen_positive_representative_fixture.v3",
                &predicate,
            ),
            "expected representative positive predicate to hold: {label}"
        );
    }

    let negative_source = "fn broken(list: List<Int>) -> Bool = match list { Empty => true }\n";
    for (label, predicate) in [
        (
            "non-exhaustive diagnostic",
            diagnostic_predicate(&dag, "ResolveError", Some("non-exhaustive")),
        ),
        (
            "unresolved port-state",
            port_state_predicate(&dag, "broken", "Unresolved"),
        ),
    ] {
        assert!(
            predicate_holds(
                &dag,
                negative_source,
                "testgen_negative_representative_fixture.v3",
                &predicate,
            ),
            "expected representative negative predicate to hold: {label}"
        );
    }
}
