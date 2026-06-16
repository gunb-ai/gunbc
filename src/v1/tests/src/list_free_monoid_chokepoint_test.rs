//! ctrl#1476 B1 — List=FreeMonoid Option-B chokepoint + detection test.
//!
//! List-consuming interpreter sites must route incoming values through
//! `expect_list` or `free_monoid_to_vec`. Direct `Value::List` matches on
//! operands are bypasses that break FreeMonoid<Cons/Empty> alias transparency.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

/// Supplementary source-pattern ratchet (known bypass snapshots). Semantic coverage is in the
/// runtime tests below; this catches reintroduction of specific pre-B1 `Value::List` operand arms.
#[test]
fn list_operations_do_not_match_value_list_on_incoming_operands() {
    let source = include_str!("../../stage0/src/v1_interpreter.rs");
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

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "three_len") {
        Ok(Value::Int(3)) => {}
        other => panic!("expected Int(3) from list literal .length(), got {other:?}"),
    }
}

#[test]
fn freemonoid_cons_chain_length_routes_through_chokepoint() {
    // Declare the FreeMonoid Empty/Cons coproduct inline: the v2 test harness only
    // indexes src/v1 + dsl, so v2.std.algebra is unreachable here. The interpreter's
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

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "cons_len") {
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
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "has_substring") {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected Bool(true) from \"abcd\".contains(\"bc\"), got {other:?}"),
    }
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "lacks_substring") {
        Ok(Value::Bool(false)) => {}
        other => panic!("expected Bool(false) from \"abcd\".contains(\"xy\"), got {other:?}"),
    }
}

/// `.contains` membership over a FreeMonoid Cons/Empty chain must route through the
/// chokepoint (free_monoid_to_vec), not the native-`Value::List` arm. A regression that
/// matched the receiver as a bare `Value::List` would fail `expect_list` on the Cons-chain
/// alias and error "contains not supported"; this pins element membership instead.
///
/// Distinct from `string_contains_multichar_substring_not_char_membership` (Str-substring
/// path) and `freemonoid_cons_chain_length_routes_through_chokepoint` (length, not membership):
/// it is the contains × FreeMonoid intersection that lens analysis (coverage.dag) relies on.
#[test]
fn contains_membership_over_freemonoid_cons_chain_routes_through_chokepoint() {
    // Inline Empty/Cons coproduct: the chokepoint matches the variant *names*, so this
    // exercises the identical FreeMonoid<->List bridge as v2.std.algebra (ctrl#1476 B1).
    let src = r#"module test.fm_cons_contains
type IntList = Empty | Cons { head: Int, tail: IntList }
fn cons_three() -> IntList {
  Cons { head: 1, tail: Cons { head: 2, tail: Cons { head: 3, tail: Empty } } }
}
fn member_present() -> Bool {
  cons_three().contains(2)
}
fn member_absent() -> Bool {
  cons_three().contains(9)
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    // Discriminating: a bare-`Value::List` regression errors on the Cons-chain alias.
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "member_present") {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected Bool(true) from Cons chain .contains(2), got {other:?}"),
    }
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "member_absent") {
        Ok(Value::Bool(false)) => {}
        other => panic!("expected Bool(false) from Cons chain .contains(9), got {other:?}"),
    }
}

/// BinOp::Add list concat must match .append: Str operand stays one element.
#[test]
fn list_add_str_operand_stays_single_element() {
    let src = r#"module test.add_str
fn two_elements() -> Int {
  let xs = [1]
  (xs + "ab").length()
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "two_elements") {
        Ok(Value::Int(2)) => {}
        other => panic!("expected Int(2) from [1] + \"ab\", got {other:?}"),
    }
}

/// List append must not explode a Str argument into char elements via the chokepoint.
#[test]
fn list_append_str_arg_stays_single_element() {
    let src = r#"module test.append_str
fn two_elements() -> Int {
  let xs = [1]
  xs.append("ab").length()
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "two_elements") {
        Ok(Value::Int(2)) => {}
        other => panic!("expected Int(2) from [1].append(\"ab\"), got {other:?}"),
    }
}

/// flat_map flattening applies to Cons/List chains only; a returned Str is one element.
#[test]
fn flat_map_str_result_not_char_exploded() {
    let src = r#"module test.flat_map_str
fn one_str_element() -> Int {
  let xs = [1]
  xs.flat_map(fn(_x) { "ab" }).length()
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "one_str_element") {
        Ok(Value::Int(1)) => {}
        other => panic!(
            "expected Int(1) from flat_map returning \"ab\" (one Str element, not char-exploded), got {other:?}"
        ),
    }
}
