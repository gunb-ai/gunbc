//! Phase 1 — hand-Rust Tier-3 mirror timing (C1 perf-budget worker toward
//! `tier3_mirror_dissolution_perf_within_budget`).
//!
//! Maps to the remaining mirror dissolution slices in
//! `docs/briefs/r3-pb-tier3-perf-budget-worker.md` (computation /
//! induction / effect-carrier). The termination mirror bench retired with
//! `tier3_termination_mirror_dissolved`.
//!
//! **Frozen baseline** (`tier3_baseline.json`, deliverable 0c): not committed by
//! this skeleton. Capture median + p99 (ns) per group on the canonical CI
//! machine after this bench is stable, then commit JSON per the worker brief.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use v3_compiler::dag::{
    lower_call_pattern, positive_amount_from_i64, positive_descent_count, type_iteration_dimension,
    CallPattern,
};
use v3_compiler::dag::{EffectShape, IdempotentShape, OperationEffect, WorkflowEffect};
use v3_compiler::lane2_workflow_idempotency_report;

fn bench_computation_mirror(c: &mut Criterion) {
    let steps = positive_amount_from_i64(32).expect("fixture Peano depth");
    c.bench_function("tier3_computation_positive_descent_count", |bencher| {
        bencher.iter(|| black_box(positive_descent_count(black_box(&steps))));
    });

    c.bench_function("tier3_computation_lower_same_argument_call", |bencher| {
        bencher.iter(|| {
            black_box(lower_call_pattern(black_box(CallPattern::SameArgumentCall)));
        });
    });
}

fn bench_induction_mirror(c: &mut Criterion) {
    // Public projection on the induction mirror; unknown types fail closed.
    c.bench_function("tier3_induction_type_iteration_dimension_miss", |bencher| {
        bencher.iter(|| {
            black_box(type_iteration_dimension(black_box(
                "__tier3_bench_unknown__",
            )));
        });
    });
}

fn bench_effect_carrier_mirror(c: &mut Criterion) {
    let read_op = OperationEffect {
        operation_name: "read".into(),
        shape: EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
    };
    let workflow = WorkflowEffect::LinearEffect {
        ops: vec![
            read_op.clone(),
            read_op.clone(),
            read_op.clone(),
            read_op.clone(),
        ],
    };
    c.bench_function("tier3_effects_lane2_linear_read_chain", |bencher| {
        bencher.iter(|| black_box(lane2_workflow_idempotency_report(black_box(&workflow))));
    });
}

criterion_group!(
    tier3_mirror_phase1,
    bench_computation_mirror,
    bench_induction_mirror,
    bench_effect_carrier_mirror
);
criterion_main!(tier3_mirror_phase1);
