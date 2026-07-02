//! RESIDUAL after 5-test-migration (2026-07-02): 4 of the original 6 tests are
//! migrated to marker-discovered floor witnesses in
//! dsl/test/claim/map_lookup_dual_dispatch_witness_test.dag (method get, index
//! sugar, lookup builtin, std.graph adjacency smoke — a chokepoint perturbation
//! reds all four, proving the routing claim by execution).
//! The 2 tests below stay:
//! - map_lookup_operations_do_not_probe_value_map_outside_chokepoint: scans
//!   v1_interpreter.rs source text for forbidden bypass patterns — a pinned-harness
//!   fact, dissolving with the v1 interpreter.
//! - map_get_function_returns_optional_for_empty_list_hit: #[ignore]d on a
//!   pre-existing empty-list-literal inference bug (bucket=inference); migrate when
//!   the bug is fixed and the witness can be authored green.
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
#[ignore = "failing: empty list literal inference error 'expected type is not a collection'. Pre-existing (never run in CI under the 3-test allowlist), surfaced by the run-all widening #5427; fix as follow-up. bucket=inference"]
fn map_get_function_returns_optional_for_empty_list_hit() {
    let src = r#"module test.map_get_empty_list
fn hit() -> Bool {
  let m = empty_map() |> map_insert("a", [])
  match map_get(m, "a") {
    Present { value: ns } => ns == []
    Absent => false
  }
}

fn miss() -> Bool {
  let m = empty_map() |> map_insert("a", [])
  match map_get(m, "b") {
    Present { value: _ } => false
    Absent => true
  }
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    let si = resolved.source_indices.clone();

    match v1_interpreter::run(graph, si.clone(), "hit") {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected empty-list map_get hit, got {other:?}"),
    }
    match v1_interpreter::run(graph, si, "miss") {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected map_get miss -> Absent, got {other:?}"),
    }
}
