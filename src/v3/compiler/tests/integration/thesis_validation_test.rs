// Thesis validation fixtures for the claims that are testable on the
// current v3 compiler, independent of the reflection work tracked in
// `swift-ram-158`.
//
// The goal of this file is traceability: each test name starts with
// the claim id from `docs/thesis-validation-plan.md` so the plan can
// point at concrete regression coverage.

use v3_compiler::dag::{Behavior, Dag, PortState, TransformTarget};
use v3_compiler::diagnostics::{render_diagnostic_for_target, Correction, DiagnosticStyleTarget};
use v3_compiler::lens_cost::cost_of;

use crate::common::{
    cached_compile_any, cached_compile_outcome, cached_compile_to_dag, CachedCompileOutcome,
};
use v3_compiler::types::TypeShape;
use v3_compiler::Diagnostic;

fn compile_any(src: &str, file: &str) -> Dag {
    cached_compile_any(src, file)
}

fn compile_semantic_fixture(src: &str, file: &str) -> Dag {
    match cached_compile_outcome(src, file) {
        CachedCompileOutcome::Semantic(dag) => dag,
        other => panic!("expected semantic failure for {file}, got {other:?}"),
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
    let port = bind_named(dag, name).value;
    crate::common::require_fixture_cost_usize(cost_of(dag, &port), &format!("bind `{name}`"))
}

fn rendered_rust_diagnostic(dag: &Dag, diagnostic: &Diagnostic) -> String {
    render_diagnostic_for_target(dag, DiagnosticStyleTarget::Rust, diagnostic).expect("render")
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
    let dag = cached_compile_to_dag(src, "t1_1_field_rename.v3");
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
        TransformTarget::ResolvedFieldProject { .. } => {}
        other => panic!("expected FieldProject target, got {other:?}"),
    }
}

#[test]
fn t1_2_nonexistent_field_diagnostic_names_the_missing_field() {
    let src = "\
type Point { a: Int b: Int }
fn read(point: Point) -> Int = point.c
";
    let dag = compile_semantic_fixture(src, "t1_2_missing_field.v3");
    let bind = bind_named(&dag, "read");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "missing-field access must fail closed at the bind output; got {:?}",
        dag.port(bind.value).state()
    );
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. } if name.contains("field `c` does not exist") => {
                Some(diag)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected missing-field diagnostic naming `c`, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    assert!(
        matches!(diag.correction(), Correction::DeferredCorrection { .. }),
        "ambiguous missing-field diagnostic should carry an explicit deferral, got {:?}",
        diag.correction()
    );
    let rendered = rendered_rust_diagnostic(&dag, diag);
    assert!(
        !rendered.contains("FIX:"),
        "ambiguous missing-field diagnostic should not render an arbitrary FIX line, got {rendered}"
    );
    assert!(
        rendered.contains("DEFERRED CORRECTION:")
            && rendered.contains("multiple valid replacements")
            && rendered.contains("a")
            && rendered.contains("b"),
        "ambiguous missing-field diagnostic should surface concrete alternatives without marking them applyable, got {rendered}"
    );
    assert!(
        !rendered.contains("\n    \"a\";") && !rendered.contains("\n    \"b\";"),
        "ambiguous missing-field diagnostic should not choose between fields, got {rendered}"
    );
}

#[test]
fn t1_2_chained_missing_field_fix_targets_the_missing_segment() {
    let src = "\
type Inner { leaf: Int }
type Outer { ok: Inner }
fn read(x: Outer) -> Int = x.bad.leaf
";
    let bad_start = src.find("bad").expect("fixture contains missing field") as u32;
    let dag = compile_semantic_fixture(src, "t1_2_chained_missing_field.v3");
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. }
                if name.contains("field `bad` does not exist") =>
            {
                Some(diag)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected chained missing-field diagnostic, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    let fix = diag.correction();
    assert_eq!(
        fix.new_source(),
        Some("ok"),
        "missing-field fix should suggest `ok`"
    );
    let span = fix.span().expect("missing-field fix should be live");
    assert_eq!(span.byte_start, bad_start);
    assert_eq!(span.byte_end, bad_start + "bad".len() as u32);
}

#[test]
fn t1_3_non_exhaustive_match_diagnostic_names_the_missing_variant() {
    let src = "\
type AB = A | B
fn read(x: AB) -> Int = match x { A => 1 }
";
    let dag = compile_semantic_fixture(src, "t1_3_non_exhaustive.v3");
    let bind = bind_named(&dag, "read");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "non-exhaustive match must fail closed at the bind output; got {:?}",
        dag.port(bind.value).state()
    );
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. }
                if name.contains("non-exhaustive match") && name.contains("`B`") =>
            {
                Some(diag)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected non-exhaustive match diagnostic naming `B`, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    assert!(
        diag.correction().description().contains("`B`"),
        "non-exhaustive match diagnostic should suggest the missing `B` arm"
    );
}

