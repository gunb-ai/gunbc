// Thesis validation fixtures for the claims that are testable on the
// current v3 compiler, independent of the reflection work tracked in
// `swift-ram-158`.
//
// The goal of this file is traceability: each test name starts with
// the claim id from `docs/thesis-validation-plan.md` so the plan can
// point at concrete regression coverage.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, PortState, TransformTarget};
use v3_compiler::lens_cost::CostLens;
use v3_compiler::types::TypeShape;
use v3_compiler::{CompileError, Diagnostic};

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

// Tests are allowed to name declarations directly so the expected
// shapes remain legible. This is test-only infrastructure, not a
// precedent for emitter dispatch, which is separately gated against
// name-string lookups.
fn primitive_shape(dag: &Dag, name: &str) -> TypeShape {
    TypeShape::new(
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("primitive `{name}` missing from bootstrap"))
            .id,
    )
}

fn bind_named<'a>(dag: &'a Dag, name: &str) -> &'a v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

fn bind_cost(dag: &Dag, name: &str) -> usize {
    CostLens::new(dag).cost_of(bind_named(dag, name).value)
}

#[test]
fn t1_1_field_rename_propagates_to_lowered_field_projection() {
    // The full "rename a record field and re-emit Rust" path is
    // blocked today by the current emitter surface area. This fixture
    // pins the fact the emitter consumes: field access lowering carries
    // the declaration-derived label, not a compiler-side hardcoded
    // string.
    let src = "\
type Box<T> { renamed: T }
fn read(boxed: Box<Int>) -> Int = boxed.renamed
";
    let dag = compile_to_dag(src, "t1_1_field_rename.v3").expect("compiles");
    let bind = bind_named(&dag, "read");

    let projection = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind(read).value has a producer"),
    ) {
        Behavior::Transform(transform) => transform,
        other => panic!("expected field projection Transform, got {other:?}"),
    };
    match &projection.target {
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            assert_eq!(field_label, "renamed");
            // field_child should resolve to some declaration (the thesis
            // claim is that the label propagates, not that the child
            // resolves to a specific DeclarationId).
            assert!(
                field_child.is_some(),
                "field_child should be resolved after inference"
            );
        }
        other => panic!("expected FieldProject target, got {other:?}"),
    }
}

#[test]
fn t1_2_nonexistent_field_diagnostic_names_the_missing_field() {
    let src = "\
type Point { a: Int b: Int }
fn read(point: Point) -> Int = point.c
";
    let dag = match compile_to_dag(src, "t1_2_missing_field.v3") {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "read");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "missing-field access must fail closed at the bind output; got {:?}",
        dag.port(bind.value).state()
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("field `c` does not exist")
        )),
        "expected missing-field diagnostic naming `c`, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn t1_3_non_exhaustive_match_diagnostic_names_the_missing_variant() {
    let src = "\
type AB = A | B
fn read(x: AB) -> Int = match x { A => 1 }
";
    let dag = match compile_to_dag(src, "t1_3_non_exhaustive.v3") {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "read");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "non-exhaustive match must fail closed at the bind output; got {:?}",
        dag.port(bind.value).state()
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("non-exhaustive match")
                    && name.contains("`B`")
        )),
        "expected non-exhaustive match diagnostic naming `B`, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn t1_3_exhaustive_match_compiles_cleanly() {
    let src = "\
type AB = A | B
fn read(x: AB) -> Int = match x { A => 1, B => 2 }
";
    let dag = compile_to_dag(src, "t1_3_exhaustive.v3").expect("exhaustive match compiles");
    assert!(
        dag.diagnostics().is_empty(),
        "no diagnostics expected for exhaustive match, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn t1_4_type_mismatch_produces_a_typemismatch_diagnostic() {
    let dag = match compile_to_dag("let x: Bool = 1", "t1_4_type_mismatch.v3") {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "x");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "type mismatch must fail closed"
    );

    let diag = dag
        .diagnostics()
        .get(bind.value)
        .expect("diagnostic recorded for mismatched value");
    match diag {
        Diagnostic::TypeMismatch {
            expected, actual, ..
        } => {
            assert_eq!(*expected, primitive_shape(&dag, "Bool"));
            assert_eq!(*actual, primitive_shape(&dag, "Int"));
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn t1_4_type_mismatch_does_not_cascade_fabricated_diagnostics() {
    let dag = compile_any("let x: Bool = 1\nlet y: Int = 2", "t1_4_no_cascade.v3");
    let bind = bind_named(&dag, "y");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Resolved(_)),
        "well-typed binding after an unrelated type error should still resolve; got {:?}",
        dag.port(bind.value).state()
    );
    assert!(
        !dag.diagnostics().contains(bind.value),
        "well-typed downstream binding should not receive a fabricated diagnostic"
    );
}

