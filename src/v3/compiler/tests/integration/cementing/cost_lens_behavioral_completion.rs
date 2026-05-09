//! **Layer:** integration
//!
//! V3-side cementing for `src/v3/lenses/cost.dag`.
//!
//! The cost-lens behavioral-completion gate is currently cemented through the
//! Rust-dispatched `analyze_symbolic_cost_dimension` entrypoint. That is the
//! interim authority for `Dimension<SymbolicCost>` until class-5 data record
//! bodies can express the dimension declaration directly in `std/dimensions.dag`.
//! This is not the full Band-C frozen-v2-oracle parity closure.

use v3_compiler::dag::{Behavior, BindNode, NodeId, PortId, SymbolicCost};
use v3_compiler::{analyze_symbolic_cost_dimension, compile_to_dag, DimensionReport, Witness};

fn find_bind<'a>(dag: &'a v3_compiler::dag::Dag, name: &str) -> &'a BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
}

fn expect_dimension_ok(
    dag: &v3_compiler::dag::Dag,
    bind_name: &str,
) -> (SymbolicCost, Vec<Witness<SymbolicCost>>) {
    let root: NodeId = find_bind(dag, bind_name).id;
    match analyze_symbolic_cost_dimension(dag, root) {
        DimensionReport::DimensionOk {
            dimension_name,
            composed,
            witnesses,
        } => {
            assert_eq!(dimension_name, "symbolic_cost");
            assert!(
                witnesses
                    .iter()
                    .all(|w| !matches!(w, Witness::Violates { .. })),
                "cost dimension should not carry violating witnesses for `{bind_name}`: {witnesses:?}"
            );
            assert!(
                witnesses
                    .iter()
                    .all(|w| !matches!(w, Witness::Inhabits(SymbolicCost::UnknownCost { .. }))),
                "cost dimension must not fabricate UnknownCost witnesses for `{bind_name}`: {witnesses:?}"
            );
            (composed, witnesses)
        }
        other => panic!("expected DimensionOk for bind `{bind_name}`, got {other:?}"),
    }
}

fn linear_size_ports(cost: &SymbolicCost, out: &mut Vec<PortId>) {
    match cost {
        SymbolicCost::LinearCost { _0: var } | SymbolicCost::LogCost { _0: var } => {
            out.push(var.source_port);
        }
        SymbolicCost::PolynomialCost { var, .. } => {
            out.push(var.source_port);
        }
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            for term in terms.iter() {
                linear_size_ports(term.as_ref(), out);
            }
        }
        SymbolicCost::ConstantCost { .. } | SymbolicCost::UnknownCost { .. } => {}
    }
}

fn run_with_cost_lens_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("cost-lens-cementing".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn cost lens cementing thread")
        .join()
        .expect("cost lens cementing thread should not panic");
}

#[test]
fn literal_bind_cements_symbolic_cost_dimension_constant() {
    run_with_cost_lens_stack(|| {
        let dag = compile_to_dag("let lit: Int = 7", "cement_cost_lit.v3")
            .expect("literal fixture compiles");
        let (composed, witnesses) = expect_dimension_ok(&dag, "lit");

        assert!(
            matches!(composed, SymbolicCost::ConstantCost { _0: 0 }),
            "literal cost should preserve the v3 symbolic-cost dimension's constant source cost, \
             got {composed:?}"
        );
        assert!(
            witnesses
                .iter()
                .any(|w| matches!(w, Witness::Inhabits(SymbolicCost::ConstantCost { _0: 0 }))),
            "literal dimension report should expose constant-cost witnesses, got {witnesses:?}"
        );
    });
}

#[test]
fn recursive_countdown_cements_dimension_linear_cost_and_sizevar_identity() {
    run_with_cost_lens_stack(|| {
        let dag = compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n <= 0 then 0 else countdown(n - 1)",
            "cement_cost_countdown.v3",
        )
        .expect("recursive countdown fixture compiles");
        let countdown = find_bind(&dag, "countdown");
        let parameter = countdown
            .params
            .first()
            .copied()
            .expect("countdown should have one size-bearing parameter port");
        let (composed, witnesses) = expect_dimension_ok(&dag, "countdown");

        let mut composed_ports = Vec::new();
        linear_size_ports(&composed, &mut composed_ports);
        assert!(
            composed_ports.contains(&parameter),
            "recursive countdown cost should carry a SizeVariable keyed by the parameter port \
             {parameter:?}, got composed={composed:?}"
        );
        assert!(
            matches!(composed, SymbolicCost::LinearCost { .. }),
            "recursive countdown should normalize iterate(O(n), O(1)) to a linear bound, \
             got {composed:?}"
        );
        assert!(
            witnesses.iter().any(|w| {
                let Witness::Inhabits(cost) = w else {
                    return false;
                };
                let mut ports = Vec::new();
                linear_size_ports(cost, &mut ports);
                ports.contains(&parameter)
            }),
            "at least one reachable cost witness should preserve SizeVariable source_port identity \
             for countdown's parameter, got {witnesses:?}"
        );
    });
}
