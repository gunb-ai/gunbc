//! Lane 2 Stage 2e / DB-20 — `ParallelEffect` parallel composition safety.

use std::sync::OnceLock;

use v3_compiler::analyze_parallelism;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, BreakingShape, CompositionVerdict, EffectShape, IdempotentShape, KeySource,
    NonEmptyList, NonSingletonList, OperationEffect, WorkflowEffect, WorkflowParallelismReport,
};
use v3_compiler::Dag;
use v3_compiler::NodeId;

/// One `compile_to_dag("let _ = 1", …)` for all tests that mutate the user DAG
/// (each test [`Dag::clone`]s so `try_register_lane2_workflow_effect` stays isolated).
static TRIVIAL_USER_DAG: OnceLock<Dag> = OnceLock::new();

fn trivial_user_dag() -> Dag {
    TRIVIAL_USER_DAG
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
    let mut dag = trivial_user_dag();
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
fn parallel_disjoint_path_upserts_commute() {
    let mut dag = trivial_user_dag();
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
    assert!(matches!(
        r,
        WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}

#[test]
fn parallel_read_vs_upsert_does_not_commute() {
    let mut dag = trivial_user_dag();
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
    assert_eq!(d.variant_name, "PairwiseNonCommute");
    assert_eq!(d.downstream_stage, "lane2_stage2e_parallelism_lens");
    assert!(d.reason.contains("get") && d.reason.contains("put"));
}

#[test]
fn parallel_append_in_branch_is_broken_by() {
    let mut dag = trivial_user_dag();
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
    let mut dag = trivial_user_dag();
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![read("only")]).unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelismUnsupported(d) = r else {
        panic!("expected unsupported");
    };
    assert_eq!(d.variant_name, "NotParallelEffect");
}
