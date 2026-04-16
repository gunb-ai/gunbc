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
            assert_eq!(*field_child, Some(primitive_shape(&dag, "Int").declaration));
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
    let dag = compile_any("let x: Bool = 1", "t1_4_type_mismatch.v3");
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
#[ignore = "blocked on structural List<T> carrier / canonicalization work in swift-ram-158"]
fn t1_5_structural_list_descent_is_accepted() {
    // Enabled once structural list descent lands on main. Keeping
    // the exact fixture here makes the validation-plan blocker
    // explicit and gives us a ready-to-flip regression gate.
    let src = "\
fn count(list: List<Int>) -> Int =
  match list { Empty => 0, Cons(p) => 1 + count(p.tail) }
";
    compile_to_dag(src, "t1_5_list_descent.v3").expect("list descent compiles");
}

#[test]
fn t1_5_missing_descent_is_rejected() {
    let dag = compile_any("fn diverge(x: Int) -> Int = diverge(x)", "t1_5_no_descent.v3");
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
fn t2_4_no_force_unwrap_primitive_exists() {
    let src = "\
type Maybe<T> = Some(T) | None
fn bad(m: Maybe<Int>) -> Int = unwrap(m)
";
    let dag = compile_any(src, "t2_4_no_unwrap.v3");
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
let total: Int = fold_int(cons_int(1, cons_int(2, singleton_int(3))), 0, |acc, x| acc + x)
";
    compile_to_dag(src, "kf_5_fold.v3").expect("bounded fold compiles");
}

#[test]
fn kf_5_unbounded_zero_arg_recursion_is_rejected() {
    let dag = compile_any("fn endless() -> Int = endless()", "kf_5_unbounded.v3");
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
        "let total: Int = fold_int(singleton_int(1), 0, |acc, x| acc + x)",
        "lens_cost_flat.v3",
    )
    .expect("flat program compiles");
    let nested = compile_to_dag(
        "let total: Int = fold_int(map_int(singleton_int(1), |x| x + 1), 0, |acc, x| acc + x)",
        "lens_cost_nested.v3",
    )
    .expect("nested program compiles");

    let flat_cost = CostLens::new(&flat).cost_of(bind_named(&flat, "total").value);
    let nested_cost = CostLens::new(&nested).cost_of(bind_named(&nested, "total").value);

    assert_eq!(flat_cost, 2, "flat structural cost should stay pinned");
    assert_eq!(nested_cost, 3, "nested structural cost should stay pinned");
    assert!(
        nested_cost > flat_cost,
        "nested program should cost more structurally: flat={flat_cost}, nested={nested_cost}"
    );
}
