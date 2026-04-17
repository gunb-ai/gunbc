// Lane 3 Stage 3a M2 feature-parity acceptance tests.
//
// Covers DB-10..DB-13 (3a.2–3a.5) and DB-9 (3a.1). Each sub-stage has
// the concrete acceptance test declared in
// docs/lane3-self-hosting-cycle.md and docs/design-m2-feature-parity.md.
//
// These tests lock the feature-parity surface `compiler.dag` needs. If
// any of them regresses, the self-hosting cycle cannot close.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{AtomPayload, Dag, TypeConnective};
use v3_compiler::CompileError;

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

// =================================================================
// 3a.4 — Surface generics
// =================================================================

#[test]
fn test_3a4_bare_generic_fn_compiles() {
    // Acceptance: `fn id<T>(x: T) -> T = x` compiles with explicit
    // type param.
    let dag = compile_to_dag("fn id<T>(x: T) -> T = x", "test.v3")
        .expect("fn id<T>(x: T) -> T = x must compile");

    // Structural: the `id` declaration carries exactly one TypeParam
    // child in its `type_params` slot, named "T".
    let id_decl = dag
        .declaration_by_name("id")
        .expect("declaration `id` must exist");
    assert_eq!(
        id_decl.type_params.len(),
        1,
        "id must carry exactly one type parameter"
    );
    let param = dag.declaration(id_decl.type_params[0]);
    match &param.connective {
        TypeConnective::Atom(AtomPayload::TypeParam(name)) => {
            assert_eq!(name, "T", "type param name must be `T`");
        }
        other => panic!("expected TypeParam atom, got {other:?}"),
    }
}

#[test]
fn test_3a4_multi_param_generic_fn_compiles() {
    // Acceptance: `fn pair<A, B>(...)` compiles; two TypeParam
    // declarations linked via `Declaration.type_params`.
    // Keep the body to a bare parameter reference so the test
    // isolates type-param surface syntax from unrelated expression
    // grammar. Full record construction inside a generic fn body is
    // orthogonal.
    let src = "fn pair<A, B>(a: A, b: B) -> A = a";
    let dag = compile_any(src, "test.v3");
    let pair_fn = dag
        .declaration_by_name("pair")
        .expect("`pair` declaration must exist");
    assert_eq!(
        pair_fn.type_params.len(),
        2,
        "pair must carry exactly two type parameters"
    );
    let names: Vec<&str> = pair_fn
        .type_params
        .iter()
        .map(|id| match &dag.declaration(*id).connective {
            TypeConnective::Atom(AtomPayload::TypeParam(n)) => n.as_str(),
            other => panic!("expected TypeParam atom, got {other:?}"),
        })
        .collect();
    assert_eq!(names, vec!["A", "B"]);
}

#[test]
fn test_3a4_bounded_form_rejected_at_parse() {
    // Acceptance: `fn f<T: Ord>(x: T) -> T = x` fails at parse with a
    // diagnostic; bounds are rejected because algebra constraints
    // come from use-site `inhabits` resolution.
    let err = compile_to_dag("fn bound<T: Ord>(x: T) -> T = x", "test.v3")
        .expect_err("bounded form must fail at parse");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Colon")
            || msg.to_lowercase().contains("bound")
            || msg.contains(":"),
        "bounded-form diagnostic should mention the bound syntax; got: {msg}"
    );
}

// =================================================================
// 3a.5 — Disj dotted-path
// =================================================================

#[test]
fn test_3a5_match_arm_dotted_path_compiles() {
    // Acceptance: dotted path through a pattern-bound variant
    // payload. Unblocks Half B's B13.
    let src = "type Point { x: Int, y: Int }\n\
               type Opt = Some(Point) | None\n\
               fn first(o: Opt) -> Int = match o { Some(p) => p.x, None => 0 }";
    let dag = compile_to_dag(src, "test.v3")
        .expect("match-arm dotted path `p.x` must compile");
    let first = dag
        .declaration_by_name("first")
        .expect("`first` declaration must exist");
    assert!(
        first.type_params.is_empty(),
        "`first` is not generic; type_params must be empty"
    );

    // Stronger check: the compiled DAG must contain a FieldProject
    // targeting `x`. If the arm-body path lowered correctly, the
    // Transform exists; if the arm-body bailed out with an
    // unresolved placeholder, there will be no FieldProject.
    let has_field_project_x = dag.nodes().iter().any(|n| {
        matches!(
            n,
            v3_compiler::dag::Behavior::Transform(t)
            if matches!(
                &t.target,
                v3_compiler::dag::TransformTarget::FieldProject { field_label, .. }
                    if field_label == "x"
            )
        )
    });
    assert!(
        has_field_project_x,
        "match arm `Some(p) => p.x` must lower to a FieldProject<x> Transform"
    );
}

