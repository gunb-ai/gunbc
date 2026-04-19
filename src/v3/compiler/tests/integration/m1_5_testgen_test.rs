use v3_compiler::dag::{
    Behavior, Dag, Declaration, DeclarationId, FieldValue, LiteralBits, PortState, TypeConnective,
    ValueBody,
};
use v3_compiler::lens_cost::cost_of;

use crate::common::{cached_compile_any, cached_compile_outcome, CachedCompileOutcome};
use v3_compiler::lens_testgen::{GeneratedClaim, TestgenLens};
use v3_compiler::Diagnostic;

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

fn compiled_generated_claim(claim: &GeneratedClaim<'_>) -> Dag {
    let dag = compile_any(
        &claim.render_declaration_source(),
        "generated_test_claim.dag",
    );
    assert!(
        dag.diagnostics().is_empty(),
        "generated claim declaration should compile cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let decl = generated_claim_decl(&dag, claim.declaration_name());
    assert_eq!(
        decl.meta_tag,
        dag.declaration_by_name("TestClaim").map(|decl| decl.id),
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
        variant_field(&dag, compiled_fields, "predicate").0,
        variant_field(claim.dag(), claim.fields(), "predicate").0
    );
    dag
}

fn claim_name(claim: &GeneratedClaim<'_>) -> String {
    string_field(claim.fields(), "name")
}

fn claim_holds(claim: &GeneratedClaim<'_>) -> bool {
    let source = string_field(claim.fields(), "source");
    let file_name = string_field(claim.fields(), "file_name");
    let predicate = claim
        .fields()
        .iter()
        .find(|(label, _)| label == "predicate")
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("generated claim is missing `predicate`"));
    predicate_holds(claim.dag(), &source, &file_name, predicate)
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
            // Post review round 1b.5: route through the shared
            // fixture helper so MissingCost AND negative-FoundCost
            // both fail closed. The earlier inline match only
            // handled MissingCost, which was drift flagged by the
            // chatgpt review.
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

fn executable_today(claim: &GeneratedClaim<'_>) -> bool {
    let (predicate, payload) = variant_field(claim.dag(), claim.fields(), "predicate");
    if predicate != "FailsWithDiagnostic" {
        return true;
    }
    let [reference] = payload else {
        panic!("FailsWithDiagnostic payload should be a single DiagnosticReference");
    };
    let kind = record_value(reference)
        .iter()
        .find(|(label, _)| label == "kind")
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("DiagnosticReference is missing `kind`"));
    variant_value(claim.dag(), kind).0 != "TypeMismatch"
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

// Exhaustive compile-every-claim coverage is valuable, but it is too expensive
// for the required pull-request wall-clock gate. Keep the lighter structural
// regression tests required in CI and run this sweep manually / in a non-gating
// lane until the testgen lane is reshaped to spot-check or cache more work.
#[test]
#[ignore = "slow exhaustive testgen sweep; excluded from required PR CI wall-clock gate"]
fn testgen_lens_emits_claims_as_structural_testclaim_values() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load std files cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let claims = TestgenLens::new(&dag).query();
    for claim in &claims {
        compiled_generated_claim(claim);
    }
    let claim_names: Vec<_> = claims.iter().map(claim_name).collect();
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestPredicate variant Compiles compiles"),
        "expected a compile claim for TestPredicate::Compiles, got {:?}",
        claim_names
    );
    assert!(
        claim_names.iter().any(|name| name == "TestClaim compiles"),
        "expected a compile claim for TestClaim"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "List<Int> variant Empty compiles"),
        "expected a compile claim for List<Int>::Empty"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "List<Int> requires exhaustive match"),
        "expected a non-exhaustive-match claim for List<Int>"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim witness resolves"),
        "expected a port-state claim for TestClaim witnesses, got {:?}",
        claim_names
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim witness has bounded cost"),
        "expected a cost-bounded claim for TestClaim witnesses, got {:?}",
        claim_names
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim rejects field type mismatch"),
        "expected a field-type-mismatch claim for TestClaim"
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim mismatched witness stays unresolved"),
        "expected an unresolved port-state claim for TestClaim mismatches, got {:?}",
        claim_names
    );
}

// Same rationale as the structural-value sweep above: this executes every
// generated claim against the compile boundary and currently dominates the full
// suite budget on cold CI runners.
#[test]
#[ignore = "slow exhaustive testgen sweep; excluded from required PR CI wall-clock gate"]
fn testgen_generated_claims_execute_against_compile_boundary() {
    let dag = Dag::new();
    let claims = TestgenLens::new(&dag).query();
    assert!(
        !claims.is_empty(),
        "testgen lens should emit at least one claim against the bootstrapped stdlib"
    );
    for claim in claims.iter().filter(|claim| executable_today(claim)) {
        assert!(
            claim_holds(claim),
            "generated claim should hold: name={}",
            claim_name(claim)
        );
    }
}

#[test]
fn structural_predicates_cover_four_regression_fixtures() {
    let dag = Dag::new();
    let fixtures = [
        (
            "id",
            "fn id(x: Int) -> Int = x\n",
            port_state_predicate(&dag, "id", "Resolved"),
            cost_bounded_predicate(&dag, "id", "Eq", 0),
        ),
        (
            "drop",
            "fn drop(x: Int) -> Int = 0\n",
            port_state_predicate(&dag, "drop", "Resolved"),
            cost_bounded_predicate(&dag, "drop", "Eq", 0),
        ),
        (
            "wrap",
            "type Box<T> { value: T }\nfn wrap(x: Int) -> Box<Int> = { value: x }\n",
            port_state_predicate(&dag, "wrap", "Resolved"),
            cost_bounded_predicate(&dag, "wrap", "Eq", 1),
        ),
        (
            "is_empty",
            "fn inspect_is_empty(list: List<Int>) -> Bool = match list { Empty => true, Cons(payload) => false }\n",
            port_state_predicate(&dag, "inspect_is_empty", "Resolved"),
            cost_bounded_predicate(&dag, "inspect_is_empty", "Eq", 1),
        ),
    ];

    for (name, source, state_predicate, cost_predicate) in fixtures {
        assert!(
            predicate_holds(
                &dag,
                source,
                &format!("{name}_fixture.v3"),
                &state_predicate
            ),
            "expected resolved port-state predicate to hold for fixture `{name}`"
        );
        assert!(
            predicate_holds(&dag, source, &format!("{name}_fixture.v3"), &cost_predicate),
            "expected cost predicate to hold for fixture `{name}`"
        );
    }

    let negative_source = "fn broken(list: List<Int>) -> Bool = match list { Empty => true }\n";
    assert!(
        predicate_holds(
            &dag,
            negative_source,
            "broken_fixture.v3",
            &diagnostic_predicate(&dag, "ResolveError", Some("non-exhaustive")),
        ),
        "expected a diagnostic reference to match the non-exhaustive fixture"
    );
    assert!(
        predicate_holds(
            &dag,
            negative_source,
            "broken_fixture.v3",
            &port_state_predicate(&dag, "broken", "Unresolved"),
        ),
        "expected an unresolved port-state predicate to match the non-exhaustive fixture"
    );
}
