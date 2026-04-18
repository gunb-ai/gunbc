// Lane 2 Stage 2d / DB-7 — symbolic-cost lens acceptance tests.
//
// Authority: `src/v3/lenses/cost.dag` (projection:
// `src/v3/compiler/src/lens_cost_symbolic_generated.rs`).
//
// These tests pin the per-Behavior lowering contract DB-7 specifies
// — every Behavior variant produces a honest asymptotic bound, and
// the composition algebra in `src/v3/std/algebra.dag` (+ its Rust
// mirror in `dag.rs`) normalizes correctly.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    dominates, iterate, max_path, normalize, sequential, Behavior, PortId, SizeVariable,
    SymbolicCost,
};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn expect_cost(dag: &v3_compiler::dag::Dag, port: PortId) -> SymbolicCost {
    match symbolic_cost_of(dag, &port) {
        SymbolicCostLookup::FoundCost { _0: cost } => cost,
        SymbolicCostLookup::MissingCost => {
            panic!("symbolic_cost_of returned MissingCost for {port:?}")
        }
    }
}

fn bind_cost(source: &str, file: &str, name: &str) -> SymbolicCost {
    let dag = compile_to_dag(source, file).expect("fixture compiles");
    expect_cost(&dag, find_bind_value(&dag, name))
}

fn is_constant(cost: &SymbolicCost) -> bool {
    matches!(cost, SymbolicCost::ConstantCost { .. })
}

fn mentions_linear(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::LinearCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } | SymbolicCost::ProductCost { _0: terms } => {
            terms.iter().any(mentions_linear)
        }
        _ => false,
    }
}

fn size_var(source_port: PortId) -> SizeVariable {
    SizeVariable { source_port }
}

// ── Per-Behavior lowering ────────────────────────────────────────

#[test]
fn value_reports_constant() {
    // `let x = 1` → ConstantCost(0) (leaf literal, no work).
    let dag = compile_to_dag("let x = 1", "test.v3").expect("compiles");
    let cost = expect_cost(&dag, find_bind_value(&dag, "x"));
    assert!(
        is_constant(&cost),
        "literal value should report Constant, got {cost:?}"
    );
}

#[test]
fn transform_single_op_reports_constant_only_sum() {
    // `let x = 1 + 2` → sequential(Constant(1), Constant(0) + Constant(0))
    // After normalization: Constant wrapper carrying the op-count —
    // the lens reports Constant for a pure-scalar single op, which
    // is the correct asymptotic bound: O(1).
    let cost = bind_cost("let x = 1 + 2", "test.v3", "x");
    assert!(
        is_constant(&cost),
        "single scalar op should report Constant, got {cost:?}"
    );
}

#[test]
fn branch_reports_constant_when_both_arms_constant() {
    // `if 1 > 0 then 10 else 20` — both arms are leaf literals, so
    // max_path over two Constants stays Constant.
    let cost = bind_cost("let r = if 1 > 0 then 10 else 20", "test.v3", "r");
    assert!(
        is_constant(&cost),
        "branch over constant arms should report Constant, got {cost:?}"
    );
}

#[test]
fn recursive_fn_reports_linear_via_loop_lowering() {
    // `fn countdown(n) = if n == 0 then 0 else countdown(n - 1)`
    // lowers to a Loop whose source is the `n` param port. The
    // lens's `loop_cost` fires LinearCost(size_var_of(source)),
    // so the bound carries a LinearCost term.
    let dag = compile_to_dag(
        "fn countdown(n: Int) -> Int =\n  if n == 0 then 0 else countdown(n - 1)",
        "countdown.v3",
    )
    .expect("compiles");
    let cost = expect_cost(&dag, find_bind_value(&dag, "countdown"));
    assert!(
        mentions_linear(&cost),
        "recursive fn should surface a LinearCost term, got {cost:?}"
    );
}

// ── Composition algebra ─────────────────────────────────────────

#[test]
fn sum_of_linear_and_constant_normalizes_to_linear() {
    // DB-7 acceptance gate: `SumCost([LinearCost, ConstantCost])`
    // normalizes to `LinearCost` (Constant is dominated).
    let port = PortId::new(0);
    let linear = SymbolicCost::LinearCost { _0: size_var(port) };
    let result = sequential(linear.clone(), SymbolicCost::ConstantCost { _0: 0 });
    assert_eq!(
        result, linear,
        "Linear + Constant(0) should normalize to Linear, got {result:?}"
    );
}

