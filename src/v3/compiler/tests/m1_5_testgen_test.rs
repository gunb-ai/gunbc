use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Dag, Declaration, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody,
};
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
    let (predicate, payload) = variant_field(claim.dag(), claim.fields(), "predicate");

    match predicate.as_str() {
        "Compiles" => compile_to_dag(&source, &file_name).is_ok(),
        "FailsWithDiagnostic" => {
            let [expectation] = payload else {
                panic!("FailsWithDiagnostic payload should be a single DiagnosticExpectation");
            };
            match compile_to_dag(&source, &file_name) {
                Err(CompileError::Semantic(dag)) => {
                    diagnostic_matches(claim.dag(), &dag, expectation)
                }
                _ => false,
            }
        }
        other => panic!("testgen currently emits only Compiles/FailsWithDiagnostic, got {other}"),
    }
}

fn diagnostic_matches(expectation_dag: &Dag, actual_dag: &Dag, expectation: &FieldValue) -> bool {
    let (label, payload) = variant_value(expectation_dag, expectation);
    match label.as_str() {
        "KindIs" => {
            let [kind] = payload else {
                panic!("KindIs payload should carry one DiagnosticKind");
            };
            let (kind_label, kind_payload) = variant_value(expectation_dag, kind);
            assert!(
                kind_payload.is_empty(),
                "DiagnosticKind variants should be payload-free, got {kind_payload:?}"
            );
            actual_dag
                .diagnostics()
                .iter()
                .any(|(_, diag)| diagnostic_kind(diag) == kind_label)
        }
        "ResolveNameContains" => {
            let [FieldValue::Literal(LiteralBits::String(needle))] = payload else {
                panic!("ResolveNameContains payload should be a single String literal");
            };
            actual_dag.diagnostics().iter().any(|(_, diag)| match diag {
                Diagnostic::ResolveError { name, .. } => name.contains(needle),
                _ => false,
            })
        }
        other => panic!("unsupported DiagnosticExpectation variant {other}"),
    }
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

fn executable_today(claim: &GeneratedClaim<'_>) -> bool {
    let (predicate, payload) = variant_field(claim.dag(), claim.fields(), "predicate");
    if predicate != "FailsWithDiagnostic" {
        return true;
    }
    let [expectation] = payload else {
        panic!("FailsWithDiagnostic payload should be a single DiagnosticExpectation");
    };
    let (label, nested) = variant_value(claim.dag(), expectation);
    if label != "KindIs" {
        return true;
    }
    let [kind] = nested else {
        panic!("KindIs payload should carry one DiagnosticKind");
    };
    variant_value(claim.dag(), kind).0 != "TypeMismatch"
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
            .any(|name| name == "PortState variant Unresolved compiles"),
        "expected a compile claim for bootstrapped std substrate type PortState, got {:?}",
        claim_names
    );
    assert!(
        claim_names
            .iter()
            .any(|name| name == "TestClaim rejects field type mismatch"),
        "expected a field-type-mismatch claim for TestClaim"
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