#[test]
fn t1_3_empty_match_correction_seeds_first_arm_without_leading_comma() {
    let src = "\
type AB = A | B
fn read(x: AB) -> Int = match x {}
";
    let dag = compile_semantic_fixture(src, "t1_3_empty_match.v3");
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. } if name.contains("non-exhaustive match") => {
                Some(diag)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected non-exhaustive match diagnostic, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    assert!(
        diag.correction()
            .new_source()
            .is_some_and(|new_source| !new_source.starts_with(", ") && new_source == "A => 1"),
        "empty match fix should seed a valid first arm, got {:?}",
        diag.correction()
    );
}

#[test]
fn t1_3_exhaustive_match_compiles_cleanly() {
    let src = "\
type AB = A | B
fn read(x: AB) -> Int = match x { A => 1, B => 2 }
";
    let dag = cached_compile_to_dag(src, "t1_3_exhaustive.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "no diagnostics expected for exhaustive match, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn t1_4_type_mismatch_produces_a_typemismatch_diagnostic() {
    let dag = compile_semantic_fixture("let x: Bool = 1", "t1_4_type_mismatch.v3");
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
            expected,
            actual,
            correction,
            ..
        } => {
            assert_eq!(*expected, primitive_shape(&dag, "Bool"));
            assert_eq!(*actual, primitive_shape(&dag, "Int"));
            assert!(
                matches!(
                    correction,
                    v3_compiler::diagnostics::Correction::LiveCorrection { .. }
                ),
                "type mismatch diagnostic should carry a live correction"
            );
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
fn t1_4_named_payload_sum_fix_renders_supported_constructor_syntax() {
    let src = "\
type MaybeInt = Some { value: Int } | None
let x: MaybeInt = true
";
    let dag = compile_semantic_fixture(src, "t1_4_named_payload_sum_fix.v3");
    let bind = bind_named(&dag, "x");
    let diag = dag
        .diagnostics()
        .get(bind.value)
        .expect("diagnostic recorded for mismatched sum value");
    let Diagnostic::TypeMismatch { correction, .. } = diag else {
        panic!("expected TypeMismatch, got {diag:?}");
    };
    let rendered = rendered_rust_diagnostic(&dag, diag);
    assert!(
        rendered.contains("\n    \"Some(1)\";"),
        "rendered diagnostic should show the supported positional constructor syntax, got {rendered}"
    );
    assert_eq!(correction.new_source(), Some("Some(1)"));
}

#[test]
fn t1_4_refined_declarations_do_not_get_shape_only_fix_witnesses() {
    let src = "\
fn div(n: Int, d: Int where d != 0) -> Int = n
fn bad() -> Int = div(1, nope)
";
    let dag = compile_semantic_fixture(src, "t1_4_refined_no_fix_witness.v3");
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. } if name == "nope" => Some(diag),
            _ => None,
        })
        .expect("diagnostic recorded for unresolved name in refined argument position");
    let Diagnostic::ResolveError { correction, .. } = diag else {
        panic!("expected ResolveError, got {diag:?}");
    };
    assert!(
        matches!(
            correction,
            v3_compiler::diagnostics::Correction::DeferredCorrection { .. }
        ),
        "refined declarations must fail closed instead of emitting a base-shape witness; got {correction:?}"
    );
}

#[test]
fn t1_5_numeric_descent_is_accepted() {
    let src = "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)
";
    let dag = cached_compile_to_dag(src, "t1_5_numeric_descent.v3");
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
    cached_compile_to_dag(src, "t1_5_list_descent.v3");
}

#[test]
fn t1_5_missing_descent_is_rejected() {
    let dag = compile_semantic_fixture(
        "fn diverge(x: Int) -> Int = diverge(x)",
        "t1_5_no_descent.v3",
    );
    let bind = bind_named(&dag, "diverge");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "non-decreasing recursion must fail closed"
    );
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. }
                if name.contains("cannot prove recursion in `diverge` terminates") =>
            {
                Some(diag)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected descent-proof diagnostic, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    assert!(
        !diag.correction().description().is_empty(),
        "termination diagnostic should carry a named correction"
    );
}

#[test]
fn t2_4_option_like_values_require_a_match() {
    let src = "\
type Maybe<T> = Some(T) | None
fn unwrap_or_zero(m: Maybe<Int>) -> Int = match m { Some(value) => value, None => 0 }
";
    cached_compile_to_dag(src, "t2_4_match_required.v3");
}

#[test]
fn t2_4_option_like_values_without_match_are_rejected() {
    let src = "\
type Maybe<T> = Some(T) | None
fn bad(m: Maybe<Int>) -> Int = m
";
    let dag = compile_semantic_fixture(src, "t2_4_missing_match.v3");
    let bind = bind_named(&dag, "bad");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "using an option-like value without matching must fail closed"
    );
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. } if name.contains("declared signature") => {
                Some(diag)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a declared-signature diagnostic, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    assert!(
        !diag.correction().description().is_empty(),
        "declared-signature diagnostic should carry a named correction"
    );
}

