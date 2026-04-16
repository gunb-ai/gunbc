use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, DeclarationId, PortState, TypeConnective};
use v3_compiler::CompileError;

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

fn find_named(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn record_fields(dag: &Dag, name: &str) -> Vec<String> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Conj { children } => {
            children.iter().map(|field| field.label.clone()).collect()
        }
        other => panic!("expected `{name}` to lower to a Conj, got {other:?}"),
    }
}

fn sum_variants(dag: &Dag, name: &str) -> Vec<(String, Vec<String>)> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .map(|variant| {
                let payload = match &dag.declaration(variant.ty).connective {
                    TypeConnective::Conj { children } => {
                        children.iter().map(|field| field.label.clone()).collect()
                    }
                    other => panic!(
                        "expected variant `{}` under `{name}` to lower to a Conj payload, got {other:?}",
                        variant.label
                    ),
                };
                (variant.label.clone(), payload)
            })
            .collect(),
        other => panic!("expected `{name}` to lower to a Disj, got {other:?}"),
    }
}

fn bind_value_type_decl(dag: &Dag, name: &str) -> DeclarationId {
    let value_port = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Bind(bind) if bind.name == name => Some(bind.value),
            _ => None,
        })
        .unwrap_or_else(|| panic!("bind `{name}` not found"));
    match dag.port(value_port).state() {
        PortState::Resolved(ty) => ty.declaration,
        other => panic!("bind `{name}` did not resolve, got {other:?}"),
    }
}

#[test]
fn bootstrap_loads_verification_authority_types() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load staged std.verification cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    assert_eq!(
        record_fields(&dag, "TestClaim"),
        vec!["name", "source", "file_name", "predicate"]
    );
    assert_eq!(record_fields(&dag, "TestSuite"), vec!["name", "claims"]);
    assert_eq!(
        sum_variants(&dag, "DiagnosticKind"),
        vec![
            (String::from("TokenizerError"), Vec::new()),
            (String::from("ParseError"), Vec::new()),
            (String::from("TypeMismatch"), Vec::new()),
            (String::from("ArityMismatch"), Vec::new()),
            (String::from("ResolveError"), Vec::new()),
        ]
    );
    assert_eq!(
        record_fields(&dag, "DiagnosticReference"),
        vec![String::from("kind"), String::from("detail_contains")]
    );
    assert_eq!(
        sum_variants(&dag, "DiagnosticDetailExpectation"),
        vec![
            (String::from("AnyDetail"), Vec::new()),
            (String::from("Contains"), vec![String::from("_0")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "PortStateExpectation"),
        vec![
            (String::from("Resolved"), Vec::new()),
            (String::from("Unresolved"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "TestPredicate"),
        vec![
            (String::from("Compiles"), Vec::new()),
            (
                String::from("FailsWithDiagnostic"),
                vec![String::from("_0")],
            ),
            (String::from("OutputEquals"), vec![String::from("expected")],),
            (
                String::from("PortHasState"),
                vec![String::from("bind_name"), String::from("state")],
            ),
            (
                String::from("CostBounded"),
                vec![
                    String::from("bind_name"),
                    String::from("comparator"),
                    String::from("bound"),
                ],
            ),
        ]
    );
}

#[test]
fn verification_predicate_witnesses_compile_cleanly() {
    let src = r#"
let pred_compiles: TestPredicate = Compiles
let pred_fails: TestPredicate = FailsWithDiagnostic({ kind: ResolveError, detail_contains: Contains("missing") })
let pred_fails_kind: TestPredicate = FailsWithDiagnostic({ kind: TypeMismatch, detail_contains: AnyDetail })
let pred_output: TestPredicate = OutputEquals("let x: Int = 1")
let pred_port_resolved: TestPredicate = PortHasState("answer", Resolved)
let pred_port_unresolved: TestPredicate = PortHasState("missing", Unresolved)
let pred_cost_eq: TestPredicate = CostBounded("answer", Eq, 8)
let pred_cost_above: TestPredicate = CostBounded("answer", Gt, 3)

let claim_compiles: TestClaim = {
  name: "compiles",
  source: "let x: Int = 1",
  file_name: "compiles.v3",
  predicate: pred_compiles
}

let claim_fails: TestClaim = {
  name: "fails",
  source: "let x: Bool = 1",
  file_name: "fails.v3",
  predicate: pred_fails
}

let suite: TestSuite = {
  name: "verification_smoke",
  claims: [claim_compiles, claim_fails]
}
"#;

    let dag = compile_any(src, "verification_witnesses.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "verification witnesses should compile cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let test_predicate = find_named(&dag, "TestPredicate");
    for bind in [
        "pred_compiles",
        "pred_fails",
        "pred_fails_kind",
        "pred_output",
        "pred_port_resolved",
        "pred_port_unresolved",
        "pred_cost_eq",
        "pred_cost_above",
    ] {
        assert_eq!(bind_value_type_decl(&dag, bind), test_predicate);
    }
    assert_eq!(
        bind_value_type_decl(&dag, "claim_compiles"),
        find_named(&dag, "TestClaim")
    );
    assert_eq!(
        bind_value_type_decl(&dag, "claim_fails"),
        find_named(&dag, "TestClaim")
    );
    assert_eq!(
        bind_value_type_decl(&dag, "suite"),
        find_named(&dag, "TestSuite")
    );
}
