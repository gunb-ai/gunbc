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
        .unwrap_or_else(|| panic!("variant declaration {:?} not found under any reflected sum", variant_id))
}

fn compiled_generated_claim(claim: &GeneratedClaim) -> Dag {
    let dag = compile_any(&claim.declaration_source, "generated_test_claim.dag");
    assert!(
        dag.diagnostics().is_empty(),
        "generated claim declaration should compile cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let decl = generated_claim_decl(&dag, &claim.declaration_name);
    assert_eq!(
        decl.meta_tag,
        dag.declaration_by_name("TestClaim").map(|decl| decl.id),
        "generated data declaration should be typed as TestClaim"
    );
    dag
}

fn claim_name(claim: &GeneratedClaim) -> String {
    let dag = compiled_generated_claim(claim);
    string_field(structural_fields(generated_claim_decl(&dag, &claim.declaration_name)), "name")
}

fn claim_holds(claim: &GeneratedClaim) -> bool {
    let dag = compiled_generated_claim(claim);
    let fields = structural_fields(generated_claim_decl(&dag, &claim.declaration_name));
    let source = string_field(fields, "source");
    let file_name = string_field(fields, "file_name");
    let (predicate, payload) = variant_field(&dag, fields, "predicate");

    match predicate.as_str() {
        "Compiles" => compile_to_dag(&source, &file_name).is_ok(),
        "FailsWithDiagnostic" => {
            let [FieldValue::Literal(LiteralBits::String(kind))] = payload else {
                panic!("FailsWithDiagnostic payload should be a single String literal");
            };
            match compile_to_dag(&source, &file_name) {
                Err(CompileError::Semantic(dag)) => diagnostic_matches(&dag, kind),
                _ => false,
            }
        }
        other => panic!("testgen currently emits only Compiles/FailsWithDiagnostic, got {other}"),
    }
}

fn diagnostic_matches(dag: &Dag, kind: &str) -> bool {
    dag.diagnostics()
        .iter()
        .any(|(_, diag)| match (kind, diag) {
            ("TypeMismatch", Diagnostic::TypeMismatch { .. }) => true,
            (needle, Diagnostic::ResolveError { name, .. }) => name.contains(needle),
            _ => false,
        })
}

fn executable_today(claim: &GeneratedClaim) -> bool {
    let dag = compiled_generated_claim(claim);
    let fields = structural_fields(generated_claim_decl(&dag, &claim.declaration_name));
    let (predicate, payload) = variant_field(&dag, fields, "predicate");
    if predicate != "FailsWithDiagnostic" {
        return true;
    }
    let [FieldValue::Literal(LiteralBits::String(kind))] = payload else {
        panic!("FailsWithDiagnostic payload should be a single String literal");
    };
    kind != "TypeMismatch"
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
