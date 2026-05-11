//! **Layer:** integration
//!
//! Band-C cementing for `src/v3/lenses/cost.dag`.
//!
//! This is the same-PR receipt required for promoting the `cost_symbolic`
//! registry entry to behavioral COMPLETE. It consumes frozen expectations
//! rather than a live v2 oracle: the v2-side behavior being cemented is the
//! symbolic cost family from the v2 complexity analysis, projected to the
//! standalone v3 `SymbolicCost` carrier.
//!
//! Temporary Rust receipt: `.dag` `TestClaim` data cannot yet express the
//! recursive `SymbolicCost` expected values with `SizeVariable` identity
//! assertions (`M1_2_8_STRUCTURAL_SYMBOLIC_COST_DATA`).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId, SymbolicCost};
use v3_compiler::{analyze_symbolic_cost_dimension, DimensionReport, Witness};

use crate::common::assert_recursive_countdown_linear_semantics;

fn find_bind(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

fn contains_linear_for_port(cost: &SymbolicCost, port: PortId) -> bool {
    match cost {
        SymbolicCost::LinearCost { _0: var } => var.source_port == port,
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => terms
            .iter()
            .any(|term| contains_linear_for_port(term.as_ref(), port)),
        _ => false,
    }
}

fn contains_log_for_port(cost: &SymbolicCost, port: PortId) -> bool {
    match cost {
        SymbolicCost::LogCost { _0: var } => var.source_port == port,
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => terms
            .iter()
            .any(|term| contains_log_for_port(term.as_ref(), port)),
        _ => false,
    }
}

fn expect_symbolic_cost_dimension(
    dag: &v3_compiler::dag::Dag,
    bind_name: &str,
) -> (SymbolicCost, Vec<Witness<SymbolicCost>>) {
    let bind = find_bind(dag, bind_name);
    let report = analyze_symbolic_cost_dimension(dag, bind.id);
    let DimensionReport::DimensionOk {
        dimension_name,
        composed,
        witnesses,
    } = report
    else {
        panic!("expected DimensionOk for `{bind_name}` symbolic cost, got {report:?}");
    };

    assert_eq!(dimension_name, "symbolic_cost");
    (composed, witnesses)
}

fn run_with_cost_cementing_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("cost-lens-symbolic-cementing".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn cost lens symbolic cementing thread")
        .join()
        .expect("cost lens symbolic cementing thread should not panic");
}

#[test]
fn literal_bind_cements_dimension_symbolic_cost_constant() {
    run_with_cost_cementing_stack(|| {
        let dag = compile_to_dag("let lit: Int = 7", "cement_cost_symbolic_lit.v3")
            .expect("literal fixture compiles");
        let (composed, witnesses) = expect_symbolic_cost_dimension(&dag, "lit");

        assert!(
            matches!(composed, SymbolicCost::ConstantCost { _0: 0 }),
            "literal frozen cost projection should stay constant zero, got {composed:?}"
        );
        assert!(
            witnesses.iter().all(|w| matches!(w, Witness::Inhabits(_))),
            "literal cost dimension should have only Inhabits witnesses, got {witnesses:?}"
        );
    });
}

#[test]
fn recursive_countdown_cements_dimension_symbolic_cost_linear_sizevar() {
    run_with_cost_cementing_stack(|| {
        let dag = compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            "cement_cost_symbolic_countdown.v3",
        )
        .expect("recursive countdown fixture compiles");
        let countdown = find_bind(&dag, "countdown");
        let parameter = countdown
            .params
            .first()
            .copied()
            .expect("countdown should expose one size-bearing parameter");

        let (composed, witnesses) = expect_symbolic_cost_dimension(&dag, "countdown");

        assert_recursive_countdown_linear_semantics(&composed);
        assert!(
            contains_linear_for_port(&composed, parameter),
            "countdown frozen cost projection should carry a LinearCost keyed by parameter \
             {parameter:?}, got {composed:?}"
        );
        assert!(
            witnesses.iter().all(|w| matches!(w, Witness::Inhabits(_))),
            "countdown cost dimension should have only Inhabits witnesses, got {witnesses:?}"
        );
    });
}

#[test]
fn division_cements_log_cost_on_dividend_sizevar() {
    run_with_cost_cementing_stack(|| {
        let dag = compile_to_dag(
            "import std.error_primitives { DivError, Result }\n\
             fn half(n: Int) -> Int =\n  match n / 2 { Ok(q) => q, Err(e) => 0 }",
            "cement_cost_symbolic_division.v3",
        )
        .expect("division fixture compiles");
        let half = find_bind(&dag, "half");
        let dividend = half
            .params
            .first()
            .copied()
            .expect("half should expose one dividend parameter");

        let (composed, witnesses) = expect_symbolic_cost_dimension(&dag, "half");

        assert!(
            contains_log_for_port(&composed, dividend),
            "division frozen cost projection should carry a LogCost keyed by dividend \
             {dividend:?}, got {composed:?}"
        );
        assert!(
            witnesses.iter().all(|w| matches!(w, Witness::Inhabits(_))),
            "division cost dimension should have only Inhabits witnesses, got {witnesses:?}"
        );
    });
}