#[test]
fn t1_5_numeric_descent_is_accepted() {
    let src = "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)
";
    let dag = compile_to_dag(src, "t1_5_numeric_descent.v3").expect("compiles");
    let bind = bind_named(&dag, "countdown");

    assert!(
        matches!(
            dag.node(
                dag.port(bind.value)
                    .produced_by
                    .expect("recursive fn body has a producer")
            ),
            Behavior::Loop(_)
        ),
        "descent-provable recursion should lower to Loop"
    );
}

#[test]
fn t1_5_structural_list_descent_is_accepted() {
    let src = "\
fn count(list: List<Int>) -> Int =
  match list { Empty => 0, Cons(p) => 1 + count(p.tail) }
";
    compile_to_dag(src, "t1_5_list_descent.v3").expect("list descent compiles");
}

#[test]
fn t1_5_missing_descent_is_rejected() {
    let dag = match compile_to_dag(
        "fn diverge(x: Int) -> Int = diverge(x)",
        "t1_5_no_descent.v3",
    ) {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "diverge");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "non-decreasing recursion must fail closed"
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("cannot prove recursion in `diverge` terminates")
        )),
        "expected descent-proof diagnostic, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn t2_4_option_like_values_require_a_match() {
    let src = "\
type Maybe<T> = Some(T) | None
fn unwrap_or_zero(m: Maybe<Int>) -> Int = match m { Some(value) => value, None => 0 }
";
    compile_to_dag(src, "t2_4_match_required.v3").expect("Option-like match compiles");
}

#[test]
fn t2_4_option_like_values_without_match_are_rejected() {
    let src = "\
type Maybe<T> = Some(T) | None
fn bad(m: Maybe<Int>) -> Int = m
";
    let dag = match compile_to_dag(src, "t2_4_missing_match.v3") {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "bad");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "using an option-like value without matching must fail closed"
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("declared signature")
        )),
        "expected a declared-signature diagnostic, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn t2_4_no_force_unwrap_primitive_exists() {
    let src = "\
type Maybe<T> = Some(T) | None
fn bad(m: Maybe<Int>) -> Int = unwrap(m)
";
    let dag = match compile_to_dag(src, "t2_4_no_unwrap.v3") {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "bad");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "unknown unwrap primitive must fail closed"
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. } if name == "unwrap"
        )),
        "expected unresolved callable diagnostic for `unwrap`, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn kf_5_bounded_fold_compiles_on_supported_primitives() {
    let src = "\
let total: Int = fold(cons(1, cons(2, singleton(3))), 0, |acc, x| acc + x)
";
    compile_to_dag(src, "kf_5_fold.v3").expect("bounded fold compiles");
}

#[test]
fn kf_5_unbounded_zero_arg_recursion_is_rejected() {
    let dag = match compile_to_dag("fn endless() -> Int = endless()", "kf_5_unbounded.v3") {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "endless");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "zero-arg recursion must fail closed"
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("recursive but has no parameters")
        )),
        "expected zero-arg recursion diagnostic, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn lens_cost_nested_program_counts_more_structure_than_flat_program() {
    let flat = compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
        "lens_cost_flat.v3",
    )
    .expect("flat program compiles");
    let nested = compile_to_dag(
        "let total: Int = fold(map(singleton(1), |x| x + 1), 0, |acc, x| acc + x)",
        "lens_cost_nested.v3",
    )
    .expect("nested program compiles");

    let flat_cost = CostLens::new(&flat).cost_of(bind_named(&flat, "total").value);
    let nested_cost = CostLens::new(&nested).cost_of(bind_named(&nested, "total").value);

    assert!(
        flat_cost > 0,
        "flat program should still carry non-zero structural cost: {flat_cost}"
    );
    assert!(
        nested_cost > flat_cost,
        "nested program should cost more structurally: flat={flat_cost}, nested={nested_cost}"
    );
}

