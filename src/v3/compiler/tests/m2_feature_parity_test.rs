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
    // with the operator-shaped predicate `d > 10`, then forwarded to
    // `div(d: Int where d != 0)`. The narrowed decl's own refinement
    // is `d > 10` (does NOT walk-equal to `d != 0`); the alias chain
    // reaches the outer `d != 0`, which does.
    //
    // Operator-shaped narrow predicates are the common surface shape
    // that `narrowable_var_name` recognizes. Making this path honest
    // requires `resolve_operator_arrow` to strip refinements from
    // operand positions — otherwise the literal `10` would be typed
    // as `Int where d != 0` (the mirrored lhs carrier) and fail
    // discharge before narrowing ever runs. See
    // `strip_refinement_to_base` in `infer.rs`.
    let src = "fn div(n: Int, d: Int where d != 0) -> Int = n\n\
               fn caller(n: Int, d: Int where d != 0) -> Int = \
                 if d > 10 then div(n, d) else 0";
    let _ = compile_to_dag(src, "test.v3").expect(
        "operator-shaped narrow predicate on an already-refined param \
         must compile: stripped operator operands let lowering succeed, \
         and the alias-chain walk rediscovers the outer `d != 0` to \
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
fn test_3a3_callable_predicate_structural_identity_across_sites() {
    // Acceptance (DB-11): two syntactically identical calls to the
    // same generic predicate from different refinement contexts must
    // discharge each other. Call lowering materializes a fresh
    // `Instantiation` declaration per call-site when the callee has
    // retained template arguments (see
    // `retained_template_arguments_for_target` in `lower.rs`) — so
    // `Callable(DeclarationId)` nominal identity is per-site, not
    // per-callee. Structural comparison via template + substituted
    // arguments (`declaration_shapes_equivalent`) is the true
    // identity; without it, the two `where always_false(d, 0)` sites
    // below have differently-identified Callable targets even though
    // they call the same generic with the same inferred type
    // argument (T = Int), and forwarding `a(d)` from `b`'s body fails
    // discharge on nominal id mismatch.
    let src = "fn always_false<T>(x: T, y: T) -> Bool = 0 == 1\n\
               fn a(d: Int where always_false(d, 0)) -> Int = d\n\
               fn b(d: Int where always_false(d, 0)) -> Int = a(d)";
    let _ = compile_to_dag(src, "test.v3").expect(
        "two identical calls to the same generic predicate from \
         different refinement sites must discharge structurally, \
         regardless of per-site Instantiation id",
    );
}

#[test]
fn test_3a3_logical_and_in_refinement_parses_and_lowers() {
    // Acceptance (DB-11): `where <lhs> && <rhs>` — the `&&` /
    // `||` logical primitives land so composite refinement bodies
    // and narrowing-over-narrowing can be represented canonically.
    // Bool-monomorphic: inputs and output are always Bool,
    // independent of `lhs_type`.
    //
    // The composite-canonical refactor that makes structural
    // discharge accept a sub-conjunct of actual's predicate is a
    // separate concern; this test only locks that the primitive
    // parses, lowers to `TransformTarget::Operator(Logical(And))`,
    // and type-checks.
    let src = "fn in_range(d: Int where d > 0 && d < 10) -> Int = d";
    let _ = compile_to_dag(src, "test.v3").expect(
        "`where d > 0 && d < 10` must compile — logical conjunction \
         must parse, infer Bool-Bool-Bool, and lower to a \
         Logical(And) Transform",
    );
}

#[test]
fn test_3a3_logical_or_in_refinement_parses_and_lowers() {
    // Acceptance (DB-11): counterpart to the `&&` test — `||`
    // lands in the same round and with the same semantics.
    let src = "fn out_of_range(d: Int where d < 0 || d > 100) -> Int = d";
    let _ = compile_to_dag(src, "test.v3").expect(
        "`where d < 0 || d > 100` must compile — logical disjunction \
         must parse, infer Bool-Bool-Bool, and lower to a \
         Logical(Or) Transform",
    );
}

#[test]
fn test_3a3_logical_operator_rejects_non_bool_operand() {
    // Acceptance (DB-11): `&&` is Bool-monomorphic. An Int operand
    // must surface as a type mismatch on the Bool input slot, not
    // silently propagate through as the Arithmetic / Comparison
    // fallback does with `lhs_type`. Counterexample lock for the
    // Logical arm of `resolve_operator_arrow`'s primitive fallback.
    let src = "fn f(d: Int) -> Bool = d && d";
    let err = compile_to_dag(src, "test.v3")
        .expect_err("`Int && Int` must fail: `&&` operands must be Bool");
    let msg = format!("{err:?}");
    assert!(
        msg.to_lowercase().contains("type")
            || msg.to_lowercase().contains("mismatch")
            || msg.to_lowercase().contains("bool"),
        "diagnostic must name the type mismatch or Bool expectation; got: {msg}"
    );
}

#[test]
fn test_3a3_conjunction_discharge_ignores_grouping() {
    // Acceptance (DB-11): conjunction is a logical fact, not a
    // syntax tree. `a && (b && c)` and `(a && b) && c` represent
    // the same fact and must discharge symmetrically. Regression
    // lock against the substrate-level grouping blocker called out
    // by the ChatGPT review on `31a3709d`.
    //
    // Narrowing naturally produces right-associated composites
    // because `build_narrowed_refinement` wraps `outer && new_cond`:
    // when the narrowing cond is itself a composite `b && c` (left-
    // associated by the parser), the final composite is
    // `outer && (b && c)` — structurally distinct from the callee's
    // left-associated `outer && b`.
    //
    // Caller outer: `a` (= d != 0). Narrowing cond: `b && c`
    // (= d > 0 && d < 100). Narrowed composite body:
    // `a && (b && c)`. Callee expected: `a && b`. Pre-flatten the
    // root-conjunct-only walk couldn't find `a && b` as a subtree
    // of `a && (b && c)` and rejected the program. Post-flatten,
    // both sides reduce to conjunct multisets (actual: {a, b, c};
    // expected: {a, b}) and the subset check discharges.
    let src = "fn takes_ab(d: Int where d != 0 && d > 0) -> Int = d\n\
               fn caller(n: Int, d: Int where d != 0) -> Int = \
                 if d > 0 && d < 100 then takes_ab(d) else 0";
    let _ = compile_to_dag(src, "test.v3").expect(
        "narrowing-produced right-associated composite `a && (b && c)` \
         must discharge callee's `a && b` — flatten-and-subset \
         normalizes conjunction independently of guard grouping",
    );
}

#[test]
fn test_3a3_refinement_references_top_level_data_constant() {
    // Acceptance (DB-11): a `where` predicate that references a
    // top-level `data` declaration must resolve cleanly. Regression
    // lock against seed-phase ordering: `seed_function_signatures_phase`
    // runs before the data pre-pass populates `data` declarations'
    // connectives and `value_body`s. If parameter refinement lowering
    // happens at seed time, references to `THRESHOLD` inside the
    // predicate would resolve against a placeholder declaration and
    // mark unresolved even though `THRESHOLD` is a valid Int constant.
    //
    // The fix splits refinement lowering into its own dedicated
    // phase (`lower_parameter_refinements_phase`) that runs AFTER
    // the data pre-pass. Sole caller of `lower_parameter_refinement`
    // for parameter `where` clauses — single-construction-authority
    // invariant preserved.
    let src = "data THRESHOLD: Int = 10\n\
               fn big(d: Int where d > THRESHOLD) -> Int = d";
    let _ = compile_to_dag(src, "test.v3").expect(
        "a `where` predicate that references a top-level `data` \
         constant must compile — refinement lowering runs after the \
         data pre-pass",
    );
}

#[test]
fn test_3a3_rejects_out_of_fragment_refinement_predicate() {
    // Acceptance (DB-11): the discharge walker and composite-
    // narrowing clone path only model `Value` / `Transform` predicate
    // bodies. `where` predicates that lower through `Branch` / `Loop` /
    // `Bind` are admitted by lowering but cannot be compared or cloned
    // downstream — pre-fix they failed silently at discharge as
    // generic "not equal" diagnostics, never at the actual boundary.
    //
    // Lowering now fail-closes at the refinement phase with an
    // explicit "unsupported shape" diagnostic naming the out-of-
    // fragment construct. Reviewer R6 (`df5fc7b3` codex review)
    // called this out as a BLOCKING fail-closed violation — admitted
    // surface > supported fragment without an honest boundary
    // diagnostic.
    let src = "fn f(d: Int where if d > 0 then d != 0 else d > 1) -> Int = d";
    let err = compile_to_dag(src, "test.v3")
        .expect_err("`where if cond then ... else ...` must be rejected at lowering");
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
        joined.to_lowercase().contains("not supported")
            || joined.to_lowercase().contains("unsupported")
            || joined.contains("`if`"),
        "diagnostic must name the unsupported refinement shape; got:\n{joined}"
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
