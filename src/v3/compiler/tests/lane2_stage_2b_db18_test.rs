//! DB-18 / Lane 2 Stage 2b — `WorkflowEffect`, `bool_port_of`, and idempotency analysis.

use v3_compiler::analyze_workflow;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, BreakingShape, CompositionVerdict, CreateCause, EffectShape, IdempotentShape,
    KeySource, NonSingletonList, OperationEffect, TypeConnective, WorkflowEffect,
    WorkflowIdempotencyReport,
};
use v3_compiler::diagnostics::{Diagnostic, SourceSpan};
use v3_compiler::Dag;
use v3_compiler::NodeId;

fn lane2_anchor(dag: &Dag) -> NodeId {
    // Do not use `nodes()[0]`: allocation order can place Transform/Branch/Loop
    // before the first Value/Bind — `try_register_lane2_workflow_effect` only
    // accepts Value or Bind.
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
fn bool_port_of_requires_bool_port() {
    let dag = compile_to_dag("let x = 1 + 2\nlet y = 1 < 2", "branch_arm.v3").expect("compile");
    let binds: Vec<_> = dag.nodes().iter().filter_map(Behavior::as_bind).collect();
    let int_bind = binds.iter().find(|b| b.name == "x").expect("x");
    let bool_bind = binds.iter().find(|b| b.name == "y").expect("y");
    let linear = || WorkflowEffect::LinearEffect {
        ops: vec![op(
            "noop",
            EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
        )],
    };
    assert!(dag.bool_port_of(int_bind.value).is_none());
    let pred = dag.bool_port_of(bool_bind.value).expect("bool port");
    let arm = v3_compiler::dag::BranchArm::new(pred, linear());
    assert_eq!(arm.bool_port().port_id(), bool_bind.value);
}

#[test]
fn linear_empty_ops_is_idempotent_composition() {
    let mut dag = compile_to_dag("let _ = 1", "lane2_empty.v3").expect("compile");
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::LinearEffect { ops: vec![] };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_workflow(&dag, root);
    assert!(matches!(
        r,
        WorkflowIdempotencyReport::WorkflowCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}

#[test]
fn gcp_style_linear_chain_idempotent() {
    let mut dag = compile_to_dag("let _ = 1", "lane2_gcp.v3").expect("compile");
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::LinearEffect {
        ops: vec![
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
        ],
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_workflow(&dag, root);
    assert!(matches!(
        r,
        WorkflowIdempotencyReport::WorkflowCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}

#[test]
fn append_effect_breaks_linear_chain() {
    let mut dag = compile_to_dag("let _ = 1", "lane2_append.v3").expect("compile");
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::LinearEffect {
        ops: vec![
            op(
                "read",
                EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
            ),
            op(
                "append_audit",
                EffectShape::IsBreaking(BreakingShape::AppendEffect),
            ),
        ],
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf.clone()));
    let r = analyze_workflow(&dag, root);
    let WorkflowIdempotencyReport::WorkflowCompositionVerdict(CompositionVerdict::BrokenBy {
        first_breaker,
    }) = r
    else {
        panic!("expected BrokenBy");
    };
    let breaker = wf
        .operation_at(first_breaker)
        .expect("breaker ref should resolve into linear ops");
    assert_eq!(breaker.operation_name, "append_audit");
    assert!(matches!(breaker.shape, EffectShape::IsBreaking(BreakingShape::AppendEffect)));
}

#[test]
fn post_create_is_breaking() {
    let mut dag = compile_to_dag("let _ = 1", "lane2_post.v3").expect("compile");
    let root = lane2_anchor(&dag);
    let wf = WorkflowEffect::LinearEffect {
        ops: vec![op(
            "post_create",
            EffectShape::IsBreaking(BreakingShape::CreateEffect {
                cause: CreateCause::PostAlways,
            }),
        )],
    };
    assert!(dag.try_register_lane2_workflow_effect(root, wf));
    let r = analyze_workflow(&dag, root);
    assert!(matches!(
        r,
        WorkflowIdempotencyReport::WorkflowCompositionVerdict(CompositionVerdict::BrokenBy { .. })
    ));
}

#[test]
fn diagnostic_paths_name_stage2b() {
    let mut dag = compile_to_dag("let c = 1 < 2\nlet d = 2 < 3", "cd.v3").expect("compile");
    let binds: Vec<_> = dag.nodes().iter().filter_map(Behavior::as_bind).collect();
    let c = binds.iter().find(|b| b.name == "c").expect("c");
    let d = binds.iter().find(|b| b.name == "d").expect("d");
    let stage = "lane2_stage2b_idempotency_lens";
    let linear = WorkflowEffect::LinearEffect {
        ops: vec![op(
            "r",
            EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
        )],
    };
    let root = lane2_anchor(&dag);
    for (wf, name) in [
        (
            WorkflowEffect::BranchEffect {
                arms: NonSingletonList::from_vec(vec![
                    v3_compiler::dag::BranchArm::new(
                        dag.bool_port_of(c.value).unwrap(),
                        linear.clone(),
                    ),
                    v3_compiler::dag::BranchArm::new(
                        dag.bool_port_of(d.value).unwrap(),
                        linear.clone(),
                    ),
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
        assert!(dag.try_register_lane2_workflow_effect(root, wf));
        let r = analyze_workflow(&dag, root);
        let WorkflowIdempotencyReport::IdempotencyUnsupported(d) = r else {
            panic!("expected diagnostic for {name}");
        };
        assert_eq!(d.variant_name, name);
        assert_eq!(d.downstream_stage, stage);
    }
}

#[test]
fn branch_condition_not_bool_diagnostic_records_span() {
    let mut dag = compile_to_dag("let x = 1 + 2", "branch_cond.v3").expect("compile");
    let int_port = {
        let binds: Vec<_> = dag.nodes().iter().filter_map(Behavior::as_bind).collect();
        let int_bind = binds.iter().find(|b| b.name == "x").expect("x");
        int_bind.value
    };
    let span = SourceSpan::new("branch_cond.v3", 4, 5);
    assert!(dag
        .bool_port_for_branch_condition_or_diagnose(int_port, span.clone())
        .is_none());
    let diags: Vec<_> = dag.diagnostics().iter().collect();
    assert_eq!(diags.len(), 1);
    let d = diags[0].1;
    let Diagnostic::BranchConditionNotBool {
        port,
        actual_type,
        span: sp,
        ..
    } = d
    else {
        panic!("expected BranchConditionNotBool, got {d:?}");
    };
    assert_eq!(*port, int_port);
    assert!(actual_type.is_some());
    assert_eq!(sp.file, "branch_cond.v3");
}
