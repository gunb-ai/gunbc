//! **Layer:** integration
//!
//! Band-C cementing for `src/v3/lenses/cost.dag`.
//!
//! Exercises the generated `symbolic_cost_of` consumer (`lens_cost_symbolic_generated.rs`)
//! on `compile_to_dag` fixtures — the same lookup surface `analyze_symbolic_cost_dimension`
//! uses in `v3_compiler::dimension` (per-behavior `symbolic_cost_of` spine).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId, SymbolicCost};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn expect_symbolic_cost(dag: &v3_compiler::dag::Dag, bind_name: &str) -> SymbolicCost {
    let port = find_bind_value(dag, bind_name);
    match symbolic_cost_of(dag, &port) {
        SymbolicCostLookup::Hit(cost) => cost,
        SymbolicCostLookup::Miss => {
            panic!("symbolic_cost_of returned Miss for bind `{bind_name}`")
        }
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
fn literal_bind_cements_symbolic_cost_lens_constant() {
    run_with_cost_lens_stack(|| {
        let dag = compile_to_dag("let lit: Int = 7", "cement_cost_lit.v3")
            .expect("literal fixture compiles");
        let cost = expect_symbolic_cost(&dag, "lit");

        assert!(
            matches!(cost, SymbolicCost::ConstantCost { _0: 0 }),
            "literal cost should preserve constant source, got {cost:?}"
        );
    });
}

#[test]
fn recursive_countdown_cements_symbolic_cost_linear_and_sizevar_identity() {
    run_with_cost_lens_stack(|| {
        let dag = compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            "cement_cost_countdown.v3",
        )
        .expect("recursive countdown fixture compiles");
        let countdown = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|bind| bind.name == "countdown")
            .expect("countdown bind");
        let parameter = countdown
            .params
            .first()
            .copied()
            .expect("countdown should have one size-bearing parameter port");
        let cost = expect_symbolic_cost(&dag, "countdown");

        let mut ports = Vec::new();
        linear_size_ports(&cost, &mut ports);
        assert!(
            ports.contains(&parameter),
            "recursive countdown cost should carry a SizeVariable keyed by the parameter port \
             {parameter:?}, got cost={cost:?}"
        );
        assert!(
            matches!(cost, SymbolicCost::LinearCost { .. }),
            "recursive countdown should normalize iterate(O(n), O(1)) to a linear bound, \
             got {cost:?}"
        );
    });
}
