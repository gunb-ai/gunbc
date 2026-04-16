use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::lens_testgen::{TestClaim, TestPredicate, TestgenLens};
use v3_compiler::{CompileError, Diagnostic};

fn claim_holds(claim: &TestClaim) -> bool {
    match &claim.predicate {
        TestPredicate::Compiles => compile_to_dag(&claim.source, &claim.file_name).is_ok(),
        TestPredicate::FailsWithDiagnostic { kind } => {
            match compile_to_dag(&claim.source, &claim.file_name) {
                Err(CompileError::Semantic(dag)) => diagnostic_matches(&dag, kind),
                _ => false,
            }
        }
        other => panic!("testgen currently emits only Compiles/FailsWithDiagnostic, got {other:?}"),
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

fn executable_today(claim: &TestClaim) -> bool {
    !matches!(
        claim.predicate,
        TestPredicate::FailsWithDiagnostic { ref kind } if kind == "TypeMismatch"
    )
}

#[test]
fn testgen_lens_emits_claims_for_bootstrapped_std_types() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load std files cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let claims = TestgenLens::new(&dag).query();
    assert!(
        claims
            .iter()
            .any(|claim| claim.name == "TestPredicate variant Compiles compiles"),
        "expected a compile claim for TestPredicate::Compiles, got {:?}",
        claims.iter().map(|claim| &claim.name).collect::<Vec<_>>()
    );
    assert!(
        claims
            .iter()
            .any(|claim| claim.name == "TestClaim compiles"),
        "expected a compile claim for TestClaim"
    );
    assert!(
        claims
            .iter()
            .any(|claim| claim.name == "List<Int> variant Empty compiles"),
        "expected a compile claim for List<Int>::Empty"
    );
    assert!(
        claims
            .iter()
            .any(|claim| claim.name == "List<Int> requires exhaustive match"),
        "expected a non-exhaustive-match claim for List<Int>"
    );
    assert!(
        claims
            .iter()
            .any(|claim| claim.name == "TestClaim rejects field type mismatch"),
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
