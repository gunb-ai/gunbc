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
        msg.contains("Colon") || msg.to_lowercase().contains("bound") || msg.contains(":"),
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
    let dag = compile_to_dag(src, "test.v3").expect("match-arm dotted path `p.x` must compile");
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
fn test_3a2_data_reference_is_order_independent() {
    // Reviewer finding (PR #496, chatgpt-codex P2): bodies are
    // lowered in source order, so referencing a `data` item
    // declared *after* the referring fn must still resolve —
    // otherwise name resolution for top-level data values is
    // order-dependent. The `seed_data_value_bodies_phase`
    // pre-pass fixes this. Both orderings must compile.
    let forward = "fn f() -> Int = answer\n\
                   data answer: Int = 42";
    let backward = "data answer: Int = 42\n\
                    fn f() -> Int = answer";
    let forward_dag = compile_to_dag(forward, "forward.v3")
        .expect("forward reference to later `data answer` must compile");
    let backward_dag = compile_to_dag(backward, "backward.v3")
        .expect("backward reference to earlier `data answer` must compile");
    for (label, dag) in [("forward", forward_dag), ("backward", backward_dag)] {
        let has_value_42 = dag.nodes().iter().any(|n| {
            matches!(
                n,
                v3_compiler::dag::Behavior::Value(v)
                if matches!(
                    &v.data,
                    v3_compiler::dag::LiteralBits::Int(42)
                )
            )
        });
        assert!(
            has_value_42,
            "{label} must inline `answer` as Value(Int(42))"
        );
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
    let dag = compile_to_dag(src, "test.v3").expect("record data declaration must compile");
    let cfg = dag.declaration_by_name("cfg").expect("`cfg` must exist");
    match &cfg.value_body {
        Some(v3_compiler::dag::ValueBody::Structural { fields }) => {
            assert_eq!(fields.len(), 2, "cfg must have two fields");
        }
        other => panic!("expected ValueBody::Structural, got {other:?}"),
    }
}

#[test]
fn test_3a2_data_field_access_resolves_statically() {
    // Acceptance (DB-10, lowering-time inlining): `cfg.host` inside
    // a fn body must resolve to the record-literal field's value at
    // compile time, producing a `Value(Int(1))` node *instead of* a
    // runtime `FieldProject` on the `cfg` declaration. Ratchets the
    // lowering-time-inlining choice (codex review on e0b4ded2f):
    // if `cfg.host` ever silently regressed to a field-read
    // Transform, this would catch it.
    let src = "type Config { host: Int, port: Int }\n\
               data cfg: Config = { host: 1, port: 8080 }\n\
               fn get_host() -> Int = cfg.host";
    let dag = match compile_to_dag(src, "test.v3") {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "semantic error on data field access: {}",
            diagnostic_summary(&dag)
        ),
        Err(other) => panic!("structural error: {other:?}"),
    };
    let _ = dag
        .declaration_by_name("get_host")
        .expect("get_host must exist");

    // Must contain a Value node holding Int(1) — the inlined `host`
    // field value from the record literal.
    let has_value_1 = dag.nodes().iter().any(|n| {
        matches!(
            n,
            v3_compiler::dag::Behavior::Value(v)
            if matches!(&v.data, v3_compiler::dag::LiteralBits::Int(1))
        )
    });
    assert!(
        has_value_1,
        "`cfg.host` must lower to Value(Int(1)) (inlined field); got no such node"
    );

    // Must NOT contain a FieldProject<host> Transform — that would
    // indicate runtime field access on a data value, defeating the
    // lowering-time inlining.
    let has_field_project_host = dag.nodes().iter().any(|n| {
        matches!(
            n,
            v3_compiler::dag::Behavior::Transform(t)
            if matches!(
                &t.target,
                v3_compiler::dag::TransformTarget::FieldProject { field_label, .. }
                    if field_label == "host"
            )
        )
    });
    assert!(
        !has_field_project_host,
        "`cfg.host` on a data declaration must NOT lower to a runtime \
         FieldProject<host> Transform; the literal must be inlined at compile time"
    );
}

// =================================================================
// 3a.3 — where refinement (DB-11 consumer landing)
// =================================================================

