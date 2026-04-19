//! Lane 2 Stage 2e / DB-20 — `ParallelEffect` parallel composition safety.
//!
//! Derivation coverage (merge gate: pairwise commutativity from `OperationEffect` shapes):
//! - **Commute (green):** `parallel_read_only_branches_commute` — cross-branch `ReadEffect` only;
//!   `parallel_same_key_path_upserts_commute` — same `KeySource` **and** same `operation_name`
//!   on upserts (distinct op names are not proven same write); `parallel_same_key_distinct_upsert_names_do_not_commute` — red.
//! - **Non-commute / fail-closed (red):** `parallel_different_path_param_names_not_proven_commute` —
//!   distinct `PathParam` names (not a disjointness proof); `parallel_read_vs_upsert_does_not_commute` —
//!   read vs write shape clash.
//! - **Breaking op:** `parallel_append_in_branch_is_broken_by` — `BrokenBy` before pairwise check.
//!
//! **CI / Layer 1:** integration tests must not call `compile_to_dag` per `#[test]` for the same
//! fixture — cold runners multiply that into minutes (see #543 / v3 full-suite wall-clock ratchet).
//! One `OnceLock` compile + `Dag::clone` per test keeps `try_register_lane2_workflow_effect` isolated.

use std::sync::OnceLock;

use v3_compiler::analyze_parallelism;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, BreakingShape, CompositionVerdict, EffectShape, IdempotentShape, KeySource,
    NonEmptyList, NonSingletonList, OperationEffect, ParallelismUnsupportedKind, WorkflowEffect,
    WorkflowParallelismReport,
};
use v3_compiler::Dag;
use v3_compiler::NodeId;

/// Shared bootstrap: single `compile_to_dag("let _ = 1", …)` for every test that mutates the DAG.
/// Each test uses [`Dag::clone`] so `try_register_lane2_workflow_effect` does not cross-test pollute.
static SHARED_FIXTURE_DAG: OnceLock<Dag> = OnceLock::new();

fn shared_fixture_dag() -> Dag {
    SHARED_FIXTURE_DAG
        .get_or_init(|| {
            compile_to_dag("let _ = 1", "lane2_stage_2e_fixture.v3")
                .expect("compile trivial fixture")
        })
        .clone()
}

fn lane2_anchor(dag: &Dag) -> NodeId {
    dag.nodes()
        .iter()
        .find(|b| matches!(b, Behavior::Value(_) | Behavior::Bind(_)))
        .expect("compile fixture should include a Value or Bind for lane2 staging")
        .id()
}

fn op(name: &str, shape: EffectShape) -> OperationEffect {
    OperationEffect {
        operation_name: name.to_string(),
        shape,
    }
}

fn read(name: &str) -> OperationEffect {
    op(name, EffectShape::IsIdempotent(IdempotentShape::ReadEffect))
}

#[test]
fn parallel_requires_at_least_two_branches_type_level() {
    let linear = WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![read("r")]).unwrap(),
    };
    assert!(
        NonSingletonList::from_vec(vec![Box::new(linear.clone())]).is_none(),
        "singleton branch list is not a NonSingletonList"
    );
}

#[test]
fn parallel_read_only_branches_commute() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![read("a"), read("b")]).unwrap(),
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![read("c")]).unwrap(),
            }),
        ])
        .unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    assert!(matches!(
        r,
        WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}

#[test]
fn parallel_same_key_path_upserts_commute() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let key = KeySource::PathParam { param: "id".into() };
    // Same route identity: matching operation_name + KeySource is the v1 witness for upsert commute.
    let upsert = op(
        "put_item",
        EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
            key_source: key.clone(),
        }),
    );
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![upsert.clone()]).unwrap(),
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![upsert]).unwrap(),
            }),
        ])
        .unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    assert!(matches!(
        r,
        WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}

#[test]
fn parallel_same_key_distinct_upsert_names_do_not_commute() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let key = KeySource::PathParam { param: "id".into() };
    let upsert = |name: &str| {
        op(
            name,
            EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
                key_source: key.clone(),
            }),
        )
    };
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![upsert("put_a")]).unwrap(),
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![upsert("put_b")]).unwrap(),
            }),
        ])
        .unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelismUnsupported(d) = r else {
        panic!("expected ParallelismUnsupported — same KeySource but distinct operation_name is not a same-write proof");
    };
    assert_eq!(d.kind, ParallelismUnsupportedKind::PairwiseNonCommute);
}

#[test]
fn parallel_different_path_param_names_not_proven_commute() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let upsert = |name: &str, param: &str| {
        op(
            name,
            EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
                key_source: KeySource::PathParam {
                    param: param.into(),
                },
            }),
        )
    };
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![upsert("put_a", "a")]).unwrap(),
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![upsert("put_b", "b")]).unwrap(),
            }),
        ])
        .unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelismUnsupported(d) = r else {
        panic!("expected ParallelismUnsupported — distinct PathParam names are not a disjointness proof");
    };
    assert_eq!(d.kind, ParallelismUnsupportedKind::PairwiseNonCommute);
    assert!(d.reason.contains("put_a") && d.reason.contains("put_b"));
}

#[test]
fn parallel_read_vs_upsert_does_not_commute() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let upsert = op(
        "put",
        EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
            key_source: KeySource::PathParam { param: "k".into() },
        }),
    );
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![read("get")]).unwrap(),
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![upsert]).unwrap(),
            }),
        ])
        .unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelismUnsupported(d) = r else {
        panic!("expected ParallelismUnsupported");
    };
    assert_eq!(d.kind, ParallelismUnsupportedKind::PairwiseNonCommute);
    assert_eq!(d.downstream_stage, "lane2_stage2e_parallelism_lens");
    assert!(d.reason.contains("get") && d.reason.contains("put"));
}

#[test]
fn parallel_append_in_branch_is_broken_by() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![read("get")]).unwrap(),
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: NonEmptyList::from_vec(vec![op(
                    "append_audit",
                    EffectShape::IsBreaking(BreakingShape::AppendEffect),
                )])
                .unwrap(),
            }),
        ])
        .unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelCompositionVerdict(CompositionVerdict::BrokenBy {
        first_breaker,
    }) = r
    else {
        panic!("expected BrokenBy");
    };
    assert_eq!(first_breaker.operation_name, "append_audit");
    assert!(matches!(first_breaker.shape, BreakingShape::AppendEffect));
}

#[test]
fn non_parallel_root_is_unsupported() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![read("only")]).unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelismUnsupported(d) = r else {
        panic!("expected unsupported");
    };
    assert_eq!(d.kind, ParallelismUnsupportedKind::NotParallelEffectRoot);
}
