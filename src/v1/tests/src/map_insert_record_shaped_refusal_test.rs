//! Inverted control for primitive-backed `map_insert`: a record-shaped Map TypeErrors.
//!
//! Floor Bool witnesses cannot enroll a throw (`ExpectedRedArm::RuntimeErrored`). This module
//! is the expecting-error harness: `run_in_context` of a record-shaped insert must be
//! `InterpError::TypeError` naming the HostRealizedSeam. The cost witness is the same fixture
//! family: n native inserts increment `map_insert_calls` by n (the wrap body never takes the
//! HAMT arm), and first-key lookup eval-steps at n=8 vs n=64 stay sublinear in n — citing
//! `std.primitives` `map_insert_contract` `CarrierSensitive { ephemeral_work: "n",
//! persistent_work: "log32 n" }`.
//!
//! Clippy `--all-targets` compiles this crate. `gunbc.rung_drop` `rust_unit_tests_off_the_merge_path`
//! means no required lane runs the assertions today; `cargo test -p v1-compiler-tests` does.

use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpError, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const PROBE: &str = r#"
module test.map_insert_record_shaped_refusal

import std.types { Map }
import v2.std.collection { empty_map, map_insert, map_lookup }
import v2.std.optional { Absent, Present, Optional }

fn record_shaped_insert() -> Map<String, String> {
  map_insert(m: Map { lookup: fn(_) { Absent } }, key: "k", value: "v")
}

fn insert_from(m: Map<Int, Int>, i: Int, n: Int) -> Map<Int, Int> {
  if i >= n {
    m
  } else {
    insert_from(m: map_insert(m, i, i), i: i + 1, n: n)
  }
}

fn map_of_size(n: Int) -> Map<Int, Int> {
  insert_from(m: empty_map(), i: 0, n: n)
}

fn map_size_8() -> Map<Int, Int> { map_of_size(n: 8) }
fn map_size_64() -> Map<Int, Int> { map_of_size(n: 64) }

fn lookup_zero(m: Map<Int, Int>) -> Optional<Int> {
  map_lookup(m: m, key: 0)
}
"#;

fn assert_resolved(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .filter(|d| cli_run::compile_clean_diagnostic_is_hard(d))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "probe must resolve, got {msgs:?} (graph: {})",
        resolved.graph.is_some()
    );
}

fn with_probe_ctx<R>(body: impl FnOnce(&v1_interpreter::InterpContext) -> R) -> R {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots("test.dag", PROBE, &roots);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    body(&ctx)
}

#[test]
fn record_shaped_map_insert_is_located_type_error() {
    with_probe_ctx(|ctx| {
        match v1_interpreter::run_in_context(ctx, "record_shaped_insert", false) {
            Err(InterpError::TypeError { msg }) => {
                assert!(
                    msg.contains("map_insert") && msg.contains("HostRealizedSeam"),
                    "TypeError must locate the insert seam, got: {msg}"
                );
            }
            other => panic!(
                "record-shaped map_insert must TypeError, not succeed or drop the write; got {other:?}"
            ),
        }
    });
}

fn lookup_is_present_zero(ctx: &v1_interpreter::InterpContext, looked: &Value) {
    match looked {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "Present") => match ctx.field(fields.as_slice(), "value") {
            Some(Value::Int(0)) => {}
            other => panic!("Present value must be 0, got {other:?}"),
        },
        other => panic!("first-inserted key must be Present {{ value: 0 }}, got {other:?}"),
    }
}

#[test]
fn native_inserts_take_the_hamt_arm_and_first_key_lookup_is_sublinear() {
    let (calls8, lookup_steps_8) = with_probe_ctx(|ctx| {
        let map8 = v1_interpreter::run_in_context(ctx, "map_size_8", false).expect("map_size_8");
        let calls = ctx.mutation_counters_snapshot().map_insert_calls;
        let before = v1_interpreter::evaluator_steps();
        let looked = v1_interpreter::run_in_context_with_args(
            ctx,
            "lookup_zero",
            &[(Some("m".to_string()), map8)],
            false,
        )
        .expect("lookup_zero n=8");
        lookup_is_present_zero(ctx, &looked);
        (
            calls,
            v1_interpreter::evaluator_steps().saturating_sub(before),
        )
    });
    assert_eq!(
        calls8, 8,
        "closure-chain wrap never increments map_insert_calls; HAMT insert must fire once per key"
    );

    let (calls64, lookup_steps_64) = with_probe_ctx(|ctx| {
        let map64 = v1_interpreter::run_in_context(ctx, "map_size_64", false).expect("map_size_64");
        let calls = ctx.mutation_counters_snapshot().map_insert_calls;
        let before = v1_interpreter::evaluator_steps();
        let looked = v1_interpreter::run_in_context_with_args(
            ctx,
            "lookup_zero",
            &[(Some("m".to_string()), map64)],
            false,
        )
        .expect("lookup_zero n=64");
        lookup_is_present_zero(ctx, &looked);
        (
            calls,
            v1_interpreter::evaluator_steps().saturating_sub(before),
        )
    });
    assert_eq!(
        calls64, 64,
        "HAMT insert count must track n; wrap-body fallthrough would stay near 0"
    );

    // 8× keys. A wrap-chain lookup of the earliest key is Θ(n). HAMT lookup is log32 n
    // (`map_insert_contract` persistent_work). Refuse linear growth.
    assert!(
        lookup_steps_64
            < lookup_steps_8
                .saturating_mul(4)
                .max(lookup_steps_8.saturating_add(64)),
        "first-key lookup steps must not scale with n like a wrap chain \
         (n=8 steps={lookup_steps_8}, n=64 steps={lookup_steps_64})"
    );
}
