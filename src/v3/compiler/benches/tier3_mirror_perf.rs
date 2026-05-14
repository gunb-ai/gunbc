//! Phase 1 — hand-Rust Tier-3 mirror timing (C1 perf-budget worker toward
//! `tier3_mirror_dissolution_perf_within_budget`).
//!
//! Maps 1:1 to the four mirror dissolution slices in
//! `docs/briefs/r3-pb-tier3-perf-budget-worker.md` (active Phase-1 mirrors:
//! computation / effect-carrier; termination and induction are retired). See that brief for thresholds (≤2× median,
//! ≤5× p99) and Phase 1 / Phase 2 split.
//!
//! **Frozen baseline** (`tier3_baseline.json` alongside this bench): Phase-1
//! committed median + p99 (ns) per budgeted `bench_function` (regen via
//! ``scripts/aggregate_tier3_baseline.py`` + procedure at
//! `docs/audit/c1-tier3-baseline-capture-procedure.md`). Ubicloud canonical
//! recapture uses `.github/workflows/tier3-baseline-capture.yml` (diff against
//! the committed JSON; baseline updates land only through PR).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use v3_compiler::dag::WorkflowEffect;
use v3_compiler::dag::{
    lower_call_pattern, positive_amount_from_i64, positive_descent_count, CallPattern, CallableRef,
    HttpMethodScalar, InputField, Operation, PathTemplate, RestEndpointBinding,
};
use v3_compiler::lane2_workflow_idempotency_report;
use v3_compiler::Dag;

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

fn bench_effect_carrier_mirror(c: &mut Criterion) {
    let dag = Dag::new();
    let callable = dag
        .declaration_by_name("get_method")
        .expect("bootstrap should provide get_method declaration")
        .id;
    let read_op = Operation {
        callable: CallableRef { decl: callable },
        inputs: std::collections::BTreeMap::<String, InputField>::new(),
        endpoint: RestEndpointBinding {
            method: HttpMethodScalar::Get,
            path: PathTemplate { tokens: vec![] },
        },
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
        bencher.iter(|| {
            black_box(lane2_workflow_idempotency_report(
                black_box(&dag),
                black_box(&workflow),
            ))
        });
    });
}

criterion_group!(
    tier3_mirror_phase1,
    bench_computation_mirror,
    bench_effect_carrier_mirror
);
criterion_main!(tier3_mirror_phase1);
