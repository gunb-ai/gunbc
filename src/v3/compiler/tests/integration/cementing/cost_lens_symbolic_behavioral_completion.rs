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
//! nested `SymbolicCost` expected values with `SizeVariable` identity
//! assertions (`M1_2_8_STRUCTURAL_SYMBOLIC_COST_DATA`). The compile-backed
//! consumer-path fixtures remain in `lane2_stage_2d_symbolic_cost_test`; this
//! receipt stays under the per-test ratchet by cementing the generated
//! `SymbolicCost` surface directly.

use v3_compiler::dag::{
    classify_symbolic_cost, dominates, iterate, max_path, ArithmeticOp, AsymptoticClass,
    DegreeAtLeastTwo, Lookup, NonSingletonList, OperatorKind, PortId, SizeVariable, SymbolicCost,
    TransformTarget,
};
use v3_compiler::lens_cost_symbolic::{transform_cost_for_target, SymbolicCostEntry};

fn contains_linear_for_port(cost: &SymbolicCost, port: PortId) -> bool {
    match cost {
        SymbolicCost::LinearCost { _0: var } => var.source_port == port,
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => terms
            .iter()
            .any(|term| contains_linear_for_port(term.as_ref(), port)),
        _ => false,
    }
}

fn bootstrap_ports() -> (PortId, PortId) {
    let dag = v3_compiler::dag::Dag::new();
    let ports: Vec<PortId> = dag.ports().iter().map(|port| port.id()).collect();
    assert!(
        ports.len() >= 2,
        "bootstrap Dag should expose at least two ports for symbolic-cost surface receipts"
    );
    (ports[0], ports[1])
}

fn size_var(source_port: PortId) -> SizeVariable {
    SizeVariable {
        source_port,
        display_name: None,
    }
}

fn linear(source_port: PortId) -> SymbolicCost {
    SymbolicCost::LinearCost {
        _0: size_var(source_port),
    }
}

fn log_cost(source_port: PortId) -> SymbolicCost {
    SymbolicCost::LogCost {
        _0: size_var(source_port),
    }
}

fn constant(value: i64) -> SymbolicCost {
    SymbolicCost::ConstantCost { _0: value }
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

fn product_terms(cost: &SymbolicCost) -> usize {
    match cost {
        SymbolicCost::ProductCost { _0: terms } => terms.iter().count(),
        _ => 0,
    }
}

fn sum_terms(cost: &SymbolicCost) -> usize {
    match cost {
        SymbolicCost::SumCost { _0: terms } => terms.iter().count(),
        _ => 0,
    }
}

#[test]
fn symbolic_cost_surface_cements_product_shape() {
    let (p0, p1) = bootstrap_ports();

    let product = iterate(linear(p0), log_cost(p1));
    assert!(
        matches!(product, SymbolicCost::ProductCost { .. }),
        "iterate(linear, log) should retain ProductCost shape, got {product:?}"
    );
    assert_eq!(
        product_terms(&product),
        2,
        "ProductCost receipt should retain both bound and body terms"
    );
    assert!(
        contains_linear_for_port(&product, p0) && contains_log_for_port(&product, p1),
        "ProductCost receipt should preserve Linear({p0:?}) and Log({p1:?}), got {product:?}"
    );
}

#[test]
fn symbolic_cost_surface_cements_polynomial_classification() {
    let (p0, _) = bootstrap_ports();

    let polynomial = SymbolicCost::PolynomialCost {
        var: size_var(p0),
        degree: DegreeAtLeastTwo::TWO,
    };
    let SymbolicCost::PolynomialCost { var, degree } = polynomial else {
        panic!("explicit polynomial receipt should construct PolynomialCost, got {polynomial:?}");
    };
    assert_eq!(var.source_port, p0);
    assert_eq!(degree.raw(), 2);
    assert_eq!(
        classify_symbolic_cost(SymbolicCost::PolynomialCost { var, degree }),
        AsymptoticClass::ClassQuadratic
    );
}

#[test]
fn symbolic_cost_surface_cements_sum_shape() {
    let (p0, p1) = bootstrap_ports();

    let sum = max_path(&[linear(p0), linear(p1)]);
    assert!(
        matches!(sum, SymbolicCost::SumCost { .. }),
        "incomparable branch costs should retain SumCost shape, got {sum:?}"
    );
    assert_eq!(sum_terms(&sum), 2);
    assert!(
        contains_linear_for_port(&sum, p0) && contains_linear_for_port(&sum, p1),
        "SumCost receipt should preserve both incomparable linear terms, got {sum:?}"
    );
}

#[test]
fn symbolic_cost_surface_cements_dominance_over_sum_children() {
    let (p0, p1) = bootstrap_ports();

    let explicit_sum = SymbolicCost::SumCost {
        _0: NonSingletonList::from_vec(vec![
            Box::new(log_cost(p0)),
            Box::new(SymbolicCost::PolynomialCost {
                var: size_var(p1),
                degree: DegreeAtLeastTwo::new(3).expect("degree >= 2"),
            }),
        ])
        .expect("sum receipt uses two terms"),
    };
    assert!(
        dominates(&explicit_sum, &log_cost(p0)),
        "dominance should scan SumCost children, got {explicit_sum:?}"
    );
}

#[test]
fn symbolic_cost_surface_cements_unknown_classification() {
    assert_eq!(
        classify_symbolic_cost(SymbolicCost::UnknownCost {
            _0: "cementing fallback".to_string()
        }),
        AsymptoticClass::ClassUnknown
    );
}

#[test]
fn symbolic_cost_surface_cements_div_log_cost() {
    let (dividend, divisor) = bootstrap_ports();

    let cheap_inputs = vec![
        SymbolicCostEntry {
            port: dividend,
            cost: Lookup::Hit(constant(0)),
        },
        SymbolicCostEntry {
            port: divisor,
            cost: Lookup::Hit(constant(0)),
        },
    ];
    let Lookup::Hit(div_cost) = transform_cost_for_target(
        &cheap_inputs,
        &TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Div)),
        &[dividend, divisor],
    ) else {
        panic!("Div with operands should produce Hit(LogCost), not Miss");
    };
    assert!(
        contains_log_for_port(&div_cost, dividend),
        "Div should retain LogCost keyed by dividend {dividend:?}, got {div_cost:?}"
    );
}

#[test]
fn symbolic_cost_surface_cements_div_fail_closed_miss() {
    let missing_inputs = transform_cost_for_target(
        &[],
        &TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Div)),
        &[],
    );
    assert!(
        matches!(missing_inputs, Lookup::Miss),
        "Div with no operands should fail closed as Lookup::Miss, got {missing_inputs:?}"
    );
}
