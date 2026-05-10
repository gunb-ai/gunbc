//! **Layer:** integration
//!
//! R3 gate #73 — `lens_behavioral_parity_demonstration`.
//!
//! This is the executable demonstration receipt for the four in-R3 lens surfaces
//! named by T-Lens-Behavioral-Parity. It consumes the current frozen expectations
//! directly rather than calling a live v2 oracle, so it remains compatible with
//! `v2_oracle_no_remaining_test_consumers`.
//!
//! P5 receipt: this hand-written Rust test exists only as the gate #73 host
//! receipt while `LensOutputEquals` / frozen-oracle claims are still migrating to
//! data. Dissolution is tracked by `docs/r3-program-plan.md` §1.8 row 73
//! (`lens_behavioral_parity_demonstration`), the T-Tests-As-Data rows 84/87, and
//! `ROADMAP.md` §"Post-merge debt" row "Hand-Rust census" / T-PB-B test subset;
//! when those rows can express this four-lens snapshot as `.dag` TestClaim data,
//! delete this module and its `tests/integration.rs` registration.

use std::sync::OnceLock;

use v3_compiler::analyze_parallelism;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    AsymptoticClass, Behavior, CompositionVerdict, EffectShape, IdempotentShape, NonSingletonList,
    OperationEffect, PortId, SymbolicCost, TransformTarget, TypeConnective, WorkflowEffect,
    WorkflowParallelismReport,
};
use v3_compiler::lens_cost::ComplexitySummary;
use v3_compiler::lens_cost::{complexity_of, Certainty, ComplexityLookup};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
use v3_compiler::lens_effect_enumeration::{
    enumerate_effects, EffectEnumerationReport, StructuralEffectShape, TransactionalPattern,
};
use v3_compiler::Dag;

static COUNTDOWN_DAG: OnceLock<Dag> = OnceLock::new();
static EFFECTS_DAG: OnceLock<Dag> = OnceLock::new();
static COUNTDOWN_COMPLEXITY: OnceLock<(ComplexitySummary, PortId)> = OnceLock::new();
static COUNTDOWN_SYMBOLIC_COST: OnceLock<(SymbolicCost, PortId)> = OnceLock::new();
static EFFECTS_REPORT: OnceLock<EffectEnumerationReport> = OnceLock::new();

fn countdown_dag() -> &'static Dag {
    COUNTDOWN_DAG.get_or_init(|| {
        compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            "r3_gate_73_countdown.v3",
        )
        .expect("recursive countdown fixture compiles")
    })
}

fn effects_dag() -> &'static Dag {
    EFFECTS_DAG.get_or_init(|| {
        compile_to_dag("let answer: Int = 1 + 2", "r3_gate_73_effects.v3")
            .expect("effect enumeration fixture compiles")
    })
}

fn countdown_complexity() -> (ComplexitySummary, PortId) {
    COUNTDOWN_COMPLEXITY
        .get_or_init(|| {
            let (dag, countdown, parameter) = countdown_fixture();
            let complexity = match complexity_of(&dag, &countdown.value) {
                ComplexityLookup::Hit(summary) => summary,
                ComplexityLookup::Miss => panic!("complexity lens returned Miss for countdown"),
            };
            (complexity, parameter)
        })
        .clone()
}

fn countdown_symbolic_cost() -> (SymbolicCost, PortId) {
    COUNTDOWN_SYMBOLIC_COST
        .get_or_init(|| {
            let (dag, countdown, parameter) = countdown_fixture();
            let symbolic_cost = match symbolic_cost_of(&dag, &countdown.value) {
                SymbolicCostLookup::Hit(cost) => cost,
                SymbolicCostLookup::Miss => panic!("cost lens returned Miss for countdown"),
            };
            (symbolic_cost, parameter)
        })
        .clone()
}

fn effects_report() -> EffectEnumerationReport {
    EFFECTS_REPORT
        .get_or_init(|| enumerate_effects(effects_dag()))
        .clone()
}

fn find_bind(dag: &Dag, name: &str) -> v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

fn contains_linear(cost: &SymbolicCost, source_port: PortId) -> bool {
    match cost {
        SymbolicCost::LinearCost { _0: var } => var.source_port == source_port,
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => terms
            .iter()
            .any(|term| contains_linear(term.as_ref(), source_port)),
        _ => false,
    }
}

fn read_op(name: &str) -> OperationEffect {
    OperationEffect {
        operation_name: name.to_string(),
        shape: EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
    }
}

fn has_non_arrow_callable_gap(dag: &Dag, report: &EffectEnumerationReport) -> bool {
    report.coverage_gaps.iter().any(|gap| {
        let Some(Behavior::Transform(transform)) = dag
            .nodes()
            .iter()
            .find(|behavior| behavior.id() == gap.node)
        else {
            return false;
        };
        let TransformTarget::Callable(target) = &transform.target else {
            return false;
        };
        dag.declaration_opt(target)
            .is_some_and(|decl| !matches!(decl.connective, TypeConnective::Arrow { .. }))
    })
}

fn run_with_parity_demo_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("lens-behavioral-parity-demo".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn lens behavioral parity demo thread")
        .join()
        .expect("lens behavioral parity demo thread should not panic");
}

