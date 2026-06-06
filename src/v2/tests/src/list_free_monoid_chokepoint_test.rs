//! ctrl#1476 B1 — List=FreeMonoid Option-B chokepoint + detection test.
//!
//! List-consuming interpreter sites must route incoming values through
//! `expect_list` or `free_monoid_to_vec`. Direct `Value::List` matches on
//! operands are bypasses that break FreeMonoid<Cons/Empty> alias transparency.

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

/// Detection: red if any list op bypasses the Option-B chokepoint.
#[test]
fn list_operations_do_not_match_value_list_on_incoming_operands() {
    let source = include_str!("../../stage0/src/v2_interpreter.rs");
    let forbidden = [
        "match collection {\n        Value::List(items)",
        "(Value::List(items), Value::Int(i))",
        "(Value::List(items), Value::Int(s), Value::Int(e))",
        "match &receiver {\n            Value::List(items) =>",
        "Value::List(items) => {\n                let target = args.first()",
        "Value::List(items) => {\n                let idx = expect_int",
        "[Value::List(items), item]",
        "[Value::List(a), Value::List(b)]",
        "Some(Value::List(items)) => Ok(Some(Value::Int(items.len()",
        "Some(Value::List(items)) => {\n                let mut r = items.to_vec()",
    ];
    for needle in forbidden {
        assert!(
            !source.contains(needle),
            "List=FreeMonoid bypass: incoming operand matched as Value::List outside chokepoint.\n\
             forbidden pattern:\n{needle}\n\
             Route list consumption through expect_list/free_monoid_to_vec (ctrl#1476 B1)."
        );
    }
}

#[test]
fn list_literal_length_routes_through_chokepoint() {
    let src = r#"module test.fm_len
fn three_len() -> Int {
  let xs = [1, 2, 3]
  xs.length()
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(graph, resolved.source_indices.clone(), "three_len") {
        Ok(Value::Int(3)) => {}
        other => panic!("expected Int(3) from list literal .length(), got {other:?}"),
    }
}

#[test]
fn freemonoid_cons_chain_length_routes_through_chokepoint() {
    // Declare the FreeMonoid Empty/Cons coproduct inline: the v2 test harness only
    // indexes src/v2 + dsl, so v4.std.algebra is unreachable here. The interpreter's
    // `free_monoid_to_vec` chokepoint matches the variant *names* "Empty"/"Cons", so an
    // inline declaration exercises the identical FreeMonoid<->List bridge (ctrl#1476 B1).
    let src = r#"module test.fm_cons_len
type IntList = Empty | Cons { head: Int, tail: IntList }
fn cons_three() -> IntList {
  Cons { head: 1, tail: Cons { head: 2, tail: Cons { head: 3, tail: Empty } } }
}
fn cons_len() -> Int {
  cons_three().length()
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(graph, resolved.source_indices.clone(), "cons_len") {
        Ok(Value::Int(3)) => {}
        other => panic!("expected Int(3) from Cons chain .length(), got {other:?}"),
    }
}

/// A String IS a FreeMonoid<Char>, but `.contains` on a String means *substring*
/// containment — the chokepoint must check the Str representation before the list path,
/// or a multi-char query is wrongly evaluated as char-list membership (returns false).
#[test]
fn string_contains_multichar_substring_not_char_membership() {
    let src = r#"module test.str_contains
fn has_substring() -> Bool {
  "abcd".contains("bc")
}
fn lacks_substring() -> Bool {
  "abcd".contains("xy")
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    // Discriminating: char-list membership would make a multi-char substring false.
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "has_substring") {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected Bool(true) from \"abcd\".contains(\"bc\"), got {other:?}"),
    }
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "lacks_substring") {
        Ok(Value::Bool(false)) => {}
        other => panic!("expected Bool(false) from \"abcd\".contains(\"xy\"), got {other:?}"),
    }
}

/// Slicing a String must return a substring (Str), not a char-list. The FreeMonoid
/// chokepoint would reroute the Str through the list arm and return Value::List of
/// one-char Strs; the dedicated Str arm must win.
#[test]
fn string_slice_returns_substring_not_char_list() {
    let src = r#"module test.str_slice
fn mid() -> String {
  "abcdef"[1..4]
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(graph, resolved.source_indices.clone(), "mid") {
        Ok(Value::Str(s)) => assert_eq!(s, "bcd"),
        other => panic!("expected Str(\"bcd\") from \"abcdef\"[1:4], got {other:?}"),
    }
}

/// Indexing a String must return a one-char Str via its dedicated arm, not a
/// char-list element via the chokepoint.
#[test]
fn string_index_returns_one_char_str() {
    let src = r#"module test.str_index
fn nth() -> String {
  "abcdef"[2]
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(graph, resolved.source_indices.clone(), "nth") {
        Ok(Value::Str(s)) => assert_eq!(s, "c"),
        other => panic!("expected Str(\"c\") from \"abcdef\"[2], got {other:?}"),
    }
}