#[test]
fn t2_4_no_force_unwrap_primitive_exists() {
    let src = "\
type Maybe<T> = Some(T) | None
fn bad(m: Maybe<Int>) -> Int = unwrap(m)
";
    let dag = compile_semantic_fixture(src, "t2_4_no_unwrap.v3");
    let bind = bind_named(&dag, "bad");
    assert!(
        matches!(dag.port(bind.value).state(), PortState::Unresolved),
        "unknown unwrap primitive must fail closed"
    );
    let diag = dag
        .diagnostics()
        .iter()
        .find_map(|(_, diag)| match diag {
            Diagnostic::ResolveError { name, .. } if name == "unwrap" => Some(diag),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "expected unresolved callable diagnostic for `unwrap`, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        });
    assert!(
        !diag.correction().description().is_empty(),
        "unresolved callable diagnostic should carry a named correction"
    );
}

#[test]
fn t2_4_unresolved_calls_still_lower_argument_diagnostics() {
    let src = "\
type Maybe<T> = Some(T) | None
fn bad(m: Maybe<Int>) -> Int = unknown(unwrap(m))
";
    let dag = compile_semantic_fixture(src, "t2_4_unresolved_call_keeps_arg_diags.v3");
    let diagnostics = dag.diagnostics().iter().map(|(_, d)| d).collect::<Vec<_>>();
    assert!(
        diagnostics
            .iter()
            .any(|diag| matches!(diag, Diagnostic::ResolveError { name, .. } if name == "unknown")),
        "outer unresolved call diagnostic should be preserved; got {diagnostics:?}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|diag| matches!(diag, Diagnostic::ResolveError { name, .. } if name == "unwrap")),
        "argument diagnostics must still lower under unresolved calls; got {diagnostics:?}"
    );
}

#[test]
fn kf_5_bounded_fold_compiles_on_supported_primitives() {
    let src = "\
let total: Int = fold(cons(1, cons(2, singleton(3))), 0, |acc, x| acc + x)
";
    cached_compile_to_dag(src, "kf_5_fold.v3");
}

#[test]
fn kf_5_unbounded_zero_arg_recursion_is_rejected() {
    let dag = compile_semantic_fixture("fn endless() -> Int = endless()", "kf_5_unbounded.v3");
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
    let flat = cached_compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
        "lens_cost_flat.v3",
    );
    let nested = cached_compile_to_dag(
        "let total: Int = fold(map(singleton(1), |x| x + 1), 0, |acc, x| acc + x)",
        "lens_cost_nested.v3",
    );

    let flat_cost = bind_cost(&flat, "total");
    let nested_cost = bind_cost(&nested, "total");

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
    let literal =
        cached_compile_to_dag("fn constant(n: Int) -> Int = 0", "lens_cost_literal_fn.v3");
    let recursive = cached_compile_to_dag(
        "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)
",
        "lens_cost_recursive_fn.v3",
    );

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
    let cheaper = cached_compile_to_dag(
        "let r: Int = if 1 > 0 then 20 + 30 else 40 + 50 + 60",
        "lens_cost_branch_cheaper.v3",
    );
    let pricier = cached_compile_to_dag(
        "let r: Int = if 1 > 0 then 20 + 30 + 40 + 50 else 60 + 70 + 80",
        "lens_cost_branch_pricier.v3",
    );

    let cheaper_cost = bind_cost(&cheaper, "r");
    let pricier_cost = bind_cost(&pricier, "r");

    assert!(
        pricier_cost > cheaper_cost,
        "making the most expensive branch path larger should increase branch cost: cheaper={cheaper_cost}, pricier={pricier_cost}"
    );
}

#[test]
fn kf_1_structural_list_operation_ordering_holds() {
    let singleton = cached_compile_to_dag("let xs = singleton(1)", "lens_cost_singleton.v3");
    let cons = cached_compile_to_dag("let xs = cons(1, singleton(2))", "lens_cost_cons.v3");
    let fold = cached_compile_to_dag(
        "let total: Int = fold(cons(1, singleton(2)), 0, |acc, x| acc + x)",
        "lens_cost_fold.v3",
    );
    let map_fold = cached_compile_to_dag(
        "let total: Int = fold(map(cons(1, singleton(2)), |x| x + 1), 0, |acc, x| acc + x)",
        "lens_cost_map_fold.v3",
    );

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
    let baseline = cached_compile_to_dag(
        "let r: Int = if 1 > 0 then 10 + 20 + 30 + 40 + 50 else 60 + 70",
        "lens_cost_branch_baseline.v3",
    );
    let extra_dead_work = cached_compile_to_dag(
        "let r: Int = if 1 > 0 then 10 + 20 + 30 + 40 + 50 else 60 + 70 + 80",
        "lens_cost_branch_dead_work.v3",
    );

    let baseline_cost = bind_cost(&baseline, "r");
    let extra_dead_work_cost = bind_cost(&extra_dead_work, "r");

    assert!(
        extra_dead_work_cost == baseline_cost,
        "growing only the non-max branch path should not change cost: baseline={baseline_cost}, extra_dead_work={extra_dead_work_cost}"
    );
}