#[test]
fn sum_of_constant_and_linear_normalizes_to_linear() {
    // Dominance is order-independent: Constant + Linear also
    // normalizes to Linear.
    let port = PortId::new(0);
    let linear = SymbolicCost::LinearCost { _0: size_var(port) };
    let result = sequential(SymbolicCost::ConstantCost { _0: 5 }, linear.clone());
    // ConstantCost(5) is non-zero but still dominated by Linear.
    assert!(
        matches!(result, SymbolicCost::LinearCost { .. }),
        "Constant + Linear should normalize to Linear, got {result:?}"
    );
}

#[test]
fn product_of_two_linears_over_same_var_folds_to_polynomial_squared() {
    // DB-7 §"Nested fold detection (O(n²) diagnostic)":
    // `iterate(Linear(n), Linear(n))` folds to `Polynomial(n, 2)`.
    let port = PortId::new(0);
    let linear = SymbolicCost::LinearCost { _0: size_var(port) };
    let result = iterate(linear.clone(), linear);
    match result {
        SymbolicCost::PolynomialCost { var, degree } => {
            assert_eq!(var, size_var(port));
            assert_eq!(degree, 2);
        }
        other => panic!("expected PolynomialCost(n, 2), got {other:?}"),
    }
}

#[test]
fn product_of_linears_over_different_vars_stays_product() {
    // Two folds over DISTINCT lists stay as ProductCost — the
    // two size variables aren't the same, so the polynomial
    // collapse doesn't fire.
    let port_a = PortId::new(0);
    let port_b = PortId::new(1);
    let linear_a = SymbolicCost::LinearCost {
        _0: size_var(port_a),
    };
    let linear_b = SymbolicCost::LinearCost {
        _0: size_var(port_b),
    };
    let result = iterate(linear_a, linear_b);
    assert!(
        matches!(result, SymbolicCost::ProductCost { .. }),
        "Linear(a) * Linear(b) over different ports should stay Product, got {result:?}"
    );
}

#[test]
fn dominance_unknown_dominates_everything() {
    // DB-7 §"Dominance" — Unknown is the safest over-approximation.
    let port = PortId::new(0);
    let unknown = SymbolicCost::UnknownCost {
        _0: "reflection".to_string(),
    };
    let linear = SymbolicCost::LinearCost { _0: size_var(port) };
    assert!(dominates(&unknown, &linear));
    assert!(dominates(&unknown, &SymbolicCost::ConstantCost { _0: 0 }));
}

#[test]
fn dominance_linear_dominates_log_and_constant() {
    let port = PortId::new(0);
    let linear = SymbolicCost::LinearCost { _0: size_var(port) };
    let log = SymbolicCost::LogCost { _0: size_var(port) };
    let constant = SymbolicCost::ConstantCost { _0: 3 };
    assert!(dominates(&linear, &log));
    assert!(dominates(&linear, &constant));
    assert!(!dominates(&constant, &linear));
}

#[test]
fn dominance_polynomial_degree_ordering() {
    let port = PortId::new(0);
    let poly_2 = SymbolicCost::PolynomialCost {
        var: size_var(port),
        degree: 2,
    };
    let poly_3 = SymbolicCost::PolynomialCost {
        var: size_var(port),
        degree: 3,
    };
    assert!(dominates(&poly_3, &poly_2));
    assert!(!dominates(&poly_2, &poly_3));
}

#[test]
fn max_path_returns_dominant_term() {
    // `max_path([Constant(0), Linear(n), Polynomial(n, 2)])` → Polynomial(n, 2).
    let port = PortId::new(0);
    let paths = vec![
        SymbolicCost::ConstantCost { _0: 0 },
        SymbolicCost::LinearCost { _0: size_var(port) },
        SymbolicCost::PolynomialCost {
            var: size_var(port),
            degree: 2,
        },
    ];
    let result = max_path(&paths);
    match result {
        SymbolicCost::PolynomialCost { degree, .. } => assert_eq!(degree, 2),
        other => panic!("expected PolynomialCost, got {other:?}"),
    }
}

#[test]
fn normalize_keeps_singleton_costs_unchanged() {
    // `normalize` is structural — non-Sum/Product variants pass
    // through unmodified.
    let port = PortId::new(0);
    for cost in [
        SymbolicCost::ConstantCost { _0: 7 },
        SymbolicCost::LinearCost { _0: size_var(port) },
        SymbolicCost::LogCost { _0: size_var(port) },
        SymbolicCost::UnknownCost {
            _0: "opaque".to_string(),
        },
    ] {
        let normalized = normalize(cost.clone());
        assert_eq!(normalized, cost, "normalize should pass through leaf costs");
    }
}
