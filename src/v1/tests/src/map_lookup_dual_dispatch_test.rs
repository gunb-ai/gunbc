//! ctrl#1476 B6 — Map lookup Option-C dual-dispatch chokepoint + detection test.
//!
//! Map key-probe sites must route through `raw_map_lookup`, which dispatches both
//! native `Value::Map` and record-form `Map { lookup: fn }`. Direct `Value::Map`
//! key probes outside the chokepoint break Option-C alias transparency.

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

/// Detection: red if any map key-probe bypasses the Option-C chokepoint.
#[test]
fn map_lookup_operations_do_not_probe_value_map_outside_chokepoint() {
    let source = include_str!("../../stage0/src/v1_interpreter.rs");
    let forbidden = [
        "(Value::Map(m), Value::Str(k)) => Ok(m.get(k.as_str())",
        "(Value::Map(m), [Value::Str(k)]) =>",
        "Value::Map(m) => {\n                let key = expect_str(args.first(), \"get\")?;\n                Ok(m.get(&key)",
        "[Value::Map(m), Value::Str(k)] => {\n                Ok(Some(m.get(k.as_str())",
        "Value::Map(m) => Ok(m.get(field).cloned().unwrap_or(Value::Null))",
    ];
    for needle in forbidden {
        assert!(
            !source.contains(needle),
            "Map lookup bypass: native Value::Map key-probe outside raw_map_lookup.\n\
             forbidden pattern:\n{needle}\n\
             Route map key probes through raw_map_lookup (ctrl#1476 B6 Option-C)."
        );
    }
}

#[test]
fn map_get_method_routes_through_dual_dispatch_chokepoint() {
    let src = r#"module test.map_get_method
fn found() -> Int {
  let m = empty_map() |> map_insert("a", 10)
  m |> get("a")
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "found") {
        Ok(Value::Int(10)) => {}
        other => panic!("expected Int(10) from map get, got {other:?}"),
    }
}

#[test]
fn map_index_routes_through_dual_dispatch_chokepoint() {
    let src = r#"module test.map_index
fn read_b() -> Int {
  let m = empty_map() |> map_insert("b", 42)
  m["b"]
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "read_b") {
        Ok(Value::Int(42)) => {}
        other => panic!("expected Int(42) from map index, got {other:?}"),
    }
}

#[test]
fn lookup_builtin_routes_through_dual_dispatch_chokepoint() {
    let src = r#"module test.lookup_builtin
fn probe() -> Int {
  let m = empty_map() |> map_insert("k", 7)
  lookup(m, "k")
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v1_interpreter::run(graph, resolved.source_indices.clone(), "probe") {
        Ok(Value::Int(7)) => {}
        other => panic!("expected Int(7) from lookup builtin, got {other:?}"),
    }
}

// NB: the native-map-miss -> `Witness` `Violates` bridge (runtime completeness for the
// `Map.lookup: fn(K) -> Witness<V>` contract) is regression-tested at the v2 layer in
// src/v2/compiler/manual/map_lookup_miss_witness_violates_test.dag — the bootstrap
// typechecker projects a raw `.lookup` result as Option (Some/None) unless it flows
// through a `-> Witness` annotated wrapper, which the std `Map` contract provides but a
// bare v2-harness fixture (no v2.std) cannot.
