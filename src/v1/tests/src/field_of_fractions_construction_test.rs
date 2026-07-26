use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

// Discriminating witness for the §7.2 FieldOfFractions model grounding (sharp-bee-290
// sign-off): `FieldOfFractions<R> = { num: R, denom: R }` is now a real 2-field record at
// its single authority (dag/std/algebra.dag). Unlike GroupCompletion (#7197), this is
// deliberately negative-space: FieldOfFractions has no native Rust scalar checkpoint to
// collapse into (Rational has no lossless native representation — an f64 collapse would be
// dishonest, per the design doc), so `eval_record_lit` must NOT special-case it. A plain
// `FieldOfFractions { num, denom }` construction with native `Value::Int` fields must stay
// a boxed `Value::Record` carrying both fields untouched. A regression that special-cases
// FieldOfFractions into any native collapse (silently lossy or otherwise) fails this
// witness; a regression that leaves the type hollow (no `num`/`denom` fields at all) fails
// to resolve at all, which this witness's `assert_resolved` also guards.
const RECEIPTS_SOURCE: &str = r#"
module test.field_of_fractions_construction

import std.algebra { FieldOfFractions }

fn one_half() -> FieldOfFractions<Int> { FieldOfFractions { num: 1, denom: 2 } }
fn three_quarters() -> FieldOfFractions<Int> { FieldOfFractions { num: 3, denom: 4 } }
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
fn field_of_fractions_pair_stays_boxed_record_not_native_collapse() {
    with_receipts_ctx(|ctx| {
        for (f, expected_num, expected_denom) in
            [("one_half", 1i64, 2i64), ("three_quarters", 3i64, 4i64)]
        {
            match v1_interpreter::run_in_context(ctx, f, false) {
                Ok(Value::Record { type_name, fields }) => {
                    assert!(
                        ctx.sym_eq(type_name, "FieldOfFractions"),
                        "{f}: expected type_name FieldOfFractions, got {}",
                        ctx.resolve(type_name)
                    );
                    match ctx.field(&fields, "num") {
                        Some(Value::Int(n)) if *n == expected_num => {}
                        other => panic!("{f}: num field mismatch, got {other:?}"),
                    }
                    match ctx.field(&fields, "denom") {
                        Some(Value::Int(n)) if *n == expected_denom => {}
                        other => panic!("{f}: denom field mismatch, got {other:?}"),
                    }
                }
                other => panic!(
                    "{f}: expected a boxed Value::Record{{num, denom}} — FieldOfFractions has \
                     no native Rust scalar to collapse into, so a regression that special-cases \
                     it into any native Value variant (Int, Float, or otherwise) surfaces here; \
                     got {other:?}"
                ),
            }
        }
    });
}