#[test]
fn t1_5_4_structural_recursive_loop_cost_exceeds_literal_body_cost() {
    let literal = compile_to_dag("fn constant(n: Int) -> Int = 0", "lens_cost_literal_fn.v3")
        .expect("literal-bodied function compiles");
    let recursive = compile_to_dag(
        "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)
",
        "lens_cost_recursive_fn.v3",
    )
    .expect("recursive function compiles");

    let literal_cost = bind_cost(&literal, "constant");
    let recursive_cost = bind_cost(&recursive, "countdown");

    assert_eq!(literal_cost, 0, "literal function body should be a leaf");
    assert!(
        recursive_cost > literal_cost,
        "recursive loop lowering should add structural cost: literal={literal_cost}, recursive={recursive_cost}"
    );
}

#[test]
fn t1_5_4_branch_cost_tracks_the_most_expensive_path() {
    let cheaper = compile_to_dag(
        "let r: Int = if 1 > 0 then 20 + 30 else 40 + 50 + 60",
        "lens_cost_branch_cheaper.v3",
    )
    .expect("cheaper branch program compiles");
    let pricier = compile_to_dag(
        "let r: Int = if 1 > 0 then 20 + 30 + 40 + 50 else 60 + 70 + 80",
        "lens_cost_branch_pricier.v3",
    )
    .expect("pricier branch program compiles");

    let cheaper_cost = bind_cost(&cheaper, "r");
    let pricier_cost = bind_cost(&pricier, "r");

    assert!(
        pricier_cost > cheaper_cost,
        "making the most expensive branch path larger should increase branch cost: cheaper={cheaper_cost}, pricier={pricier_cost}"
    );
}

#[test]
fn kf_1_structural_list_operation_ordering_holds() {
    let singleton = compile_to_dag("let xs = singleton(1)", "lens_cost_singleton.v3")
        .expect("singleton compiles");
    let cons = compile_to_dag("let xs = cons(1, singleton(2))", "lens_cost_cons.v3")
        .expect("cons compiles");
    let fold = compile_to_dag(
        "let total: Int = fold(cons(1, singleton(2)), 0, |acc, x| acc + x)",
        "lens_cost_fold.v3",
    )
    .expect("fold compiles");
    let map_fold = compile_to_dag(
        "let total: Int = fold(map(cons(1, singleton(2)), |x| x + 1), 0, |acc, x| acc + x)",
        "lens_cost_map_fold.v3",
    )
    .expect("map+fold compiles");

    let singleton_cost = bind_cost(&singleton, "xs");
    let cons_cost = bind_cost(&cons, "xs");
    let fold_cost = bind_cost(&fold, "total");
    let map_fold_cost = bind_cost(&map_fold, "total");

    assert!(
        singleton_cost < cons_cost,
        "cons should cost more structurally than singleton: singleton={singleton_cost}, cons={cons_cost}"
    );
    assert!(
        cons_cost < fold_cost,
        "fold should cost more structurally than cons: cons={cons_cost}, fold={fold_cost}"
    );
    assert!(
        fold_cost < map_fold_cost,
        "map+fold should cost more structurally than fold: fold={fold_cost}, map_fold={map_fold_cost}"
    );
}

#[test]
fn kf_1_non_max_branch_work_does_not_change_cost() {
    let baseline = compile_to_dag(
        "let r: Int = if 1 > 0 then 10 + 20 + 30 + 40 + 50 else 60 + 70",
        "lens_cost_branch_baseline.v3",
    )
    .expect("baseline branch compiles");
    let extra_dead_work = compile_to_dag(
        "let r: Int = if 1 > 0 then 10 + 20 + 30 + 40 + 50 else 60 + 70 + 80",
        "lens_cost_branch_dead_work.v3",
    )
    .expect("branch with larger non-max path compiles");

    let baseline_cost = bind_cost(&baseline, "r");
    let extra_dead_work_cost = bind_cost(&extra_dead_work, "r");

    assert!(
        extra_dead_work_cost == baseline_cost,
        "growing only the non-max branch path should not change cost: baseline={baseline_cost}, extra_dead_work={extra_dead_work_cost}"
    );
}
