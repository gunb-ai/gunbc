// M1(3) — lens_unused_parameters integration receipts.
//
// Structural direct-Dag cases live in in-crate unit tests so they can
// use the crate-private builder surface without widening the public API.
// This file keeps receipts that still need real source parsing or
// lowering to reach the behavior under test. The in-crate unit tests
// cover direct-Dag structural walks; recursive source examples stay
// here because they are the lowering receipts for Loop.body reachability.

use v3_compiler::compile_to_dag;
use v3_compiler::lens_unused_parameters::{UnusedParametersConfig, UnusedParametersLens};

fn unused_parameter_indexes_for_source(source: &str, file: &str) -> Vec<usize> {
    let dag = compile_to_dag(source, file).expect("compiles");
    let lens = UnusedParametersLens::new(&dag);
    let mut indexes: Vec<_> = lens
        .query(&UnusedParametersConfig::default())
        .into_iter()
        .filter(|violation| violation.function_span.file == file)
        .map(|violation| violation.parameter_index)
        .collect();
    indexes.sort_unstable();
    indexes
}

#[test]
fn unused_params_canonical_target_blocked_on_parser_gaps() {
    let src = "\
fn content_upsert(content: String, path: String) -> { written: Bool } {
  let matches = content == \"\"
  { written: !matches }
}
";
    let result = compile_to_dag(src, "patterns.v3");

    assert!(
        result.is_err(),
        "v3 unexpectedly parsed content_upsert verbatim; flip this test to a positive assertion"
    );
}

#[test]
fn unused_params_catches_content_upsert_synthetic_equivalent() {
    assert_eq!(
        unused_parameter_indexes_for_source(
            "fn content_upsert(content: Int, path: Int) -> Int = content + 0",
            "patterns_synthetic.v3",
        ),
        vec![1],
        "the synthetic content_upsert shape should report only the ignored second parameter"
    );
}

#[test]
fn unused_params_descends_into_loop_body_for_recursive_calls() {
    let src = "fn count_down(n: Int, base: Int) -> Int = \
        if n == 0 then base else count_down(n - 1, base)";

    assert_eq!(
        unused_parameter_indexes_for_source(src, "recursive.v3"),
        Vec::<usize>::new(),
        "recursive source should keep both parameters reachable through the lowered Loop.body"
    );
}

#[test]
fn unused_params_loop_body_descent_finds_param_only_used_in_recursion() {
    let src = "fn count_down(n: Int, marker: Int) -> Int = \
        if n == 0 then 0 else count_down(n - 1, marker)";

    assert_eq!(
        unused_parameter_indexes_for_source(src, "loop_body_descent.v3"),
        Vec::<usize>::new(),
        "parameters referenced only through the lowered recursive case should still count as used"
    );
}
