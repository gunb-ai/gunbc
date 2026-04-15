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
use v3_compiler::Dag;

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

/// **Probe**: dump the lens output against v3's bootstrap-only
/// Dag. Documents the baseline: the lens reports zero violations
/// against the bootstrap because every bootstrap-loaded fn has
/// a block body (`ArrowBody::Unparsed`), and a block body
/// doesn't create a lowered BindNode for the body — there's no
/// sub-DAG for the lens to walk. The lens only sees expression-
/// body functions, and currently no bootstrap-loaded fn falls
/// into that category at v3 M1(3) parser scope.
///
/// **Why this test is non-empty.** It pins the baseline so that
/// if a future bootstrap addition brings in an expression-body
/// fn, the test starts asserting whatever the lens finds. This
/// is the "real code path" probe the user's review asked for —
/// not a synthetic test, but a check that the lens runs cleanly
/// against the live std/ load.
#[test]
fn unused_params_bootstrap_baseline_is_empty() {
    let dag = Dag::new();
    let lens = UnusedParametersLens::new(&dag);
    let violations = lens.query(&UnusedParametersConfig::default());
    assert_eq!(
        violations.len(),
        0,
        "bootstrap-only Dag should report zero unused parameters at v3 M1(3) scope; \
         every bootstrap fn has a block body that doesn't lower to a BindNode body. \
         If this test starts failing, a new bootstrap fn either has an unused \
         parameter (fix the std/ source) or v3's parser grew expression-body \
         support for std/ files (update this baseline). Got: {violations:?}"
    );
}

/// **Canonical target — v3 parser limitation receipt.**
///
/// The lens library spec (`docs/lens-library-design.md` §2.3)
/// names `content_upsert` in `dsl/std/patterns.dag:136-139` as
/// the known concrete finding the lens was written to catch.
/// That function uses three parser features v3 doesn't support
/// at M1(3) scope:
///
///   1. Anonymous record return types: `-> { written: Bool }`
///   2. Block-body functions (multi-statement bodies)
///   3. Record literals in user-code expression position
///
/// All three are class-5 gap follow-ups (#3 / #4 / etc.). Until
/// they land, v3 cannot load `patterns.dag` and the lens cannot
/// reach the literal `content_upsert` declaration.
///
/// This test compiles the literal source and asserts the parse
/// fails. When v3 grows the missing parser features, this test
/// flips to a positive assertion that the lens catches
/// `content_upsert`'s `path` parameter — the test becomes the
/// empirical proof that the canonical target is reached.
#[test]
fn unused_params_canonical_target_blocked_on_parser_gaps() {
    let src = "\
fn content_upsert(content: String, path: String) -> { written: Bool } {
  let matches = content == \"\"
  { written: !matches }
}
";
    let result = v3_compiler::compile_to_dag(src, "patterns.v3");
    // We expect parse failure today. The error variant doesn't
    // matter — any error proves v3 can't reach content_upsert
    // verbatim. When the parser features land, this assertion
    // flips and the test should compile cleanly + run the lens
    // + assert the path parameter is reported as unused.
    assert!(
        result.is_err(),
        "v3 unexpectedly parsed content_upsert verbatim — flip this test \
         to a positive assertion (compile, run lens, assert path param \
         in violations)"
    );
}

/// **Canonical target — synthetic equivalent.**
///
/// Since v3 can't parse `content_upsert` verbatim (see the test
/// above), this test exercises the same SHAPE the lens is
/// supposed to catch: a function with two parameters, body
/// reads only one, the other is silently ignored. The function
/// is named `content_upsert` to match the spec's canonical
/// target by intent even though the body is rewritten in
/// v3-parseable syntax.
///
/// **Why this is the right empirical proof.** The lens's job is
/// to flag "function declares param P but body never reads P."
/// Whether the body is a block, an anonymous record literal, a
/// nested let-statement, or any other v3 surface form is
/// irrelevant to the lens — the lens reads BindNode.params and
/// walks the body sub-DAG via `produced_by` edges. A
/// v3-parseable function with the same dataflow shape produces
/// the same lens output as the literal `content_upsert` would
/// once the parser catches up. The test is the empirical proof
/// the lens is doing real work, not just passing minimal
/// synthetic shapes.
#[test]
fn unused_params_catches_content_upsert_synthetic_equivalent() {
    // Same shape as patterns.dag::content_upsert: two String
    // params, body uses `content` and ignores `path`. Rendered
    // as an Int comparison because v3's M1(3) parser accepts
    // arithmetic on Int but not Bool negation in expression
    // position. Same dataflow shape: one used param, one unused.
    let src = "fn content_upsert(content: Int, path: Int) -> Int = content + 0";
    let dag = compile_to_dag(src, "patterns_synthetic.v3").expect("compiles");
    let lens = UnusedParametersLens::new(&dag);
    let violations = lens.query(&UnusedParametersConfig::default());

    // Find any violation in the test source (excluding bootstrap
    // findings, which the baseline test pinned at zero anyway).
    let user_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.function_span.file == "patterns_synthetic.v3")
        .collect();
    assert_eq!(
        user_violations.len(),
        1,
        "expected exactly one unused-param violation in the test source, got: {user_violations:?}"
    );
    assert_eq!(
        user_violations[0].parameter_index, 1,
        "expected violation on parameter index 1 (the `path` parameter), got: {:?}",
        user_violations[0]
    );
}

