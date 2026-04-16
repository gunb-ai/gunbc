use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, Dag, Declaration, DeclarationId, FieldValue, LiteralBits, PortState, TypeConnective,
    ValueBody,
};
use v3_compiler::lens_cost::CostLens;
use v3_compiler::lens_testgen::{GeneratedClaim, TestgenLens};
use v3_compiler::{CompileError, Diagnostic};

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

fn generated_claim_decl<'a>(dag: &'a Dag, name: &str) -> &'a Declaration {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("generated claim declaration `{name}` not found"))
}

fn structural_fields(decl: &Declaration) -> &[(String, FieldValue)] {
    match &decl.value_body {
        Some(ValueBody::Structural { fields }) => fields.as_slice(),
        Some(ValueBody::Unparsed(_)) => panic!("generated claim should lower structurally"),
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

fn record_value<'a>(value: &'a FieldValue) -> &'a [(String, FieldValue)] {
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
        "Compiles" => compile_to_dag(source, file_name).is_ok(),
        "FailsWithDiagnostic" => {
            let [reference] = payload else {
                panic!("FailsWithDiagnostic payload should be a single DiagnosticReference");
            };
            match compile_to_dag(source, file_name) {
                Err(CompileError::Semantic(dag)) => {
                    diagnostic_matches(expectation_dag, &dag, reference)
                }
                _ => false,
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
            let [comparator, FieldValue::Literal(LiteralBits::Int(bound))] = payload else {
                panic!("CostBounded payload should be (ComparisonOp, Int)");
            };
            let dag = compile_any(source, file_name);
            let Some(bind) = final_bind(&dag) else {
                return false;
            };
            let actual = CostLens::new(&dag).cost_of(bind.value) as i64;
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
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(text)) => Some(text.as_str()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("DiagnosticReference.detail_contains must be a String literal"));
    let (kind_label, kind_payload) = variant_value(expectation_dag, kind);
    assert!(
        kind_payload.is_empty(),
        "DiagnosticKind variants should be payload-free, got {kind_payload:?}"
    );
    actual_dag.diagnostics().iter().any(|(_, diag)| {
        diagnostic_kind(diag) == kind_label && diagnostic_detail(diag).contains(detail_contains)
    })
}

fn diagnostic_kind(diag: &Diagnostic) -> &'static str {
    match diag {
        Diagnostic::TokenizerError { .. } => "TokenizerError",
        Diagnostic::ParseError { .. } => "ParseError",
        Diagnostic::TypeMismatch { .. } => "TypeMismatch",
        Diagnostic::ArityMismatch { .. } => "ArityMismatch",
        Diagnostic::ResolveError { .. } => "ResolveError",
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
    match (label.as_str(), actual) {
        ("Resolved", PortState::Resolved(_)) => true,
        ("Unresolved", PortState::Unresolved) => true,
        _ => false,
    }
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

fn final_bind(dag: &Dag) -> Option<&v3_compiler::dag::BindNode> {
    dag.nodes().iter().rev().find_map(|node| match node {
        Behavior::Bind(bind) => Some(bind),
        _ => None,
    })
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

fn diagnostic_predicate(dag: &Dag, kind: &str, detail_contains: &str) -> FieldValue {
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
                FieldValue::Literal(LiteralBits::String(detail_contains.to_string())),
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

fn cost_bounded_predicate(dag: &Dag, comparator: &str, bound: i64) -> FieldValue {
    sum_variant(
        dag,
        "TestPredicate",
        "CostBounded",
        vec![
            sum_variant(dag, "ComparisonOp", comparator, Vec::new()),
            FieldValue::Literal(LiteralBits::Int(bound)),
        ],
    )
}

#[test]
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

#[test]
fn testgen_generated_claims_execute_against_compile_boundary() {
    let dag = Dag::new();
    let claims = TestgenLens::new(&dag).query();
    assert!(
        !claims.is_empty(),
        "testgen lens should emit at least one claim against the bootstrapped stdlib"
    );
    for claim in claims.iter().filter(|claim| executable_today(claim)) {
        assert!(claim_holds(claim), "generated claim should hold: {claim:?}");
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
            cost_bounded_predicate(&dag, "Eq", 0),
        ),
        (
            "drop",
            "fn drop(x: Int) -> Int = 0\n",
            port_state_predicate(&dag, "drop", "Resolved"),
            cost_bounded_predicate(&dag, "Eq", 0),
        ),
        (
            "wrap",
            "type Box<T> { value: T }\nfn wrap(x: Int) -> Box<Int> = { value: x }\n",
            port_state_predicate(&dag, "wrap", "Resolved"),
            cost_bounded_predicate(&dag, "Eq", 1),
        ),
        (
            "is_empty",
            "fn inspect_is_empty(list: List<Int>) -> Bool = match list { Empty => true, Cons(payload) => false }\n",
            port_state_predicate(&dag, "inspect_is_empty", "Resolved"),
            cost_bounded_predicate(&dag, "Eq", 1),
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
            &diagnostic_predicate(&dag, "ResolveError", "non-exhaustive"),
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