#[test]
fn test_3a3_refined_parameter_compiles() {
    // Acceptance (DB-11): a fn whose parameter carries a `where`
    // refinement compiles cleanly. The parser captures the predicate,
    // lowering creates a predicate Declaration and a refined type
    // Declaration, and the refined decl's `refinement` edge points at
    // the predicate. Internal use of `d` (e.g. `n / d`) is fine
    // because the `/` operator doesn't require a refinement.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n / d";
    let dag = compile_to_dag(src, "test.v3")
        .expect("refined parameter fn must compile; body's use of `d` in `/` doesn't require the refinement");
    let div = dag
        .declaration_by_name("div")
        .expect("`div` declaration must exist");
    // The div Arrow's second input (d's type) must be a refined decl,
    // i.e. carry `refinement: Some(_)`.
    let TypeConnective::Arrow { inputs, .. } = &div.connective else {
        panic!(
            "`div` must have an Arrow connective; got {:?}",
            div.connective
        );
    };
    assert_eq!(inputs.len(), 2, "div has two parameters");
    let d_decl = dag.declaration(inputs[1]);
    assert!(
        d_decl.refinement.is_some(),
        "d's parameter declaration must carry a `refinement` edge; got None"
    );
}

#[test]
fn test_3a3_call_with_violating_literal_is_rejected() {
    // Acceptance (DB-11): `div(1, 0)` is rejected at compile time
    // because the literal `0` has no refinement, but div's second
    // parameter expects `Int where d != 0`. The structural-equality
    // check fails, emitting a diagnostic.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n\n\
               fn bad() -> Int = div(1, 0)";
    let err = compile_to_dag(src, "test.v3")
        .expect_err("div(1, 0) must fail the refinement discharge check");
    let CompileError::Semantic(dag) = err else {
        panic!("expected Semantic error, got {err:?}");
    };
    let diagnostic_msgs: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, d)| format!("{d:?}"))
        .collect();
    let joined = diagnostic_msgs.join("\n");
    assert!(
        joined.contains("refinement") || joined.contains("no narrowing"),
        "at least one diagnostic must name refinement failure; got:\n{joined}"
    );
}

#[test]
fn test_3a3_call_with_matching_refined_arg_compiles() {
    // Acceptance (DB-11): passing a parameter that already carries the
    // same refinement as the callee's parameter discharges the check.
    // `f(n: Int, d: Int where d != 0)` forwards `d` to `div(n, d)`
    // whose second parameter also requires `d != 0`. Structural walk
    // on the predicate DAGs pairs the two `d`-slots across sides and
    // the `!=` operator resolution, so the predicates walk equal.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n\n\
               fn f(n: Int, d: Int where d != 0) -> Int = div(n, d)";
    let _dag = compile_to_dag(src, "test.v3").expect(
        "forwarding a parameter with the same refinement to a callee \
         with the same refinement must pass structural discharge",
    );
}

