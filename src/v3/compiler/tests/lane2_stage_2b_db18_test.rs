//! DB-18 / Lane 2 Stage 2b — `WorkflowEffect`, `branch_arm_of`, and idempotency analysis.

use v3_compiler::analyze_workflow;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, BreakingShape, CompositionVerdict, CreateCause, EffectShape, IdempotentShape,
    KeySource, NonEmptyList, NonSingletonList, OperationEffect, TypeConnective, WorkflowEffect,
    WorkflowIdempotencyReport,
};
use v3_compiler::Dag;

fn op(name: &str, shape: EffectShape) -> OperationEffect {
    OperationEffect {
        operation_name: name.to_string(),
        shape,
    }
}

#[test]
fn workflow_effect_decl_four_variants_in_bootstrap() {
    let dag = Dag::new();
    let decl = dag
        .declaration_by_name("WorkflowEffect")
        .expect("WorkflowEffect type from effects.dag");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("expected WorkflowEffect to be a sum");
    };
    assert_eq!(variants.len(), 4, "Linear / Branch / Loop / Parallel");
}

#[test]
fn branch_arm_of_requires_bool_port() {
    let dag = compile_to_dag("let x = 1 + 2\nlet y = 1 < 2", "branch_arm.v3").expect("compile");
    let binds: Vec<_> = dag.nodes().iter().filter_map(Behavior::as_bind).collect();
    let int_bind = binds.iter().find(|b| b.name == "x").expect("x");
    let bool_bind = binds.iter().find(|b| b.name == "y").expect("y");
    let linear = || WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![op(
            "noop",
            EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
        )])
        .unwrap(),
    };
    assert!(dag.branch_arm_of(int_bind.value, linear()).is_none());
    let arm = dag.branch_arm_of(bool_bind.value, linear()).expect("bool arm");
    assert_eq!(arm.branch_predicate().port_id(), bool_bind.value);
}

#[test]
fn gcp_style_linear_chain_idempotent() {
    let dag = Dag::new();
    let wf = WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![
            op(
                "get_secret",
                EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
            ),
            op(
                "put_secret",
                EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
                    key_source: KeySource::PathParam {
                        param: "name".into(),
                    },
                }),
            ),
            op(
                "grant",
                EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
            ),
        ])
        .unwrap(),
    };
    let r = analyze_workflow(&dag, &wf);
    assert!(matches!(
        r,
        WorkflowIdempotencyReport::WorkflowCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}

#[test]
fn append_effect_breaks_linear_chain() {
    let dag = Dag::new();
    let wf = WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![
            op(
                "read",
                EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
            ),
            op(
                "append_audit",
                EffectShape::IsBreaking(BreakingShape::AppendEffect),
            ),
        ])
        .unwrap(),
    };
    let r = analyze_workflow(&dag, &wf);
    let WorkflowIdempotencyReport::WorkflowCompositionVerdict(CompositionVerdict::BrokenBy {
        first_breaker,
    }) = r
    else {
        panic!("expected BrokenBy");
    };
    assert_eq!(first_breaker.operation_name, "append_audit");
    assert!(matches!(first_breaker.shape, BreakingShape::AppendEffect));
}

#[test]
fn post_create_is_breaking() {
    let dag = Dag::new();
    let wf = WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![op(
            "post_create",
            EffectShape::IsBreaking(BreakingShape::CreateEffect {
                cause: CreateCause::PostAlways,
            }),
        )])
        .unwrap(),
    };
    let r = analyze_workflow(&dag, &wf);
    assert!(matches!(
        r,
        WorkflowIdempotencyReport::WorkflowCompositionVerdict(CompositionVerdict::BrokenBy { .. })
    ));
}

#[test]
fn diagnostic_paths_name_stage2b() {
    let dag = compile_to_dag("let c = 1 < 2\nlet d = 2 < 3", "cd.v3").expect("compile");
    let binds: Vec<_> = dag.nodes().iter().filter_map(Behavior::as_bind).collect();
    let c = binds.iter().find(|b| b.name == "c").expect("c");
    let d = binds.iter().find(|b| b.name == "d").expect("d");
    let stage = "lane2_stage2b_idempotency_lens";
    let linear = WorkflowEffect::LinearEffect {
        ops: NonEmptyList::from_vec(vec![op(
            "r",
            EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
        )])
        .unwrap(),
    };
    for (wf, name) in [
        (
            WorkflowEffect::BranchEffect {
                arms: NonSingletonList::from_vec(vec![
                    dag.branch_arm_of(c.value, linear.clone()).unwrap(),
                    dag.branch_arm_of(d.value, linear.clone()).unwrap(),
                ])
                .unwrap(),
            },
            "BranchEffect",
        ),
        (
            WorkflowEffect::LoopEffect {
                body: Box::new(linear.clone()),
            },
            "LoopEffect",
        ),
        (
            WorkflowEffect::ParallelEffect {
                branches: NonSingletonList::from_vec(vec![
                    Box::new(linear.clone()),
                    Box::new(linear.clone()),
                ])
                .unwrap(),
            },
            "ParallelEffect",
        ),
    ] {
        let r = analyze_workflow(&dag, &wf);
        let WorkflowIdempotencyReport::IdempotencyUnsupported(d) = r else {
            panic!("expected diagnostic for {name}");
        };
        assert_eq!(d.variant_name, name);
        assert_eq!(d.downstream_stage, stage);
    }
}
