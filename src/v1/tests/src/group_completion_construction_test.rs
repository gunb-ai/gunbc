use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

// Discriminating witness for the §2/§3 GroupCompletion grounding (sharp-bee-290 sign-off,
// msg_6fc2ba88-549b-491e-9b6f-ab949539d682): `GroupCompletion<M> = { pos: M, neg: M }` is
// now a real 2-field record at its single authority (dag/std/algebra.dag), and
// `eval_record_lit` collapses a plain-record `GroupCompletion{pos, neg}` construction with
// native `Value::Int` fields directly to `Value::Int(pos - neg)` — mirroring #5428's
// Succ{prev} construction-side collapse — rather than building a boxed `Value::Record`.
// A regression that reintroduces a boxed record (or a wrong pair-to-int reduction) fails
// this witness; a regression that leaves the type hollow (no `pos`/`neg` fields at all)
// fails to resolve at all, which this witness's `assert_resolved` also guards.
const RECEIPTS_SOURCE: &str = r#"
module test.group_completion_construction

import v2.std.logic { Bool }
import std.algebra { GroupCompletion }

fn positive_pair() -> Int { GroupCompletion { pos: 5, neg: 2 } }
fn negative_pair() -> Int { GroupCompletion { pos: 1, neg: 4 } }
fn zero_pair() -> Int { GroupCompletion { pos: 3, neg: 3 } }

fn collapsed_pair_arithmetic() -> Bool {
  (GroupCompletion { pos: 5, neg: 2 }) + (GroupCompletion { pos: 1, neg: 4 }) == 0
}
"#;

fn assert_resolved(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "receipts source should resolve cleanly, got {:?} (graph present: {})",
        msgs,
        resolved.graph.is_some(),
    );
}

fn with_receipts_ctx<R>(body: impl FnOnce(&v1_interpreter::InterpContext) -> R) -> R {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources =
        resolve_imports_transitively_with_source_roots("test.dag", RECEIPTS_SOURCE, &roots);
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    body(&ctx)
}

#[test]
fn group_completion_pair_collapses_to_native_int() {
    with_receipts_ctx(|ctx| {
        for (f, expected) in [
            ("positive_pair", 3i64),
            ("negative_pair", -3i64),
            ("zero_pair", 0i64),
        ] {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Int(n)) if n == expected => {}
                other => panic!(
                    "{f}: expected native Value::Int({expected}) — a construction-side collapse \
                     regression (a boxed Value::Record, or a wrong pos-neg reduction) surfaces \
                     here; got {other:?}"
                ),
            }
        }
    });
}

#[test]
fn group_completion_pairs_combine_via_native_arithmetic() {
    with_receipts_ctx(|ctx| {
        match v1_interpreter::run_in_context(ctx, "collapsed_pair_arithmetic", false) {
            Ok(Value::Bool(true)) => {}
            other => panic!(
                "collapsed_pair_arithmetic: expected Bool(true) — both pairs collapse to \
                 native Value::Int (3 and -3) and combine via ordinary native addition; a \
                 boxed-record straddle here means the collapse did not fire; got {other:?}"
            ),
        }
    });
}
