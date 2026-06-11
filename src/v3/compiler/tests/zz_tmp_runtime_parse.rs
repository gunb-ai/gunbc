use v3_compiler::{parse_for_test, tokenize_for_test};

const RUNTIME_DAG: &str = include_str!("../../../v4/std/runtime.dag");

#[test]
fn tmp_v4_runtime_dag_tokenizes_and_parses() {
    let tokens = tokenize_for_test(RUNTIME_DAG, "src/v4/std/runtime.dag").expect("tokenize");
    let module = parse_for_test(&tokens, "src/v4/std/runtime.dag").expect("parse");
    assert!(!module.items.is_empty());
}
