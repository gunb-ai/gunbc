use std::rc::Rc;
use std::sync::Arc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

const SCALE: usize = 500;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

fn run_bool(src: &str, entry: &str) {
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    match v1_interpreter::run(graph, resolved.source_indices.clone(), entry) {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected Bool(true) from {entry}, got {other:?}"),
    }
}

fn int_literal_list(range: impl Iterator<Item = usize>) -> String {
    range.map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
}

#[test]
fn map_equality_insert_order_independent_at_scale() {
    let up = int_literal_list(1..=SCALE);
    let down = int_literal_list((1..=SCALE).rev());
    let src = format!(
        r#"module test.carrier_scale_map
fn order_independent() -> Bool {{
  let ascending = fold([{up}], init: empty_map(), f: fn(acc, x) {{ map_insert(acc, x, x) }})
  let descending = fold([{down}], init: empty_map(), f: fn(acc, x) {{ map_insert(acc, x, x) }})
  ascending == descending
}}
"#
    );
    run_bool(&src, "order_independent");
}

#[test]
fn map_overwrite_path_independent_at_scale() {
    let keys = int_literal_list(1..=SCALE);
    let src = format!(
        r#"module test.carrier_scale_overwrite
fn overwrite_wins() -> Bool {{
  let stale = fold([{keys}], init: empty_map(), f: fn(acc, x) {{ map_insert(acc, x, 0) }})
  let overwritten = fold([{keys}], init: stale, f: fn(acc, x) {{ map_insert(acc, x, x) }})
  let direct = fold([{keys}], init: empty_map(), f: fn(acc, x) {{ map_insert(acc, x, x) }})
  overwritten == direct
}}
"#
    );
    run_bool(&src, "overwrite_wins");
}

#[test]
fn prior_versions_survive_derived_updates_at_scale() {
    let nums = int_literal_list(1..=SCALE);
    let src = format!(
        r#"module test.carrier_persistence
fn prior_versions_valid() -> Bool {{
  let base_list = [{nums}]
  let extended = concat(base_list, [0])
  let base_map = fold([{nums}], init: empty_map(), f: fn(acc, x) {{ map_insert(acc, x, x) }})
  let updated = map_insert(base_map, 1, 0)
  let list_intact = base_list == [{nums}]
  let map_intact = base_map == fold([{nums}], init: empty_map(), f: fn(acc, x) {{ map_insert(acc, x, x) }})
  let derived_differ = !(extended == base_list) && !(updated == base_map)
  list_intact && map_intact && derived_differ
}}
"#
    );
    run_bool(&src, "prior_versions_valid");
}

#[test]
fn equality_does_not_depend_on_reference_identity() {
    let nums = int_literal_list(1..=SCALE);
    let src = format!(
        r#"module test.carrier_identity
fn build() -> List<Int> {{ concat([{nums}], [0]) }}
"#
    );
    let sources = resolve_imports_transitively("test.dag", &src);
    let resolved = compile_to_resolved(Arc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let first =
        v1_interpreter::run(graph, resolved.source_indices.clone(), "build").expect("first build");
    let second =
        v1_interpreter::run(graph, resolved.source_indices.clone(), "build").expect("second build");
    match (&first, &second) {
        (Value::List(a), Value::List(b)) => {
            assert!(!Rc::ptr_eq(a, b), "two runs must build distinct handles");
            assert_eq!(a.len(), SCALE + 1);
        }
        other => panic!("expected two Lists, got {other:?}"),
    }
    assert_eq!(first, second, "pointer-distinct twins must compare equal");
}
