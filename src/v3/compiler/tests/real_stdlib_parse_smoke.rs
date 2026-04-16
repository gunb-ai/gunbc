// Smoke test: the real dsl/std/*.dag files parse clean through the v3
// tokenizer + parser. Does NOT check lowering or inference — just that
// the surface grammar accepts what the production .dag files contain.
//
// This test exists as a gate for the M1(2.6) bootstrap migration: once
// bootstrap.rs consumes these files via include_str!, this smoke test
// becomes redundant with the `Dag::new()` bootstrap path. Until then,
// it isolates "parser extensions work" from "bootstrap wiring works."

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

fn parse_file(source: &str, name: &str) {
    let tokens = tokenize_for_test(source, name)
        .unwrap_or_else(|diag| panic!("tokenize {name} failed: {diag:?}"));
    let _module = parse_for_test(&tokens, name)
        .unwrap_or_else(|diag| panic!("parse {name} failed: {diag:?}"));
}

#[test]
fn logic_dag_parses() {
    parse_file(include_str!("../../../../dsl/std/logic.dag"), "logic.dag");
}

#[test]
fn bit_dag_parses() {
    parse_file(include_str!("../../../../dsl/std/bit.dag"), "bit.dag");
}

#[test]
fn algebra_dag_parses() {
    parse_file(
        include_str!("../../../../dsl/std/algebra.dag"),
        "algebra.dag",
    );
}

#[test]
fn types_dag_parses() {
    parse_file(include_str!("../../../../dsl/std/types.dag"), "types.dag");
}

#[test]
fn integer_dag_parses() {
    parse_file(
        include_str!("../../../../dsl/std/integer.dag"),
        "integer.dag",
    );
}

#[test]
fn float_dag_parses() {
    parse_file(include_str!("../../../../dsl/std/float.dag"), "float.dag");
}

#[test]
fn string_type_dag_parses() {
    parse_file(
        include_str!("../../../../dsl/std/string_type.dag"),
        "string_type.dag",
    );
}

#[test]
fn list_dag_parses() {
    parse_file(include_str!("../../std/list.dag"), "list.dag");
}

#[test]
fn verification_dag_parses() {
    parse_file(
        include_str!("../../std/verification.dag"),
        "verification.dag",
    );
}