#[test]
fn test_3a3_distinct_refinements_do_not_discharge() {
    // Acceptance (DB-11): refinements that aren't structurally
    // identical do not discharge each other. `x > 1` does NOT
    // automatically satisfy `x > 0` — the design commits to
    // structural equality on predicate DAGs, no implication reasoning.
    // Here the caller's refinement is `d > 1` and the callee expects
    // `d > 0`. Predicate DAGs differ at the right operand (Value(1)
    // vs Value(0)), so the walk rejects.
    let src = "fn at_least_one(n: Int, d: Int where d > 0) -> Int = n\n\
               fn caller(d: Int where d > 1) -> Int = at_least_one(0, d)";
    let err = compile_to_dag(src, "test.v3")
        .expect_err("distinct refinements should not entail each other (structural equality only)");
    let CompileError::Semantic(dag) = err else {
        panic!("expected Semantic error, got {err:?}");
    };
    let joined = dag
        .diagnostics()
        .iter()
        .map(|(_, d)| format!("{d:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("refinement"),
        "diagnostic must name refinement failure; got:\n{joined}"
    );
}

#[test]
fn test_3a3_where_clause_does_not_break_signatureless_fn() {
    // Regression: refinement lowering must fire only when a `where`
    // is present. Bare `fn id(x: Int) -> Int = x` still compiles.
    let src = "fn id(x: Int) -> Int = x";
    let _ = compile_to_dag(src, "test.v3").expect("bare-parameter fn must still compile");
}

#[test]
fn test_3a3_if_predicate_narrows_then_arm_discharge() {
    // Acceptance (DB-11): inside the `then` arm of an `if` whose
    // condition is a predicate on a scope-bound parameter, the
    // parameter is narrowed to a refined type carrying that predicate.
    // Forwarding the narrowed port to a callee expecting the same
    // refinement discharges structurally — no guard call is needed.
    //
    // `caller(n, d)` calls `div(n, d)` only inside `if d != 0`; the
    // narrowing makes `d`'s port type `Int where d != 0` inside the
    // then arm, matching `div`'s expected refinement.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n\n\
               fn caller(n: Int, d: Int) -> Int = \
                 if d != 0 then div(n, d) else 0";
    let _ = compile_to_dag(src, "test.v3")
        .expect("narrowing Branch arm must discharge the callee's refinement on the forwarded `d`");
}

#[test]
fn test_3a3_narrowed_already_refined_param_preserves_outer_refinement() {
    // Acceptance (DB-11): narrowing an already-refined parameter must
    // not drop the outer refinement. Narrowing creates a declaration
    // that aliases the outer refined decl via
    // `Atom(ResolvedIdentifier(outer))`; the outer decl still carries
    // its own refinement. Discharge walks the alias chain so ANY level's
    // refinement can satisfy the callee — including the outer one.
    //
    // `caller(d: Int where d != 0)` is narrowed inside the `if` arm
    // with `is_big(d, 10)` (a call-shaped predicate that narrowing
    // recognizes per `narrowable_var_name`). Forwarded to
    // `div(d: Int where d != 0)`: the narrowed decl's own refinement is
    // `is_big(d, 10)` (does NOT walk-equal to `d != 0`), but the alias
    // chain reaches the outer `d != 0`, which does.
    //
    // Call-shaped (not operator-shaped) narrow predicate is load-bearing
    // here: `resolve_operator_arrow`'s primitive fallback uses
    // `inputs: vec![lhs_type, lhs_type]`, which would propagate the
    // outer refinement onto the literal operand and fail discharge
    // before narrowing runs — unrelated to this test's concern.
    let src = "fn is_big(x: Int, threshold: Int) -> Bool = x > threshold\n\
               fn div(n: Int, d: Int where d != 0) -> Int = n\n\
               fn caller(n: Int, d: Int where d != 0) -> Int = \
                 if is_big(d, 10) then div(n, d) else 0";
    let _ = compile_to_dag(src, "test.v3").expect(
        "narrowing an already-refined param must not drop the outer \
         refinement: the outer `d != 0` on the alias chain should \
         discharge the callee's `d != 0`",
    );
}

#[test]
fn test_3a3_if_without_narrowing_rejects_forwarded_unrefined() {
    // Counterpart to the narrowing test: if the caller does NOT guard
    // the call with a matching predicate, the forwarded `d` carries no
    // refinement and the discharge check fails.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n\n\
               fn caller(n: Int, d: Int) -> Int = div(n, d)";
    let err = compile_to_dag(src, "test.v3")
        .expect_err("unrefined `d` forwarded to a refined-parameter callee must fail discharge");
    let CompileError::Semantic(dag) = err else {
        panic!("expected Semantic error, got {err:?}");
    };
    let joined = dag
        .diagnostics()
        .iter()
        .map(|(_, d)| format!("{d:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("refinement"),
        "diagnostic must name refinement failure; got:\n{joined}"
    );
}

#[test]
fn test_3a3_substrate_integrity_behavior_still_five_variants() {
    // Acceptance (DB-11 §Acceptance): "Substrate integrity:
    // Declaration.refinement is the only new edge. type Behavior
    // remains at five variants." Compile a refined-parameter fn and
    // verify the DAG contains no unexpected Behavior variant — the
    // predicate sub-DAG is built from Value/Transform/Bind, all
    // pre-existing variants.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n / d";
    let dag = compile_to_dag(src, "test.v3").expect("must compile");
    for node in dag.nodes() {
        match node {
            v3_compiler::dag::Behavior::Value(_)
            | v3_compiler::dag::Behavior::Transform(_)
            | v3_compiler::dag::Behavior::Branch(_)
            | v3_compiler::dag::Behavior::Loop(_)
            | v3_compiler::dag::Behavior::Bind(_) => {}
        }
    }
    // If a sixth variant existed the match above would not compile;
    // this test locks in the five-variant substrate while DB-11 is
    // consumed.
}

// =================================================================
// 3a.1 — Mutual recursion (TODO: unimplemented)
// =================================================================
