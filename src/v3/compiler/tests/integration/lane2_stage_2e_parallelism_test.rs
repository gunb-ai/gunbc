//! Lane 2 Stage 2e / DB-20 — `ParallelEffect` parallel composition safety.
//!
//! Derivation coverage (merge gate: pairwise commutativity from `Operation` shapes):
//! - **Commute (green):** `parallel_read_only_branches_commute` — cross-branch `ReadEffect` only.
//! - **Upsert×Upsert (v1):** always **red** — `UpsertEffect` has no merge/value witness; see
//!   `parallel_upsert_cross_branch_fail_closed_same_op_name` and
//!   `parallel_upsert_cross_branch_fail_closed_distinct_op_names`.
//! - **Non-commute / fail-closed (red):** `parallel_different_path_param_names_not_proven_commute` —
//!   distinct `PathParam` names (not a disjointness proof); `parallel_read_vs_upsert_does_not_commute` —
//!   read vs write shape clash.
//! - **Breaking op:** `parallel_append_in_branch_is_broken_by` — `BrokenBy` before pairwise check.
//!
//! **CI / Layer 1:** integration tests must not call `compile_to_dag` per `#[test]` for the same
//! fixture — cold runners multiply that into minutes (see #543 / v3 full-suite wall-clock ratchet).
//! One `OnceLock` compile + `Dag::clone` per test keeps `try_register_lane2_workflow_effect` isolated.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use v3_compiler::analyze_parallelism;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    operation_effect_shape, Behavior, BreakingShape, CallableRef, CompositionVerdict, CreateCause,
    EffectShape, HttpMethodScalar, IdempotentShape, InputField, KeySource, NonSingletonList,
    Operation, ParallelismUnsupportedKind, PathTemplate, RestEndpointBinding, UrlPathToken,
    WorkflowEffect, WorkflowParallelismReport,
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

fn op(dag: &Dag, _name: &str, shape: EffectShape) -> Operation {
    let callable_name = match &shape {
        EffectShape::IsIdempotent(IdempotentShape::ReadEffect) => "get_method",
        EffectShape::IsIdempotent(IdempotentShape::UpsertEffect { .. }) => "map_insert_method",
        EffectShape::IsIdempotent(IdempotentShape::DeleteEffect { .. }) => "diff_method",
        EffectShape::IsBreaking(BreakingShape::CreateEffect { .. }) => "concat_method",
        EffectShape::IsBreaking(BreakingShape::AppendEffect) => "append_method",
    };
    let callable = dag
        .declaration_by_name(callable_name)
        .unwrap_or_else(|| panic!("missing callable declaration `{callable_name}`"))
        .id;
    let (method, tokens) = match shape {
        EffectShape::IsIdempotent(IdempotentShape::ReadEffect) => (HttpMethodScalar::Get, vec![]),
        EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
            key_source: KeySource::PathParam { param },
        }) => (
            HttpMethodScalar::Put,
            vec![UrlPathToken::ParamToken { name: param }],
        ),
        EffectShape::IsIdempotent(IdempotentShape::DeleteEffect {
            key_source: KeySource::PathParam { param },
        }) => (
            HttpMethodScalar::Delete,
            vec![UrlPathToken::ParamToken { name: param }],
        ),
        EffectShape::IsBreaking(BreakingShape::CreateEffect {
            cause: CreateCause::PostAlways,
        }) => (HttpMethodScalar::Post, vec![]),
        EffectShape::IsBreaking(BreakingShape::AppendEffect) => (HttpMethodScalar::Post, vec![]),
        EffectShape::IsBreaking(BreakingShape::CreateEffect {
            cause: CreateCause::KeylessFallback { method },
        }) => (method, vec![]),
        EffectShape::IsIdempotent(IdempotentShape::UpsertEffect { .. })
        | EffectShape::IsIdempotent(IdempotentShape::DeleteEffect { .. }) => {
            (HttpMethodScalar::Put, vec![])
        }
    };
    Operation {
        callable: CallableRef { decl: callable },
        inputs: BTreeMap::<String, InputField>::new(),
        endpoint: RestEndpointBinding {
            method,
            path: PathTemplate { tokens },
        },
    }
}

fn read(dag: &Dag, name: &str) -> Operation {
    op(
        dag,
        name,
        EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
    )
}

#[test]
fn parallel_requires_at_least_two_branches_type_level() {
    let dag = shared_fixture_dag();
    let linear = WorkflowEffect::LinearEffect {
        ops: vec![read(&dag, "r")],
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
                ops: vec![read(&dag, "a"), read(&dag, "b")],
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![read(&dag, "c")],
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
fn parallel_upsert_cross_branch_fail_closed_same_op_name() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let key = KeySource::PathParam { param: "id".into() };
    let upsert = op(
        &dag,
        "put_item",
        EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
            key_source: key.clone(),
        }),
    );
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![upsert.clone()],
            }),
            Box::new(WorkflowEffect::LinearEffect { ops: vec![upsert] }),
        ])
        .unwrap(),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelismUnsupported(d) = r else {
        panic!("expected ParallelismUnsupported — Upsert×Upsert has no merge witness in v1");
    };
    assert_eq!(d.kind, ParallelismUnsupportedKind::PairwiseNonCommute);
}

#[test]
fn parallel_upsert_cross_branch_fail_closed_distinct_op_names() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let key = KeySource::PathParam { param: "id".into() };
    let upsert = |name: &str| {
        op(
            &dag,
            name,
            EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
                key_source: key.clone(),
            }),
        )
    };
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![upsert("put_a")],
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![upsert("put_b")],
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
}

#[test]
fn parallel_different_path_param_names_not_proven_commute() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let upsert = |name: &str, param: &str| {
        op(
            &dag,
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
                ops: vec![upsert("put_a", "a")],
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![upsert("put_b", "b")],
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
    assert_eq!(
        d.reason,
        "parallel branch operations do not commute under parallel scheduling"
    );
}

#[test]
fn parallel_read_vs_upsert_does_not_commute() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let upsert = op(
        &dag,
        "put",
        EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
            key_source: KeySource::PathParam { param: "k".into() },
        }),
    );
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![read(&dag, "get")],
            }),
            Box::new(WorkflowEffect::LinearEffect { ops: vec![upsert] }),
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
    assert_eq!(
        d.reason,
        "parallel branch operations do not commute under parallel scheduling"
    );
}

#[test]
fn parallel_append_in_branch_is_broken_by() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![read(&dag, "get")],
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![op(
                    &dag,
                    "append_audit",
                    EffectShape::IsBreaking(BreakingShape::AppendEffect),
                )],
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
    let breaker = dag
        .lane2_workflow_effect_at(&root)
        .expect("registered workflow should be readable at the root")
        .operation_at(first_breaker)
        .expect("parallel breaker ref should resolve in branch-order flattening");
    assert!(matches!(
        operation_effect_shape(&dag, breaker),
        EffectShape::IsBreaking(BreakingShape::AppendEffect)
    ));
}

#[test]
fn non_parallel_root_is_unsupported() {
    let mut dag = shared_fixture_dag();
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::LinearEffect {
        ops: vec![read(&dag, "only")],
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_parallelism(&dag, root);
    let WorkflowParallelismReport::ParallelismUnsupported(d) = r else {
        panic!("expected unsupported");
    };
    assert_eq!(d.kind, ParallelismUnsupportedKind::NotParallelEffectRoot);
}
