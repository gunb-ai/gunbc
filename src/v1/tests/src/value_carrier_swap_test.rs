//! ctrl#1533 phase 2 — persistent-carrier swap probes.
//!
//! The v2.std.value_carrier sharing/identity laws have executable witnesses
//! in src/v2/test/claim/std_grounding/value_carrier_laws.dag; those run at
//! small n. These probes cover what the witnesses cannot: collection sizes
//! past the carriers' inline-chunk thresholds (so HAMT/RRB tree nodes are
//! actually exercised), prior-version validity after derived updates (the
//! Driscoll et al. persistence property — the failure class a carrier swap
//! can introduce is in-place mutation observed through a shared handle), and
//! reference-identity non-observability checked from the host side.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

/// Past im-rc's 64-element inline chunks, so both carriers run their tree
/// (non-inline) code paths.
const SCALE: usize = 500;

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

fn run_bool(src: &str, entry: &str) {
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
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

/// Map equality is insert-order independent past the inline-chunk threshold:
/// the same entry set fold-built ascending and descending yields equal maps.
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

/// Overwrite is last-write-wins independent of scale: re-inserting every key
/// with a new value equals building the final entry set directly.
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

/// Persistence (Driscoll et al. 1989): a derived version leaves the prior
/// version valid. `extended`/`updated` are derived FROM `base`; `base` must
/// still equal an independently built structural twin afterwards. An
/// in-place-mutation bug through a shared carrier handle fails this.
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

/// Reference identity is not observable through equality (M-C law 1 stays
/// one-way): two separately evaluated runs build pointer-distinct values
/// that compare equal from the host side.
#[test]
fn equality_does_not_depend_on_reference_identity() {
    let nums = int_literal_list(1..=SCALE);
    let src = format!(
        r#"module test.carrier_identity
fn build() -> List<Int> {{ concat([{nums}], [0]) }}
"#
    );
    let sources = resolve_imports_transitively("test.dag", &src);
    let resolved = compile_to_resolved(Rc::new(sources));
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