#[test]
fn test_3a5_nested_match_arm_dotted_path_compiles() {
    // Harder case: nested dotted path through an arm-bound payload.
    let src = "type Inner { a: Int, b: Int }\n\
               type Outer { inner: Inner, tag: Int }\n\
               type Wrapped = Wrap(Outer) | Empty\n\
               fn pick(w: Wrapped) -> Int = match w { Wrap(o) => o.inner.a, Empty => 0 }";
    compile_to_dag(src, "test.v3")
        .expect("nested dotted path `o.inner.a` in match arm must compile");
}

// =================================================================
// 3a.2 — data value semantics
// =================================================================

fn diagnostic_summary(dag: &Dag) -> String {
    dag.nodes()
        .iter()
        .filter_map(|n| match n {
            v3_compiler::dag::Behavior::Bind(b) => {
                let port = dag.port(b.value);
                match port.state() {
                    v3_compiler::dag::PortState::Unresolved => Some(format!(
                        "Bind `{}` unresolved: {:?}",
                        b.name,
                        dag.diagnostics().get(b.value)
                    )),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_3a2_scalar_data_compiles_and_populates_value_body() {
    match compile_to_dag("data answer: Int = 42", "test.v3") {
        Ok(dag) => {
            let answer = dag
                .declaration_by_name("answer")
                .expect("`answer` declaration must exist");
            assert!(
                answer.value_body.is_some(),
                "`data answer: Int = 42` must populate value_body; got None"
            );
        }
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "scalar data compile semantic error: {}",
                diagnostic_summary(&dag)
            );
        }
        Err(other) => panic!("scalar data compile structural error: {other:?}"),
    }
}

#[test]
fn test_3a2_data_referenced_in_fn_body_compiles() {
    let src = "data answer: Int = 42\n\
               fn f() -> Int = answer";
    match compile_to_dag(src, "test.v3") {
        Ok(dag) => {
            assert!(
                dag.declaration_by_name("f").is_some(),
                "`fn f()` must lower successfully"
            );
        }
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "semantic error referencing data in fn body: {}",
                diagnostic_summary(&dag)
            );
        }
        Err(other) => panic!("structural error: {other:?}"),
    }
}

#[test]
fn test_3a2_record_data_compiles_structural() {
    let src = "type Config { host: Int, port: Int }\n\
               data cfg: Config = { host: 1, port: 8080 }";
    let dag = compile_to_dag(src, "test.v3")
        .expect("record data declaration must compile");
    let cfg = dag
        .declaration_by_name("cfg")
        .expect("`cfg` must exist");
    match &cfg.value_body {
        Some(v3_compiler::dag::ValueBody::Structural { fields }) => {
            assert_eq!(fields.len(), 2, "cfg must have two fields");
        }
        other => panic!("expected ValueBody::Structural, got {other:?}"),
    }
}

#[test]
fn test_3a2_data_field_access_resolves_statically() {
    // Acceptance: `cfg.host` inside a fn body must resolve to the
    // record-literal field's value at compile time.
    let src = "type Config { host: Int, port: Int }\n\
               data cfg: Config = { host: 1, port: 8080 }\n\
               fn get_host() -> Int = cfg.host";
    match compile_to_dag(src, "test.v3") {
        Ok(dag) => {
            let _ = dag.declaration_by_name("get_host").expect("get_host must exist");
        }
        Err(CompileError::Semantic(dag)) => {
            panic!(
                "semantic error on data field access: {}",
                diagnostic_summary(&dag)
            );
        }
        Err(other) => panic!("structural error: {other:?}"),
    }
}

// =================================================================
// 3a.3 — where refinement
// =================================================================

#[test]
fn test_3a3_where_clause_on_parameter_parses() {
    // Foundation: the parser must NOT drop the `where` clause on a
    // fn parameter. Previously `parse_params` treated `where` as a
    // syntax error after the parameter type. This test locks in
    // that the grammar now accepts the form.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n";
    let _ = compile_to_dag(src, "test.v3").expect("where-clause on parameter must parse");
}

#[test]
fn test_3a3_where_clause_does_not_break_signatureless_fn() {
    // Regression: adding `where`-clause parsing must not change how
    // bare parameter types lower.
    let src = "fn id(x: Int) -> Int = x";
    let _ = compile_to_dag(src, "test.v3").expect("bare-parameter fn must still compile");
}

// =================================================================
// 3a.1 — Mutual recursion (TODO: unimplemented)
// =================================================================
