//! **Layer:** integration
//!
//! **`symbolic_cost_of` consumer wiring** for `src/v3/lenses/cost.dag` (`regen_lens` →
//! `lens_cost_symbolic_generated.rs`).
//!
//! **Not Band-C cementing** (`TESTING.md` § *Cementing tests — lens subsumption*): Band-C
//! governs explicit v3 **subsumes v2** / register **`COMPLETE` + real v2 counterpart**
//! claims and expects a v2-oracle match or reviewed projection on the same fixture. The
//! `cost.dag` row is **PROXY** until those obligations clear; this module is only a
//! v3-side regression pin on `compile_to_dag` fixtures (the same lookup surface
//! `analyze_symbolic_cost_dimension` walks in `v3_compiler::dimension`).
//!
//! Tests below exercise **only** the generated `symbolic_cost_of` consumer by design;
//! that is **not** a Band-C shortfall while `cost.dag` remains **PROXY** in
//! `docs/v3-lens-capability-register.md` (Band-C would demand a v2-oracle or reviewed
//! projection once the row is `COMPLETE` with a real v2 counterpart).
//!
//! Gate **#78** (`e_p_sub_value_relation_per_call_landed`): integration tests pin **`symbolic_cost_of`**
//! on unary countdown fixtures with **`assert_recursive_countdown_linear_semantics`** (linear-family).
//! Evidence-index discipline for **`per_call_descent_operand_port`** / **`inputs[k]`** vs **`inputs[0]`**
//! is additionally unit-tested on synthetic evidence in **`dag::tests`**
//! (`call_pattern_from_relations_skips_leading_preserved_rows_for_descent_bound_index`) — terminating
//! surface recursion proves descent on the **first** parameter only (`lower.rs`), so those indices do not
//! currently surface from **`compile_to_dag`** multi-arg fixtures.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId, SymbolicCost};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
use v3_compiler::{analyze_symbolic_cost_dimension, DimensionReport, Witness};

use crate::common::assert_recursive_countdown_linear_semantics;

/// Single source of truth for the gate #78 regression fixture label (`compile_to_dag` second
/// argument and `TransformNode.span.file` filter — keep them paired). Tracked debt: ROADMAP gate
/// #78 row (*Test-side `span.file` bridge*).
const E_P78_PER_CALL_PATTERN_FIXTURE_FILE: &str = "e_p78_cost_lens.v3";

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn find_bind_node(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::NodeId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .id
}

/// v3 `symbolic_cost_of` lookup only (no v2 oracle); see module docs for Band-C scope.
fn expect_symbolic_cost(dag: &v3_compiler::dag::Dag, bind_name: &str) -> SymbolicCost {
    let port = find_bind_value(dag, bind_name);
    match symbolic_cost_of(dag, &port) {
        SymbolicCostLookup::Hit(cost) => cost,
        SymbolicCostLookup::Miss => {
            panic!("symbolic_cost_of returned Miss for bind `{bind_name}`")
        }
    }
}

fn expect_symbolic_cost_dimension(dag: &v3_compiler::dag::Dag, bind_name: &str) -> SymbolicCost {
    let report = analyze_symbolic_cost_dimension(dag, find_bind_node(dag, bind_name));
    let DimensionReport::DimensionOk {
        dimension_name,
        composed,
        witnesses,
    } = report
    else {
        panic!("analyze_symbolic_cost_dimension returned failure for bind `{bind_name}`");
    };
    assert_eq!(dimension_name, "symbolic_cost");
    assert!(
        witnesses.iter().all(|w| matches!(w, Witness::Inhabits(_))),
        "symbolic cost dimension should only emit Inhabits witnesses for `{bind_name}`, got \
         {witnesses:?}"
    );
    composed
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

fn run_with_symbolic_cost_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("symbolic-cost-consumer-test".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn symbolic cost consumer test thread")
        .join()
        .expect("symbolic cost consumer test thread should not panic");
}

#[test]
fn literal_bind_pins_symbolic_cost_of_constant_on_fixture() {
    run_with_symbolic_cost_stack(|| {
        let dag = compile_to_dag("let lit: Int = 7", "cost_sym_lit.v3").expect("literal compiles");
        let cost = expect_symbolic_cost(&dag, "lit");
        let dimension_cost = expect_symbolic_cost_dimension(&dag, "lit");
        assert_eq!(
            dimension_cost, cost,
            "dimension entrypoint should compose the same generated symbolic_cost_of carrier"
        );

        assert!(
            matches!(cost, SymbolicCost::ConstantCost { _0: 0 }),
            "literal cost should preserve constant source, got {cost:?}"
        );
    });
}

#[test]
fn recursive_countdown_pins_symbolic_cost_linear_and_sizevar_on_fixture() {
    run_with_symbolic_cost_stack(|| {
        let dag = compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            "cost_sym_countdown.v3",
        )
        .expect("recursive countdown compiles");
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
        let dimension_cost = expect_symbolic_cost_dimension(&dag, "countdown");
        assert_eq!(
            dimension_cost, cost,
            "dimension entrypoint should compose the same generated symbolic_cost_of carrier"
        );

        let mut ports = Vec::new();
        linear_size_ports(&cost, &mut ports);
        assert!(
            ports.contains(&parameter),
            "recursive countdown cost should carry a SizeVariable keyed by the parameter port \
             {parameter:?}, got cost={cost:?}"
        );
        assert_recursive_countdown_linear_semantics(&cost);
    });
}

/// R3 **gate #78** (`e_p_sub_value_relation_per_call_landed`) — cost lens consumes the same
/// `per_call_pattern_at` substrate query as `complexity.dag`, projecting `SubValueRelation`
/// rows from `per_call_descent_evidence` (see `docs/design-cost-lens-sizevar-dimension-wiring.md` §3.2).
#[test]
fn e_p_sub_value_relation_per_call_landed_cost_lens_routes_through_per_call_pattern_query() {
    run_with_symbolic_cost_stack(|| {
        let dag = compile_to_dag(
            "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
            E_P78_PER_CALL_PATTERN_FIXTURE_FILE,
        )
        .expect("compile");
        let pattern_hits = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_transform)
            .filter(|t| {
                t.span.file == E_P78_PER_CALL_PATTERN_FIXTURE_FILE
                    && v3_compiler::dag::per_call_pattern_at(&dag, t.id).is_some()
            })
            .count();
        assert!(
            pattern_hits >= 1,
            "expected >=1 user Callable transform with `per_call_pattern_at` Some (self-call \
             evidence present so the cost lens can branch on recurrence)"
        );
        let cost = expect_symbolic_cost(&dag, "countdown");
        assert_recursive_countdown_linear_semantics(&cost);
    });
}
