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
//! Gate **#78** (`e_p_sub_value_relation_per_call_landed`): **`e_p78_descent_operand_port_follows_evidence_index_not_first_input`**
//! proves **`per_call_descent_operand_port`** tracks the evidence-selected **`Transform.inputs`** slot,
//! not **`inputs[0]`**. Unary Int tail recursion cannot put descent only on a later argument (termination
//! expects the first parameter to descend); this regression uses **preserved head `Int` + list-tail structural
//! descent** so evidence row `0` is **`PreservedValue`** and the relation chosen for **`CallPattern`** is on
//! **`inputs[1]`**.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;
use v3_compiler::dag::{
    per_call_descent_evidence, per_call_descent_operand_port, Behavior, PortId, SubValueRelation,
    SymbolicCost,
};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

use crate::common::assert_recursive_countdown_linear_semantics;

/// Single source of truth for the gate #78 regression fixture label (`compile_to_dag` second
/// argument and `TransformNode.span.file` filter — keep them paired).
const E_P78_PER_CALL_PATTERN_FIXTURE_FILE: &str = "e_p78_cost_lens.v3";

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
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

/// R3 gate **#78** — regression that matters for **descent index discipline**: when evidence slot 0 is
/// **`PreservedValue`** and the first selectable **`SubValueRelation`** maps to a **`CallPattern`** on a
/// **later** operand, `per_call_descent_operand_port` must resolve **`TransformNode::inputs[k]`** for that
/// row — **not** `inputs[0]`.
#[test]
fn e_p78_descent_operand_port_follows_evidence_index_not_first_input() {
    run_with_symbolic_cost_stack(|| {
        let dag = match compile_to_dag(
            "\
type EpListP = EpNilP | EpConsP(EpListP)
fn ep_g78_acc(i: Int, xs: EpListP) -> Int =
  match xs {
    EpConsP(tail) => ep_g78_acc(i, tail),
    EpNilP => i
  }
",
            "e_p78_non_head_descent.v3",
        ) {
            Ok(d) => d,
            Err(CompileError::Semantic(d)) => panic!(
                "compile diagnostics: {:?}",
                d.diagnostics().iter().collect::<Vec<_>>()
            ),
            Err(e) => panic!("compile error: {e:?}"),
        };

        let entry = per_call_descent_evidence(&dag)
            .into_iter()
            .find(|e| e.caller == "ep_g78_acc" && e.callee == "ep_g78_acc")
            .expect("expected ep_g78_acc self-call in per-call descent evidence");

        assert!(
            entry.evidence.len() >= 2,
            "fixture must carry per-arg evidence for both parameters; got {:?}",
            entry.evidence
        );
        assert!(
            matches!(entry.evidence[0], SubValueRelation::PreservedValue),
            "first arg `i` must be preserved — first evidence row should be PreservedValue; got {:?}",
            entry.evidence[0]
        );
        assert!(
            matches!(entry.evidence[1], SubValueRelation::StrictSubValue { .. }),
            "second arg `tail` must carry structural descent; got {:?}",
            entry.evidence[1]
        );

        let call_site = entry.call;
        let transform = dag
            .node_opt(&call_site)
            .and_then(Behavior::as_transform)
            .expect("call site should be a Transform");

        assert!(
            transform.inputs.len() >= 2,
            "ep_g78_acc call should have at least two operand ports; got {}",
            transform.inputs.len()
        );

        let descent_port = per_call_descent_operand_port(&dag, call_site)
            .expect("per_call_descent_operand_port must succeed when pattern exists");

        assert_eq!(
            descent_port,
            transform.inputs[1],
            "descent operand port must be inputs[1] when evidence row 0 is PreservedValue and row 1 proves descent"
        );
        assert_ne!(
            descent_port,
            transform.inputs[0],
            "gate #78 regression: must not treat head operand as descent when evidence selects arg 1"
        );

        let cost = expect_symbolic_cost(&dag, "ep_g78_acc");
        let mut ports = Vec::new();
        linear_size_ports(&cost, &mut ports);
        assert!(
            ports.contains(&descent_port),
            "symbolic cost should reference the evidence-selected descent port {descent_port:?}; got cost={cost:?}"
        );
    });
}