/// **Followup #2 regression test — Loop body descent.**
///
/// v3 lowers recursive functions to a `Loop` whose `body` is a
/// NodeId pointing at the recursive call sub-DAG. Before the
/// followup fix, the lens walker only visited `Loop.source` and
/// `Loop.init` and never descended into `Loop.body`. Parameters
/// used only inside the recursive body were falsely reported
/// unused.
///
/// This test compiles a recursive function whose body uses
/// every parameter from inside the recursive call, then asserts
/// the lens returns zero violations (every parameter is reached
/// via the loop body descent). Without the fix, this test would
/// fail with parameters falsely reported.
///
/// **Why `n - 1` instead of `n - step`.** v3's termination
/// check requires the first argument of every recursive call to
/// be `<param> - <positive int literal>` so the analysis can
/// prove the recursion bottoms out. A variable decrement (`n -
/// step`) doesn't satisfy that check and the function fails to
/// compile. The lens test uses a literal decrement to keep the
/// function compileable; the lens behavior we're testing
/// (Loop.body descent) is independent of the termination
/// check's strictness.
#[test]
fn unused_params_descends_into_loop_body_for_recursive_calls() {
    // `fn count_down(n: Int, base: Int) -> Int = ...`
    //
    // Both parameters are used: `n` is the recursive descent
    // counter (used in cond + first arg of recursive call),
    // `base` is the base-case value (used in then-branch + the
    // second arg of the recursive call). The recursive call
    // `count_down(n - 1, base)` lives inside the loop body, so
    // the walker must descend into Loop.body to see both
    // parameter reads from the recursive case.
    let src = "fn count_down(n: Int, base: Int) -> Int = \
        if n == 0 then base else count_down(n - 1, base)";
    let dag = compile_to_dag(src, "recursive.v3").expect("compiles");
    let lens = UnusedParametersLens::new(&dag);
    let violations = lens.query(&UnusedParametersConfig::default());

    let user_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.function_span.file == "recursive.v3")
        .collect();
    assert!(
        user_violations.is_empty(),
        "recursive fn using every parameter inside the loop body should \
         have no violations, got: {user_violations:?}"
    );
}

/// **Followup #2 — exact bug being fixed.**
///
/// A recursive function where one parameter is ONLY used inside
/// the recursive call (not in the base case or the conditional)
/// should still be reported as USED. Before the loop-body
/// descent fix, the walker missed the body's reads and falsely
/// flagged the param as unused. After the fix, the walk reaches
/// the param via the body descent.
#[test]
fn unused_params_loop_body_descent_finds_param_only_used_in_recursion() {
    // `marker` only appears inside the recursive call. The base
    // case (n == 0 → 0) doesn't read it; the conditional
    // doesn't read it; only the recursive case
    // (count_down(n - 1, marker)) does. Before the Loop.body
    // descent fix, the walker stopped at l.source/l.init and
    // missed this read — `marker` was falsely flagged unused.
    let src = "fn count_down(n: Int, marker: Int) -> Int = \
        if n == 0 then 0 else count_down(n - 1, marker)";
    let dag = compile_to_dag(src, "loop_body_descent.v3").expect("compiles");
    let lens = UnusedParametersLens::new(&dag);
    let violations = lens.query(&UnusedParametersConfig::default());

    let in_test: Vec<_> = violations
        .iter()
        .filter(|v| v.function_span.file == "loop_body_descent.v3")
        .collect();
    assert!(
        in_test.is_empty(),
        "lens should walk Loop.body to find params used only in the \
         recursive call; pre-fix the walk stopped at l.source/l.init \
         and missed body reads, falsely flagging `marker` as unused. \
         Got: {in_test:?}"
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