fn countdown_fixture() -> (Dag, v3_compiler::dag::BindNode, PortId) {
    let dag = countdown_dag().clone();
    let countdown = find_bind(&dag, "countdown");
    let parameter = countdown
        .params
        .first()
        .copied()
        .expect("countdown should expose one size-bearing parameter");
    (dag, countdown, parameter)
}

#[test]
fn r3_gate_73_demonstrates_complexity_work_is_linear_in_countdown_parameter() {
    run_with_parity_demo_stack(|| {
        let (complexity, parameter) = countdown_complexity();
        assert!(
            contains_linear(&complexity.work, parameter),
            "complexity work should consume the recursive CallPattern as linear parameter cost, got {:?}",
            complexity.work
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_complexity_span_is_linear_in_countdown_parameter() {
    run_with_parity_demo_stack(|| {
        let (complexity, parameter) = countdown_complexity();
        assert!(
            contains_linear(&complexity.span, parameter),
            "complexity span should consume the recursive CallPattern as linear parameter cost, got {:?}",
            complexity.span
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_complexity_classification_is_conservative() {
    run_with_parity_demo_stack(|| {
        let (complexity, _) = countdown_complexity();
        assert_eq!(
            complexity.asymptotic_class,
            AsymptoticClass::ClassUnknown,
            "frozen snapshot keeps composite countdown classification conservative"
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_complexity_certainty_is_proven() {
    run_with_parity_demo_stack(|| {
        let (complexity, _) = countdown_complexity();
        assert!(matches!(complexity.work_certainty, Certainty::Proven));
        assert!(matches!(complexity.span_certainty, Certainty::Proven));
    });
}

#[test]
fn r3_gate_73_demonstrates_symbolic_cost_is_linear() {
    run_with_parity_demo_stack(|| {
        let (symbolic_cost, _) = countdown_symbolic_cost();
        assert!(
            matches!(symbolic_cost, SymbolicCost::LinearCost { .. }),
            "cost lens frozen snapshot expects countdown symbolic cost to normalize to LinearCost, got {symbolic_cost:?}"
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_symbolic_cost_is_keyed_by_countdown_parameter() {
    run_with_parity_demo_stack(|| {
        let (symbolic_cost, parameter) = countdown_symbolic_cost();
        assert!(
            contains_linear(&symbolic_cost, parameter),
            "cost lens linear SizeVariable should be keyed by countdown parameter {parameter:?}, got {symbolic_cost:?}"
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_parallelism_parity_snapshot() {
    let mut dag = compile_to_dag("let anchor: Int = 1", "r3_gate_73_parallelism.v3")
        .expect("parallelism anchor fixture compiles");
    let root = dag
        .nodes()
        .iter()
        .find(|behavior| matches!(behavior, Behavior::Value(_) | Behavior::Bind(_)))
        .expect("anchor fixture should contain a behavior node")
        .id();
    let workflow = WorkflowEffect::ParallelEffect {
        branches: NonSingletonList::from_vec(vec![
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![read_op("read_user")],
            }),
            Box::new(WorkflowEffect::LinearEffect {
                ops: vec![read_op("read_account")],
            }),
        ])
        .expect("two branches satisfy NonSingletonList"),
    };
    assert!(dag.try_register_lane2_workflow_effect(root, workflow));

    let report = analyze_parallelism(&dag, root);
    assert!(matches!(
        report,
        WorkflowParallelismReport::ParallelCompositionVerdict(
            CompositionVerdict::IdempotentComposition
        )
    ));
}

#[test]
fn r3_gate_73_demonstrates_effect_enumeration_publishes_facts() {
    run_with_parity_demo_stack(|| {
        let report = effects_report();

        assert!(
            !report.facts.is_empty(),
            "effect enumeration should publish facts for the representative fixture"
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_effect_enumeration_preserves_non_arrow_callable_gap() {
    run_with_parity_demo_stack(|| {
        let dag = effects_dag();
        let report = effects_report();

        assert!(
            has_non_arrow_callable_gap(dag, &report),
            "effect enumeration frozen snapshot should preserve the current non-arrow callable coverage gap, got {:?}",
            report.coverage_gaps
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_effect_enumeration_proves_no_effect_fact() {
    run_with_parity_demo_stack(|| {
        let report = effects_report();

        assert!(
            report
                .facts
                .iter()
                .any(|fact| matches!(fact.shape, StructuralEffectShape::NoEffect)),
            "effect enumeration should still prove at least one source fact NoEffect, got {:?}",
            report.facts
        );
    });
}

#[test]
fn r3_gate_73_demonstrates_effect_enumeration_has_no_redundant_reads() {
    run_with_parity_demo_stack(|| {
        let report = effects_report();

        assert!(report.redundant_reads.is_empty());
    });
}

#[test]
fn r3_gate_73_demonstrates_effect_enumeration_has_no_transaction() {
    run_with_parity_demo_stack(|| {
        let report = effects_report();

        assert!(matches!(
            report.transaction,
            TransactionalPattern::NoTransaction
        ));
    });
}
