// M1(3) — lens_unused_parameters acceptance tests.
//
// First lens shipped from `docs/lens-library-design.md`. Verifies
// the algorithm against minimal v3 source programs that exercise
// each branch of the lens's logic:
//
//   - A function that uses every parameter → empty result.
//   - A function with one unused parameter → one violation
//     pointing at the right parameter.
//   - A function whose body is a literal (ignores all params)
//     → all params reported.
//   - A value binding (no parameters) → skipped entirely, no
//     violations.
//
// The lens is pure-reader and config-driven; tests construct the
// minimum config + Dag pair and assert the lens output matches.

use v3_compiler::compile_to_dag;
use v3_compiler::lens_unused_parameters::{
    UnusedParametersConfig, UnusedParametersLens,
};

fn run_lens(source: &str) -> Vec<String> {
    let dag = compile_to_dag(source, "test.v3").expect("compiles");
    let lens = UnusedParametersLens::new(&dag);
    let violations = lens.query(&UnusedParametersConfig::default());
    // Render each violation as `<function-bind-id>:param[<idx>]`
    // so test assertions don't depend on raw NodeId / PortId
    // formatting.
    violations
        .iter()
        .map(|v| format!("{:?}:param[{}]", v.function, v.parameter_index))
        .collect()
}

#[test]
fn unused_params_empty_for_function_using_every_parameter() {
    // `fn add(a: Int, b: Int) -> Int = a + b`
    // Both parameters are wired into the body's Add transform.
    // Expected: zero violations.
    let violations = run_lens(
        "fn add(a: Int, b: Int) -> Int = a + b",
    );
    assert_eq!(
        violations,
        Vec::<String>::new(),
        "every-param-used function should have no violations, got: {violations:?}"
    );
}

#[test]
fn unused_params_reports_single_unused_parameter() {
    // `fn first(a: Int, b: Int) -> Int = a`
    // The body is just `a`; `b` is declared but never read.
    // Expected: one violation pointing at parameter index 1.
    let violations = run_lens(
        "fn first(a: Int, b: Int) -> Int = a",
    );
    assert_eq!(violations.len(), 1, "expected 1 violation, got: {violations:?}");
    assert!(
        violations[0].ends_with(":param[1]"),
        "expected violation on param index 1 (the `b` parameter), got: {violations:?}"
    );
}

#[test]
fn unused_params_reports_all_parameters_for_constant_body() {
    // `fn always_one(x: Int, y: Int, z: Int) -> Int = 1`
    // The body is a literal; none of the parameters are read.
    // Expected: three violations, one per parameter.
    let violations = run_lens(
        "fn always_one(x: Int, y: Int, z: Int) -> Int = 1",
    );
    assert_eq!(
        violations.len(),
        3,
        "expected 3 violations (all params unused), got: {violations:?}"
    );
    let mut indexes: Vec<&str> = violations.iter().map(|v| v.as_str()).collect();
    indexes.sort();
    assert!(
        indexes.iter().any(|v| v.ends_with(":param[0]")),
        "expected violation on param index 0, got: {indexes:?}"
    );
    assert!(
        indexes.iter().any(|v| v.ends_with(":param[1]")),
        "expected violation on param index 1, got: {indexes:?}"
    );
    assert!(
        indexes.iter().any(|v| v.ends_with(":param[2]")),
        "expected violation on param index 2, got: {indexes:?}"
    );
}

#[test]
fn unused_params_skips_value_bindings() {
    // `let x: Int = 1 + 2`
    // No parameters at all — value bindings are skipped by the
    // function-shape filter. Expected: zero violations.
    let violations = run_lens("let x: Int = 1 + 2");
    assert_eq!(
        violations,
        Vec::<String>::new(),
        "value bindings should be skipped (no params to report), got: {violations:?}"
    );
}

#[test]
fn unused_params_handles_branch_in_body() {
    // `fn pick(a: Int, b: Int) -> Int = if a > 0 then a else b`
    // The cond uses `a`, the then-arm uses `a`, the else-arm uses
    // `b`. All parameters reachable. Expected: zero violations.
    let violations = run_lens(
        "fn pick(a: Int, b: Int) -> Int = if a > 0 then a else b",
    );
    assert_eq!(
        violations,
        Vec::<String>::new(),
        "branch-using function should have no violations, got: {violations:?}"
    );
}

#[test]
fn unused_params_reports_unused_in_branch_body() {
    // `fn always_a(a: Int, b: Int) -> Int = if a > 0 then a else a`
    // The cond uses `a`, both arms use `a`. `b` is declared but
    // never read. Expected: one violation on param index 1.
    let violations = run_lens(
        "fn always_a(a: Int, b: Int) -> Int = if a > 0 then a else a",
    );
    assert_eq!(violations.len(), 1, "expected 1 violation, got: {violations:?}");
    assert!(
        violations[0].ends_with(":param[1]"),
        "expected violation on param index 1, got: {violations:?}"
    );
}
