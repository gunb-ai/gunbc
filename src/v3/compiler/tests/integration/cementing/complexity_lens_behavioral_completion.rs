//! **Layer:** integration
//!
//! Band-C cementing for `src/v3/lenses/complexity.dag`.
//!
//! This module is the same-PR receipt required when the capability register
//! promotes a real-v2-counterpart lens to COMPLETE. The frozen v2-oracle values
//! below are the structural expectations this slice must preserve while widening
//! the v3 surface from `Lookup<Int>` to `Lookup<ComplexitySummary>`.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{AsymptoticClass, Behavior, PortId, SymbolicCost};
use v3_compiler::lens_cost::{complexity_of, Certainty, ComplexityLookup};

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn expect_summary(
    dag: &v3_compiler::dag::Dag,
    bind_name: &str,
) -> v3_compiler::lens_cost::ComplexitySummary {
    let port = find_bind_value(dag, bind_name);
    match complexity_of(dag, &port) {
        ComplexityLookup::Hit(summary) => summary,
        ComplexityLookup::Miss => panic!("complexity_of returned Miss for bind `{bind_name}`"),
    }
}

fn assert_proven(certainty: &Certainty, context: &str) {
    assert!(
        matches!(certainty, Certainty::Proven),
        "{context}: expected Proven, got {certainty:?}"
    );
}

fn contains_linear(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::LinearCost { .. } => true,
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            terms.iter().any(|term| contains_linear(term.as_ref()))
        }
        _ => false,
    }
}

fn run_with_lens_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("complexity-lens-cementing".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn complexity lens cementing thread")
        .join()
        .expect("complexity lens cementing thread should not panic");
}

#[test]
fn literal_bind_cements_constant_complexity_summary() {
    run_with_lens_stack(|| {
        let dag = compile_to_dag("let lit: Int = 7", "cement_complexity_lit.v3")
            .expect("literal fixture compiles");
        let summary = expect_summary(&dag, "lit");

        assert!(
            matches!(summary.work, SymbolicCost::ConstantCost { _0: 0 }),
            "literal work should be O(1) zero-cost source, got {:?}",
            summary.work
        );
        assert!(
            matches!(summary.span, SymbolicCost::ConstantCost { _0: 0 }),
            "literal span should be O(1) zero-cost source, got {:?}",
            summary.span
        );
        assert_eq!(summary.asymptotic_class, AsymptoticClass::ClassConstant);
        assert_proven(&summary.work_certainty, "literal work certainty");
        assert_proven(&summary.span_certainty, "literal span certainty");
    });
}

#[test]
fn recursive_countdown_cements_linear_work_and_span() {
    run_with_lens_stack(|| {
        let dag = compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            "cement_complexity_countdown.v3",
        )
        .expect("recursive countdown fixture compiles");
        let summary = expect_summary(&dag, "countdown");

        assert!(
            contains_linear(&summary.work),
            "countdown work should consume CallPattern descent as a LinearCost term, got {:?}",
            summary.work
        );
        assert!(
            contains_linear(&summary.span),
            "countdown span should consume CallPattern descent as a LinearCost term, got {:?}",
            summary.span
        );
        assert_eq!(
            summary.asymptotic_class,
            AsymptoticClass::ClassUnknown,
            "composite product classification stays conservative until degree arithmetic lands"
        );
        assert_proven(&summary.work_certainty, "countdown work certainty");
        assert_proven(&summary.span_certainty, "countdown span certainty");
    });
}
